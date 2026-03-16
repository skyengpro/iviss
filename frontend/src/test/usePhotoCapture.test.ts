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
    preprocessForHighRes: vi.fn().mockResolvedValue('data:image/jpeg;base64,processed'),
  },
}));

describe('usePhotoCapture', () => {
  let originalFetch: typeof global.fetch;

  beforeEach(() => {
    originalFetch = global.fetch;
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
    const mockGetScreenshot = () => 'data:image/jpeg;base64,screenshot';
    const mockOnConfirm = vi.fn();

    // Mock fetch for blob conversion (data URL -> Blob)
    global.fetch = vi.fn().mockResolvedValueOnce({
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
      await result.current.captureAndProcess(mockGetScreenshot);
    });

    expect(result.current.state).toBe('result');
    expect(result.current.detectedPlate).toEqual({
      plateNumber: 'CE128BC',
      confidence: 92,
      status: 'valid',
    });
    expect(result.current.editedPlate).toBe('CE128BC');
  });

  it('should handle API errors gracefully', async () => {
    const mockGetScreenshot = () => 'data:image/jpeg;base64,screenshot';

    global.fetch = vi.fn().mockResolvedValueOnce({
      blob: () => Promise.resolve(new Blob(['image'], { type: 'image/jpeg' })),
    });

    vi.mocked(photoPlate).mockResolvedValueOnce({
      data: undefined,
      error: { message: 'Server Error' },
    } as Awaited<ReturnType<typeof photoPlate>>);

    const { result } = renderHook(() => usePhotoCapture());

    await act(async () => {
      await result.current.captureAndProcess(mockGetScreenshot);
    });

    expect(result.current.state).toBe('error');
    expect(result.current.error).toBe('mobileScan.photoError');
  });

  it('should handle no screenshot gracefully', async () => {
    const mockGetScreenshot = () => null;

    const { result } = renderHook(() => usePhotoCapture());

    await act(async () => {
      await result.current.captureAndProcess(mockGetScreenshot);
    });

    expect(result.current.state).toBe('error');
    expect(result.current.error).toBe('mobileScan.photoError');
  });

  it('should reset state on retry', async () => {
    const mockGetScreenshot = () => 'data:image/jpeg;base64,screenshot';

    global.fetch = vi.fn().mockResolvedValueOnce({
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
      await result.current.captureAndProcess(mockGetScreenshot);
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

  it('should handle plate editing and confirmation', async () => {
    const mockGetScreenshot = () => 'data:image/jpeg;base64,screenshot';
    const mockOnConfirm = vi.fn();

    global.fetch = vi.fn().mockResolvedValueOnce({
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
      await result.current.captureAndProcess(mockGetScreenshot);
    });

    // Toggle edit mode
    act(() => {
      result.current.toggleEdit();
    });
    expect(result.current.isEditing).toBe(true);

    // Edit plate
    act(() => {
      result.current.updateEditedPlate('CE129BC');
    });
    expect(result.current.editedPlate).toBe('CE129BC');

    // Confirm with edited plate
    act(() => {
      result.current.confirmPlate();
    });

    expect(mockOnConfirm).toHaveBeenCalledWith(expect.objectContaining({ plateNumber: 'CE129BC' }));
  });

  it('should set status to warning for invalid format plates', async () => {
    const mockGetScreenshot = () => 'data:image/jpeg;base64,screenshot';

    global.fetch = vi.fn().mockResolvedValueOnce({
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
      await result.current.captureAndProcess(mockGetScreenshot);
    });

    expect(result.current.detectedPlate?.status).toBe('warning');
  });
});
