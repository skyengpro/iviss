import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

import { useScanPlate } from '@/hooks/feature/useScanPlate';
import { ImageProcessor } from '@/utils/imageProcessor';

vi.mock('@/openapi-rq/requests/services.gen', () => ({
  scanPlate: vi.fn(),
}));

import { scanPlate } from '@/openapi-rq/requests/services.gen';

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe('useScanPlate', () => {
  let fetchMock: any;

  beforeEach(() => {
    vi.useFakeTimers();
    vi.spyOn(ImageProcessor, 'cropToViewfinderFast').mockResolvedValue(
      'data:image/jpeg;base64,AAA'
    );
    vi.mocked(scanPlate).mockResolvedValue({
      data: {
        success: true,
        data: {
          plate: '',
          confidence: 0,
          format_valid: false,
        },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof scanPlate>>);

    fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      if (typeof input === 'string' && input.startsWith('data:image')) {
        return {
          ok: true,
          blob: async () => new Blob(['x'], { type: 'image/jpeg' }),
        } as unknown as Response;
      }
      throw new Error('Unexpected fetch call in test');
    });

    vi.stubGlobal('fetch', fetchMock);
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });

  it('should accept immediately on high-confidence format-valid result (fast-path)', async () => {
    const onSuccess = vi.fn();

    vi.mocked(scanPlate).mockResolvedValue({
      data: {
        success: true,
        data: {
          plate: 'CE128BC',
          confidence: 0.9,
          format_valid: true,
        },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof scanPlate>>);

    const { result } = renderHook(() => useScanPlate({ onSuccess }));
    const getScreenshot = vi.fn(() => 'data:image/jpeg;base64,FRAME');

    act(() => {
      result.current.startLiveScan(getScreenshot);
    });

    // Drive the scan loop
    for (let i = 0; i < 5 && onSuccess.mock.calls.length === 0; i += 1) {
      await act(async () => {
        await vi.advanceTimersByTimeAsync(120);
        await Promise.resolve();
        await Promise.resolve();
      });
    }

    expect(onSuccess).toHaveBeenCalledTimes(1);
    expect(onSuccess).toHaveBeenCalledWith({
      plateNumber: 'CE128BC',
      confidence: 90,
      status: 'valid',
    });
    expect(result.current.liveScanActive).toBe(false);
  });

  it('startLiveScan should no-op if already scanning', async () => {
    const { result } = renderHook(() => useScanPlate());
    const getScreenshot = vi.fn(() => 'data:image/jpeg;base64,FRAME');

    await act(async () => {
      result.current.startLiveScan(getScreenshot);
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(result.current.liveScanActive).toBe(true);
    const initialDetections = result.current.liveDetections;

    // Call again
    act(() => {
      result.current.startLiveScan(getScreenshot);
    });

    // Should not have reset state
    expect(result.current.liveDetections).toBe(initialDetections);
  });

  it('stopLiveScan should reset scanning state and clear detections', async () => {
    vi.mocked(scanPlate).mockResolvedValueOnce({
      data: {
        success: true,
        data: {
          plate: 'CE128BC',
          confidence: 0.5,
          format_valid: false,
        },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof scanPlate>>);

    const { result } = renderHook(() => useScanPlate());
    const getScreenshot = vi.fn(() => 'data:image/jpeg;base64,FRAME');

    act(() => {
      result.current.startLiveScan(getScreenshot);
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
      await Promise.resolve();
    });

    expect(result.current.liveScanActive).toBe(true);

    act(() => {
      result.current.stopLiveScan();
    });

    expect(result.current.liveScanActive).toBe(false);
    expect(result.current.liveDetections).toEqual([]);
    expect(result.current.scanError).toBeNull();
  });

  it('should ignore AbortError during frame processing without setting scanError', async () => {
    vi.mocked(scanPlate).mockRejectedValueOnce(
      new DOMException('signal is aborted without reason', 'AbortError')
    );

    const { result } = renderHook(() => useScanPlate());
    const getScreenshot = vi.fn(() => 'data:image/jpeg;base64,FRAME');

    await act(async () => {
      result.current.startLiveScan(getScreenshot);
      await vi.advanceTimersByTimeAsync(0);
      await Promise.resolve();
    });

    expect(result.current.scanError).toBeNull();
  });

  it('should set scanError on non-abort API failure', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined);

    vi.mocked(scanPlate).mockResolvedValueOnce({
      data: undefined,
      error: { message: 'Server crashed' },
    } as Awaited<ReturnType<typeof scanPlate>>);

    const { result } = renderHook(() => useScanPlate());
    const getScreenshot = vi.fn(() => 'data:image/jpeg;base64,FRAME');

    act(() => {
      result.current.startLiveScan(getScreenshot);
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(result.current.scanError).toBe('OCR API failed');
    expect(console.error).toHaveBeenCalledWith('Frame processing failed:', expect.any(Error));
  });

  it('should add result to liveDetections when processing a frame successfully', async () => {
    vi.mocked(scanPlate).mockResolvedValueOnce({
      data: {
        success: true,
        data: {
          plate: 'LT390HN',
          confidence: 0.85,
          format_valid: true,
        },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof scanPlate>>);

    const { result } = renderHook(() => useScanPlate());
    const getScreenshot = vi.fn(() => 'data:image/jpeg;base64,FRAME');

    act(() => {
      result.current.startLiveScan(getScreenshot);
    });

    // Wait for the first frame to be processed
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(result.current.liveDetections.length).toBeGreaterThan(0);
    expect(result.current.liveDetections[0].plateNumber).toBe('LT390HN');
    expect(result.current.liveDetections[0].status).toBe('valid');
  });

  it('does not vote on stability for reads that fail format validation', async () => {
    // Noise that doesn't even parse as a plate must never delay or wrongly
    // confirm the consensus — only format_valid reads should vote.
    vi.mocked(scanPlate).mockResolvedValue({
      data: {
        success: true,
        data: {
          plate: 'NOTAPLATE',
          confidence: 0.9,
          format_valid: false,
        },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof scanPlate>>);

    const onSuccess = vi.fn();
    const { result } = renderHook(() => useScanPlate({ onSuccess }));
    const getScreenshot = vi.fn(() => 'data:image/jpeg;base64,FRAME');

    act(() => {
      result.current.startLiveScan(getScreenshot);
    });

    for (let i = 0; i < 6; i += 1) {
      await act(async () => {
        await vi.advanceTimersByTimeAsync(120);
        await Promise.resolve();
        await Promise.resolve();
      });
    }

    expect(onSuccess).not.toHaveBeenCalled();
    // Still visible in the raw detections feed — just never confirmed as stable.
    expect(result.current.liveDetections.some((d) => d.plateNumber === 'NOTAPLATE')).toBe(true);
  });

  it('confirms a stable plate on the real field-log confidence regime (0-63, not 76-92)', async () => {
    // Once mean_text_conf is exposed unmodified, measured confidences on real
    // plates fall between 0 and 63 (median near 0-16) — a fixture stuck at
    // 76-92 would validate a path that doesn't exist in production.
    vi.mocked(scanPlate).mockResolvedValue({
      data: {
        success: true,
        data: {
          plate: 'CE568LR',
          confidence: 0.12,
          format_valid: true,
        },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof scanPlate>>);

    const onSuccess = vi.fn();
    const { result } = renderHook(() => useScanPlate({ onSuccess }));
    const getScreenshot = vi.fn(() => 'data:image/jpeg;base64,FRAME');

    act(() => {
      result.current.startLiveScan(getScreenshot);
    });

    for (let i = 0; i < 6 && onSuccess.mock.calls.length === 0; i += 1) {
      await act(async () => {
        await vi.advanceTimersByTimeAsync(120);
        await Promise.resolve();
        await Promise.resolve();
      });
    }

    expect(onSuccess).toHaveBeenCalledWith(
      expect.objectContaining({ plateNumber: 'CE568LR', confidence: 12 })
    );
  });

  it('forwards getViewfinderBox to cropToViewfinderFast so the live crop matches the on-screen overlay', async () => {
    const getViewfinderBox = vi.fn(() => ({ width: 390, height: 620 }));
    const { result } = renderHook(() => useScanPlate({ getViewfinderBox }));
    const getScreenshot = vi.fn(() => 'data:image/jpeg;base64,FRAME');

    act(() => {
      result.current.startLiveScan(getScreenshot);
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
      await Promise.resolve();
    });

    expect(ImageProcessor.cropToViewfinderFast).toHaveBeenCalledWith(
      'data:image/jpeg;base64,FRAME',
      expect.any(Function),
      { width: 390, height: 620 }
    );
  });

  it('cleanup effect aborts in-flight request and clears timeout on unmount', () => {
    vi.mocked(scanPlate).mockImplementationOnce(
      () => new Promise(() => {}) as ReturnType<typeof scanPlate>
    );

    const { result, unmount } = renderHook(() => useScanPlate());
    const getScreenshot = vi.fn(() => 'data:image/jpeg;base64,FRAME');

    act(() => {
      result.current.startLiveScan(getScreenshot);
    });

    // AbortController is created
    expect(result.current.liveScanActive).toBe(true);

    unmount();

    // The AbortController abort would run, and interval cleared. We can't directly
    // observe the internal AbortController from outside without mocking it, but we
    // can verify the test completes cleanly without memory leaks or pending timers.
    const timers = vi.getTimerCount();
    expect(timers).toBe(0);
  });
});
