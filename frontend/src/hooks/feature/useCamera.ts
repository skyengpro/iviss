import { useState, useCallback, useRef } from 'react';
import Webcam from 'react-webcam';

export type FacingMode = 'user' | 'environment';

interface UseCameraProps {
  initialFacingMode?: FacingMode;
}

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

  const handleUserMedia = useCallback(() => {
    setIsCameraReady(true);
    setError(null);
  }, []);

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
  };
}
