import { renderHook, act } from '@testing-library/react';
import { usePhotoCapture } from '@/hooks/feature/usePhotoCapture';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

vi.mock('@/openapi-rq/requests/services.gen', () => ({
  photoPlate: vi.fn(),
}));

import { photoPlate } from '@/openapi-rq/requests/services.gen';

// Mock react-i18next
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

// Mock ImageProcessor
vi.mock('@/utils/imageProcessor', () => ({
  ImageProcessor: {
    preprocessForPhoto: vi.fn().mockResolvedValue('data:image/jpeg;base64,processed_photo'),
    preprocessForPhotoCapture: vi
      .fn()
      .mockResolvedValue('data:image/jpeg;base64,processed_capture'),
    assessImageQuality: vi.fn().mockResolvedValue({ isAcceptable: true, feedback: '' }),
  },
}));

import { ImageProcessor } from '@/utils/imageProcessor';

describe('usePhotoCapture', () => {
  let originalFetch: typeof global.fetch;
  const mockCaptureStill = async () => 'data:image/jpeg;base64,screenshot';

  beforeEach(() => {
    originalFetch = global.fetch;
    vi.clearAllMocks();
    vi.mocked(ImageProcessor.assessImageQuality).mockResolvedValue({
      isAcceptable: true,
      feedback: '',
    });
    vi.mocked(ImageProcessor.preprocessForPhoto).mockResolvedValue(
      'data:image/jpeg;base64,processed_photo'
    );
    vi.mocked(ImageProcessor.preprocessForPhotoCapture).mockResolvedValue(
      'data:image/jpeg;base64,processed_capture'
    );
  });

  afterEach(() => {
    global.fetch = originalFetch;
    vi.restoreAllMocks();
  });

  it('should start in idle state', () => {
    const { result } = renderHook(() => usePhotoCapture());

    expect(result.current.state).toBe('idle');
    expect(result.current.detectedPlate).toBeNull();
    expect(result.current.error).toBeNull();
    expect(result.current.isEditing).toBe(false);
  });

  it('should process a capture successfully', async () => {
    const mockOnConfirm = vi.fn();

    // Mock fetch for blob conversion (data URL -> Blob)
    global.fetch = vi.fn().mockResolvedValue({
      blob: () => Promise.resolve(new Blob(['image'], { type: 'image/jpeg' })),
    });

    vi.mocked(photoPlate).mockResolvedValueOnce({
      data: {
        success: true,
        data: {
          plate: 'CE128BC',
          confidence: 0.92,
          format_valid: true,
        },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof photoPlate>>);

    const { result } = renderHook(() => usePhotoCapture({ onConfirm: mockOnConfirm }));

    await act(async () => {
      await result.current.captureAndProcess(mockCaptureStill);
    });

    expect(result.current.state).toBe('result');
    expect(result.current.detectedPlate).toEqual({
      plateNumber: 'CE128BC',
      confidence: 92,
      status: 'valid',
    });
    expect(result.current.editedPlate).toBe('CE128BC');
    expect(ImageProcessor.assessImageQuality).toHaveBeenCalledWith(
      'data:image/jpeg;base64,screenshot',
      expect.any(Function)
    );
    expect(ImageProcessor.preprocessForPhoto).toHaveBeenCalledWith(
      'data:image/jpeg;base64,screenshot',
      expect.any(Function)
    );
    expect(ImageProcessor.preprocessForPhotoCapture).not.toHaveBeenCalled();
  });

  it('should confirm on the real field-log confidence regime (0-63, not 76-92)', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      blob: () => Promise.resolve(new Blob(['image'], { type: 'image/jpeg' })),
    });

    vi.mocked(photoPlate).mockResolvedValueOnce({
      data: {
        success: true,
        data: {
          plate: 'CE568LR',
          confidence: 0.16,
          format_valid: true,
        },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof photoPlate>>);

    const { result } = renderHook(() => usePhotoCapture());

    await act(async () => {
      await result.current.captureAndProcess(mockCaptureStill);
    });

    expect(result.current.state).toBe('result');
    expect(result.current.detectedPlate).toEqual({
      plateNumber: 'CE568LR',
      confidence: 16,
      status: 'valid',
    });
  });

  it('should stop before OCR when image quality is not acceptable', async () => {
    vi.mocked(ImageProcessor.assessImageQuality).mockResolvedValueOnce({
      isAcceptable: false,
      feedback: 'mobileScan.qualityTooBlurry',
    });

    const { result } = renderHook(() => usePhotoCapture());

    await act(async () => {
      await result.current.captureAndProcess(mockCaptureStill);
    });

    expect(result.current.state).toBe('error');
    expect(result.current.error).toBe('mobileScan.qualityTooBlurry');
    expect(result.current.capturedImageSrc).toBe('data:image/jpeg;base64,screenshot');
    expect(ImageProcessor.preprocessForPhoto).not.toHaveBeenCalled();
    expect(ImageProcessor.preprocessForPhotoCapture).not.toHaveBeenCalled();
    expect(photoPlate).not.toHaveBeenCalled();
  });

  it('should handle API errors gracefully', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined);

    global.fetch = vi.fn().mockResolvedValue({
      blob: () => Promise.resolve(new Blob(['image'], { type: 'image/jpeg' })),
    });

    vi.mocked(photoPlate).mockResolvedValueOnce({
      data: undefined,
      error: { message: 'Server Error' },
    } as Awaited<ReturnType<typeof photoPlate>>);

    const { result } = renderHook(() => usePhotoCapture());

    await act(async () => {
      await result.current.captureAndProcess(mockCaptureStill);
    });

    expect(result.current.state).toBe('error');
    expect(result.current.error).toBe('mobileScan.photoError');
  });

  it('should handle no still captured gracefully', async () => {
    const mockCaptureStillNull = async () => null;

    vi.spyOn(console, 'error').mockImplementation(() => undefined);

    const { result } = renderHook(() => usePhotoCapture());

    await act(async () => {
      await result.current.captureAndProcess(mockCaptureStillNull);
    });

    expect(result.current.state).toBe('error');
    expect(result.current.error).toBe('mobileScan.photoError');
  });

  it('should fallback to preprocessForPhotoCapture if first OCR returns no plate', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      blob: () => Promise.resolve(new Blob(['image'], { type: 'image/jpeg' })),
    });

    // First call returns empty plate
    vi.mocked(photoPlate).mockResolvedValueOnce({
      data: {
        success: true,
        data: { plate: '', confidence: 0, format_valid: false },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof photoPlate>>);

    // Second call (fallback) returns actual plate
    vi.mocked(photoPlate).mockResolvedValueOnce({
      data: {
        success: true,
        data: { plate: 'LT390HN', confidence: 0.88, format_valid: true },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof photoPlate>>);

    const { result } = renderHook(() => usePhotoCapture());

    await act(async () => {
      await result.current.captureAndProcess(mockCaptureStill);
    });

    expect(ImageProcessor.preprocessForPhoto).toHaveBeenCalledTimes(1);
    expect(ImageProcessor.preprocessForPhotoCapture).toHaveBeenCalledTimes(1);
    expect(photoPlate).toHaveBeenCalledTimes(2);

    expect(result.current.state).toBe('result');
    expect(result.current.detectedPlate).toEqual({
      plateNumber: 'LT390HN',
      confidence: 88,
      status: 'valid',
    });
  });

  it('should extract a plate from raw_text when normalized plate is empty', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      blob: () => Promise.resolve(new Blob(['image'], { type: 'image/jpeg' })),
    });

    vi.mocked(photoPlate).mockResolvedValueOnce({
      data: {
        success: true,
        data: {
          plate: '',
          raw_text: 'OCR: ce 128 bc',
          confidence: 0.81,
          format_valid: true,
        },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof photoPlate>>);

    const { result } = renderHook(() => usePhotoCapture());

    await act(async () => {
      await result.current.captureAndProcess(mockCaptureStill);
    });

    expect(result.current.state).toBe('result');
    expect(result.current.detectedPlate).toEqual({
      plateNumber: 'CE128BC',
      confidence: 81,
      status: 'valid',
    });
    expect(ImageProcessor.preprocessForPhotoCapture).not.toHaveBeenCalled();
    expect(photoPlate).toHaveBeenCalledTimes(1);
  });

  it('should normalize top-level OCR plate fields for backward-compatible responses', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      blob: () => Promise.resolve(new Blob(['image'], { type: 'image/jpeg' })),
    });

    vi.mocked(photoPlate).mockResolvedValueOnce({
      data: {
        success: true,
        plate: 'lt-390-hn',
        confidence: 0.76,
        format_valid: true,
      },
      error: undefined,
    } as Awaited<ReturnType<typeof photoPlate>>);

    const { result } = renderHook(() => usePhotoCapture());

    await act(async () => {
      await result.current.captureAndProcess(mockCaptureStill);
    });

    expect(result.current.state).toBe('result');
    expect(result.current.detectedPlate).toEqual({
      plateNumber: 'LT390HN',
      confidence: 76,
      status: 'valid',
    });
  });

  it('should set error when both OCR calls fail to find a plate', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined);

    global.fetch = vi.fn().mockResolvedValue({
      blob: () => Promise.resolve(new Blob(['image'], { type: 'image/jpeg' })),
    });

    // First call returns empty
    vi.mocked(photoPlate).mockResolvedValueOnce({
      data: {
        success: true,
        data: { plate: '', confidence: 0, format_valid: false },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof photoPlate>>);

    // Second call returns empty too
    vi.mocked(photoPlate).mockResolvedValueOnce({
      data: {
        success: true,
        data: { plate: '', confidence: 0, format_valid: false },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof photoPlate>>);

    const { result } = renderHook(() => usePhotoCapture());

    await act(async () => {
      await result.current.captureAndProcess(mockCaptureStill);
    });

    expect(result.current.state).toBe('error');
    expect(result.current.error).toBe('mobileScan.noPlateDetected');
  });

  it('should prevent double-capture while processing', async () => {
    // Make fetch hang slightly so we can trigger again
    global.fetch = vi.fn().mockImplementation(() => {
      return new Promise((resolve) => {
        setTimeout(() => {
          resolve({ blob: () => Promise.resolve(new Blob(['image'], { type: 'image/jpeg' })) });
        }, 50);
      });
    });

    vi.mocked(photoPlate).mockResolvedValue({
      data: {
        success: true,
        data: { plate: 'CE128BC', confidence: 0.9, format_valid: true },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof photoPlate>>);

    const { result } = renderHook(() => usePhotoCapture());

    let promise1: any;
    let promise2: any;

    act(() => {
      promise1 = result.current.captureAndProcess(mockCaptureStill);
      // Immediately call again
      promise2 = result.current.captureAndProcess(mockCaptureStill);
    });

    await act(async () => {
      await Promise.all([promise1, promise2]);
    });

    // Ensure API was only called once
    expect(photoPlate).toHaveBeenCalledTimes(1);
  });

  it('should reset state on retry', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      blob: () => Promise.resolve(new Blob(['image'], { type: 'image/jpeg' })),
    });

    vi.mocked(photoPlate).mockResolvedValueOnce({
      data: {
        success: true,
        data: { plate: 'CE128BC', confidence: 0.9, format_valid: true },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof photoPlate>>);

    const { result } = renderHook(() => usePhotoCapture());

    await act(async () => {
      await result.current.captureAndProcess(mockCaptureStill);
    });

    expect(result.current.state).toBe('result');

    act(() => {
      result.current.retry();
    });

    expect(result.current.state).toBe('idle');
    expect(result.current.detectedPlate).toBeNull();
    expect(result.current.editedPlate).toBe('');
    expect(result.current.isEditing).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it('should handle plate editing, cancelling edit reverts editedPlate', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      blob: () => Promise.resolve(new Blob(['image'], { type: 'image/jpeg' })),
    });

    vi.mocked(photoPlate).mockResolvedValueOnce({
      data: {
        success: true,
        data: { plate: 'CE128BC', confidence: 0.85, format_valid: true },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof photoPlate>>);

    const { result } = renderHook(() => usePhotoCapture());

    await act(async () => {
      await result.current.captureAndProcess(mockCaptureStill);
    });

    act(() => {
      result.current.toggleEdit();
    });
    expect(result.current.isEditing).toBe(true);

    act(() => {
      result.current.updateEditedPlate('CE129BC');
    });
    expect(result.current.editedPlate).toBe('CE129BC');

    // Cancel edit
    act(() => {
      result.current.toggleEdit();
    });

    // Reverted back to original
    expect(result.current.isEditing).toBe(false);
    expect(result.current.editedPlate).toBe('CE128BC');
  });

  it('confirmPlate() should be a no-op when detectedPlate is null', () => {
    const mockOnConfirm = vi.fn();
    const { result } = renderHook(() => usePhotoCapture({ onConfirm: mockOnConfirm }));

    act(() => {
      result.current.confirmPlate();
    });

    expect(mockOnConfirm).not.toHaveBeenCalled();
  });

  it('should handle confirm with edited plate', async () => {
    const mockOnConfirm = vi.fn();

    global.fetch = vi.fn().mockResolvedValue({
      blob: () => Promise.resolve(new Blob(['image'], { type: 'image/jpeg' })),
    });

    vi.mocked(photoPlate).mockResolvedValueOnce({
      data: {
        success: true,
        data: { plate: 'CE128BC', confidence: 0.85, format_valid: true },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof photoPlate>>);

    const { result } = renderHook(() => usePhotoCapture({ onConfirm: mockOnConfirm }));

    await act(async () => {
      await result.current.captureAndProcess(mockCaptureStill);
    });

    act(() => {
      result.current.toggleEdit();
      result.current.updateEditedPlate('CE129BC');
    });

    act(() => {
      result.current.confirmPlate();
    });

    expect(mockOnConfirm).toHaveBeenCalledWith(expect.objectContaining({ plateNumber: 'CE129BC' }));
  });

  it('should set status to warning for invalid format plates', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      blob: () => Promise.resolve(new Blob(['image'], { type: 'image/jpeg' })),
    });

    vi.mocked(photoPlate).mockResolvedValueOnce({
      data: {
        success: true,
        data: { plate: 'ABC123', confidence: 0.7, format_valid: false },
      },
      error: undefined,
    } as Awaited<ReturnType<typeof photoPlate>>);

    const { result } = renderHook(() => usePhotoCapture());

    await act(async () => {
      await result.current.captureAndProcess(mockCaptureStill);
    });

    expect(result.current.detectedPlate?.status).toBe('warning');
  });
});
