import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import type Webcam from 'react-webcam';
import { useCamera } from '../feature/useCamera';

vi.mock('@/utils/captureFrame', () => ({
  captureFrame: vi.fn(),
}));

import { captureFrame } from '@/utils/captureFrame';

function fakeTrack(overrides: Partial<MediaStreamTrack & { getCapabilities: unknown }> = {}) {
  return {
    getCapabilities: vi.fn(() => ({})),
    applyConstraints: vi.fn().mockResolvedValue(undefined),
    ...overrides,
  } as unknown as MediaStreamTrack;
}

function fakeWebcam(track: MediaStreamTrack | undefined, getScreenshot = vi.fn()) {
  return {
    stream: { getVideoTracks: () => (track ? [track] : []) } as unknown as MediaStream,
    getScreenshot,
  } as unknown as Webcam;
}

describe('useCamera', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should default to environment facing mode', () => {
    const { result } = renderHook(() => useCamera());

    expect(result.current.facingMode).toBe('environment');
    expect(result.current.isCameraReady).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it('should accept custom initial facing mode', () => {
    const { result } = renderHook(() => useCamera({ initialFacingMode: 'user' }));

    expect(result.current.facingMode).toBe('user');
  });

  it('should toggle facing mode between user and environment', () => {
    const { result } = renderHook(() => useCamera());

    expect(result.current.facingMode).toBe('environment');

    act(() => {
      result.current.toggleFacingMode();
    });

    expect(result.current.facingMode).toBe('user');

    act(() => {
      result.current.toggleFacingMode();
    });

    expect(result.current.facingMode).toBe('environment');
  });

  it('should set camera ready state via handleUserMedia', () => {
    const { result } = renderHook(() => useCamera());

    expect(result.current.isCameraReady).toBe(false);

    act(() => {
      result.current.handleUserMedia();
    });

    expect(result.current.isCameraReady).toBe(true);
    expect(result.current.error).toBeNull();
  });

  it('should set error state via handleUserMediaError', () => {
    const { result } = renderHook(() => useCamera());

    act(() => {
      result.current.handleUserMediaError('Camera not available');
    });

    expect(result.current.error).toBe('Camera not available');
    expect(result.current.isCameraReady).toBe(false);
  });

  it('should handle DOMException errors', () => {
    const { result } = renderHook(() => useCamera());

    act(() => {
      result.current.handleUserMediaError(new DOMException('NotAllowedError'));
    });

    expect(result.current.error).toBe('NotAllowedError');
    expect(result.current.isCameraReady).toBe(false);
  });

  it('should return null from getScreenshot when webcam ref is null', () => {
    const { result } = renderHook(() => useCamera());

    const screenshot = result.current.getScreenshot();
    expect(screenshot).toBeNull();
  });

  describe('getPreviewScreenshot', () => {
    it('returns null when webcam ref is null', () => {
      const { result } = renderHook(() => useCamera());

      expect(result.current.getPreviewScreenshot()).toBeNull();
    });

    it('requests a 640px-wide preview capped to the 1920x1080 aspect', () => {
      const { result } = renderHook(() => useCamera());
      const getScreenshot = vi.fn(() => 'data:image/jpeg;base64,preview');
      result.current.webcamRef.current = fakeWebcam(undefined, getScreenshot);

      const preview = result.current.getPreviewScreenshot();

      expect(preview).toBe('data:image/jpeg;base64,preview');
      expect(getScreenshot).toHaveBeenCalledWith({ width: 640, height: 360 });
    });
  });

  describe('captureStill', () => {
    it('returns null without calling captureFrame when webcam ref is null', async () => {
      const { result } = renderHook(() => useCamera());

      const still = await result.current.captureStill();

      expect(still).toBeNull();
      expect(captureFrame).not.toHaveBeenCalled();
    });

    it('delegates to captureFrame with the stream and a getScreenshot fallback', async () => {
      vi.mocked(captureFrame).mockResolvedValue('data:image/jpeg;base64,still');

      const { result } = renderHook(() => useCamera());
      const getScreenshot = vi.fn(() => 'data:image/jpeg;base64,screenshot-fallback');
      const webcam = fakeWebcam(undefined, getScreenshot);
      result.current.webcamRef.current = webcam;

      const still = await result.current.captureStill();

      expect(still).toBe('data:image/jpeg;base64,still');
      expect(captureFrame).toHaveBeenCalledWith(webcam.stream, expect.any(Function));

      // The fallback passed to captureFrame must be wired to the webcam's own getScreenshot.
      const fallback = vi.mocked(captureFrame).mock.calls[0][1];
      fallback();
      expect(getScreenshot).toHaveBeenCalledTimes(1);
    });
  });

  describe('focusOnViewfinder (triggered by handleUserMedia)', () => {
    it('does nothing when there is no video track', async () => {
      const { result } = renderHook(() => useCamera());
      result.current.webcamRef.current = fakeWebcam(undefined);

      await act(async () => {
        result.current.handleUserMedia();
        await Promise.resolve();
      });

      // No track to assert on — the important part is this doesn't throw.
      expect(result.current.isCameraReady).toBe(true);
    });

    it('centers continuous focus and metering on the viewfinder when supported', async () => {
      const track = fakeTrack({
        getCapabilities: vi.fn(() => ({ focusMode: ['continuous'], pointsOfInterest: true })),
      } as never);
      const { result } = renderHook(() => useCamera());
      result.current.webcamRef.current = fakeWebcam(track);

      await act(async () => {
        result.current.handleUserMedia();
        await Promise.resolve();
        await Promise.resolve();
      });

      expect(track.applyConstraints).toHaveBeenCalledWith({
        advanced: [{ focusMode: 'continuous', pointsOfInterest: [{ x: 0.5, y: 0.5 }] }],
      });
    });

    it('does not apply constraints when the track advertises neither capability', async () => {
      const track = fakeTrack();
      const { result } = renderHook(() => useCamera());
      result.current.webcamRef.current = fakeWebcam(track);

      await act(async () => {
        result.current.handleUserMedia();
        await Promise.resolve();
        await Promise.resolve();
      });

      expect(track.applyConstraints).not.toHaveBeenCalled();
    });

    it('does not throw when getCapabilities itself throws', async () => {
      const track = fakeTrack({
        getCapabilities: vi.fn(() => {
          throw new Error('unsupported');
        }),
      } as never);
      const { result } = renderHook(() => useCamera());
      result.current.webcamRef.current = fakeWebcam(track);

      await act(async () => {
        expect(() => result.current.handleUserMedia()).not.toThrow();
        await Promise.resolve();
      });

      expect(track.applyConstraints).not.toHaveBeenCalled();
    });

    it('does not throw when applyConstraints rejects', async () => {
      const track = fakeTrack({
        getCapabilities: vi.fn(() => ({ focusMode: ['continuous'] })),
        applyConstraints: vi.fn().mockRejectedValue(new Error('rejected')),
      } as never);
      const { result } = renderHook(() => useCamera());
      result.current.webcamRef.current = fakeWebcam(track);

      await act(async () => {
        result.current.handleUserMedia();
        await Promise.resolve();
        await Promise.resolve();
        await Promise.resolve();
      });

      expect(result.current.isCameraReady).toBe(true);
    });
  });
});
