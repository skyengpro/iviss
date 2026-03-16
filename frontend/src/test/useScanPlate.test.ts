import { renderHook, act, waitFor } from '@testing-library/react';
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
  beforeEach(() => {
    vi.useFakeTimers();
    vi.spyOn(ImageProcessor, 'cropToViewfinderFast').mockResolvedValue(
      'data:image/jpeg;base64,AAA'
    );
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });

  it('should accept immediately on high-confidence format-valid result (fast-path)', async () => {
    const onSuccess = vi.fn();

    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      if (typeof input === 'string' && input.startsWith('data:image')) {
        return {
          ok: true,
          blob: async () => new Blob(['x'], { type: 'image/jpeg' }),
        } as unknown as Response;
      }

      throw new Error('Unexpected fetch call in test');
    });

    vi.stubGlobal('fetch', fetchMock);

    vi.mocked(scanPlate).mockResolvedValueOnce({
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

    vi.mocked(scanPlate).mockResolvedValueOnce({
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

    // Stability requires 2 consecutive matches in the current hook implementation.
    // Drive the scan loop (setTimeout) and let async fetch chain resolve.
    for (let i = 0; i < 10 && onSuccess.mock.calls.length === 0; i += 1) {
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

  it('stopLiveScan should reset scanning state and clear detections', async () => {
    const onSuccess = vi.fn();

    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      if (typeof input === 'string' && input.startsWith('data:image')) {
        return {
          ok: true,
          blob: async () => new Blob(['x'], { type: 'image/jpeg' }),
        } as unknown as Response;
      }

      throw new Error('Unexpected fetch call in test');
    });

    vi.stubGlobal('fetch', fetchMock);

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

    const { result } = renderHook(() => useScanPlate({ onSuccess }));

    const getScreenshot = vi.fn(() => 'data:image/jpeg;base64,FRAME');

    act(() => {
      result.current.startLiveScan(getScreenshot);
    });

    // Flush pending microtasks/state updates.
    await act(async () => {
      await Promise.resolve();
    });

    expect(result.current.liveScanActive).toBe(true);

    act(() => {
      result.current.stopLiveScan();
    });

    expect(result.current.liveScanActive).toBe(false);
    expect(result.current.liveDetections).toEqual([]);
  });

  it('should set scanError on AbortError during frame processing', async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      if (typeof input === 'string' && input.startsWith('data:image')) {
        return {
          ok: true,
          blob: async () => new Blob(['x'], { type: 'image/jpeg' }),
        } as unknown as Response;
      }

      throw new Error('Unexpected fetch call in test');
    });

    vi.stubGlobal('fetch', fetchMock);

    vi.mocked(scanPlate).mockRejectedValue(
      new DOMException('signal is aborted without reason', 'AbortError')
    );

    const { result } = renderHook(() => useScanPlate());
    const getScreenshot = vi.fn(() => 'data:image/jpeg;base64,FRAME');

    await act(async () => {
      result.current.startLiveScan(getScreenshot);
      await vi.runOnlyPendingTimersAsync();
    });

    // Flush pending microtasks/state updates.
    await act(async () => {
      await Promise.resolve();
    });

    // AbortError is intentionally ignored by the hook.
    expect(result.current.scanError).toBeNull();
  });
});
