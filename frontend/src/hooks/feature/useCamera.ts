import { useState, useCallback, useRef } from 'react';
import Webcam from 'react-webcam';
import { captureFrame } from '@/utils/captureFrame';

export type FacingMode = 'user' | 'environment';

interface UseCameraProps {
  initialFacingMode?: FacingMode;
}

// focusMode / pointsOfInterest are non-standard, missing from TS's lib.dom.d.ts.
declare global {
  interface MediaTrackConstraintSet {
    focusMode?: string;
    pointsOfInterest?: Array<{ x: number; y: number }>;
  }
  interface MediaTrackCapabilities {
    focusMode?: string[];
    pointsOfInterest?: boolean;
  }
}

const PREVIEW_SCREENSHOT_WIDTH = 640;
const PREVIEW_SCREENSHOT_HEIGHT = 360; // matches the requested 1920x1080 (16:9) capture

/**
 * Hook to manage camera state and operations.
 * Separates camera hardware interaction from scanning logic.
 */
export function useCamera({ initialFacingMode = 'environment' }: UseCameraProps = {}) {
  const [facingMode, setFacingMode] = useState<FacingMode>(initialFacingMode);
  const [isCameraReady, setIsCameraReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const webcamRef = useRef<Webcam>(null);

  const toggleFacingMode = useCallback(() => {
    setFacingMode((prev) => (prev === 'user' ? 'environment' : 'user'));
    setIsCameraReady(false);
  }, []);

  /**
   * Centers continuous focus and metering on the viewfinder frame instead of
   * the whole scene — the direct fix for backlit/bright-sky exposure. Both
   * constraints are non-standard and capability-gated; no-ops silently where
   * unsupported (most of Safari, notably).
   */
  const focusOnViewfinder = useCallback(async () => {
    const track = webcamRef.current?.stream?.getVideoTracks()[0];
    if (!track || typeof track.getCapabilities !== 'function') return;

    let capabilities: MediaTrackCapabilities;
    try {
      capabilities = track.getCapabilities();
    } catch {
      return;
    }

    const advanced: MediaTrackConstraintSet = {};
    if (capabilities.focusMode?.includes('continuous')) {
      advanced.focusMode = 'continuous';
    }
    if (capabilities.pointsOfInterest) {
      advanced.pointsOfInterest = [{ x: 0.5, y: 0.5 }];
    }

    if (Object.keys(advanced).length === 0) return;

    try {
      await track.applyConstraints({ advanced: [advanced] });
    } catch {
      // best-effort — camera keeps its default focus/exposure behavior
    }
  }, []);

  const handleUserMedia = useCallback(() => {
    setIsCameraReady(true);
    setError(null);
    void focusOnViewfinder();
  }, [focusOnViewfinder]);

  const handleUserMediaError = useCallback((err: string | DOMException) => {
    console.error('Camera Error:', err);
    setError(typeof err === 'string' ? err : err.message);
    setIsCameraReady(false);
  }, []);

  const getScreenshot = useCallback(() => {
    if (webcamRef.current) {
      return webcamRef.current.getScreenshot();
    }
    return null;
  }, []);

  /** Capped preview screenshot for lightweight, continuous coaching measurements. */
  const getPreviewScreenshot = useCallback(() => {
    if (!webcamRef.current) return null;
    return webcamRef.current.getScreenshot({
      width: PREVIEW_SCREENSHOT_WIDTH,
      height: PREVIEW_SCREENSHOT_HEIGHT,
    });
  }, []);

  /**
   * True still capture for the photo-mode shutter button — grabFrame() at
   * native track resolution first, takePhoto() second, getScreenshot()
   * fallback last. See utils/captureFrame.ts.
   */
  const captureStill = useCallback(async (): Promise<string | null> => {
    const webcam = webcamRef.current;
    if (!webcam) return null;
    return captureFrame(webcam.stream, () => webcam.getScreenshot());
  }, []);

  return {
    webcamRef,
    facingMode,
    setFacingMode,
    isCameraReady,
    error,
    toggleFacingMode,
    handleUserMedia,
    handleUserMediaError,
    getScreenshot,
    getPreviewScreenshot,
    captureStill,
  };
}
