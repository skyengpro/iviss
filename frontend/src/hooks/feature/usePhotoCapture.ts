import { useState, useCallback, useRef } from 'react';
import { ImageProcessor } from '@/utils/imageProcessor';
import { useTranslation } from 'react-i18next';
import { DetectedPlate, PlateStatus } from './useScanPlate';

type PhotoCaptureState = 'idle' | 'processing' | 'result' | 'error';

interface UsePhotoCaptureProps {
  onConfirm?: (plate: DetectedPlate) => void;
}

/**
 * Hook for single-shot photo capture and OCR processing.
 * Unlike `useScanPlate` (continuous stability loop), this captures a single
 * high-resolution frame, uploads it to the backend OCR endpoint, and
 * presents the result for user confirmation before navigation.
 */
export function usePhotoCapture({ onConfirm }: UsePhotoCaptureProps = {}) {
  const { t } = useTranslation();

  const [state, setState] = useState<PhotoCaptureState>('idle');
  const [capturedImageSrc, setCapturedImageSrc] = useState<string | null>(null);
  const [detectedPlate, setDetectedPlate] = useState<DetectedPlate | null>(null);
  const [editedPlate, setEditedPlate] = useState('');
  const [isEditing, setIsEditing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Prevent double-captures
  const isProcessingRef = useRef(false);

  /**
   * Capture a screenshot, preprocess it at high resolution, and send to the
   * backend OCR endpoint. Updates state through idle → processing → result/error.
   */
  const captureAndProcess = useCallback(
    async (getScreenshot: () => string | null) => {
      if (isProcessingRef.current) return;
      isProcessingRef.current = true;

      setState('processing');
      setError(null);
      setDetectedPlate(null);
      setCapturedImageSrc(null);

      try {
        const imageSrc = getScreenshot();
        if (!imageSrc) {
          throw new Error(t('mobileScan.photoError'));
        }

        // Save the raw captured frame for UI preview (freeze frame)
        setCapturedImageSrc(imageSrc);

        // 1. Photo preprocessing (viewfinder crop + sharpen)
        const processed = await ImageProcessor.preprocessForPhotoCapture(imageSrc, t);

        // 2. Convert data URL to blob for multipart upload
        const fetchRes = await fetch(processed);
        const blob = await fetchRes.blob();

        const formData = new FormData();
        formData.append('image', blob, 'photo.jpg');

        // 3. Call backend OCR API
        const apiUrl = (import.meta.env.VITE_API_URL || '').replace(/\/api\/?$/, '');
        const apiResponse = await fetch(`${apiUrl}/api/v1/photo/plate`, {
          method: 'POST',
          body: formData,
        });

        if (!apiResponse.ok) {
          throw new Error(t('mobileScan.photoError'));
        }

        const json = await apiResponse.json();

        if (json.success && json.data?.plate && json.data.plate.trim() !== '') {
          const confidence = json.data.confidence * 100;
          const status: PlateStatus = json.data.format_valid ? 'valid' : 'warning';

          const plate: DetectedPlate = {
            plateNumber: json.data.plate,
            confidence,
            status,
          };

          setDetectedPlate(plate);
          setEditedPlate(plate.plateNumber);
          setState('result');
        } else {
          // OCR returned nothing useful
          setError(t('mobileScan.noPlateDetected'));
          setState('error');
        }
      } catch (err) {
        console.error('Photo capture failed:', err);
        setError(err instanceof Error ? err.message : t('mobileScan.photoError'));
        setState('error');
      } finally {
        isProcessingRef.current = false;
      }
    },
    [t]
  );

  /** Reset to idle state for a new capture attempt. */
  const retry = useCallback(() => {
    setState('idle');
    setCapturedImageSrc(null);
    setDetectedPlate(null);
    setEditedPlate('');
    setIsEditing(false);
    setError(null);
  }, []);

  /** Toggle edit mode for the detected plate text. */
  const toggleEdit = useCallback(() => {
    setIsEditing((prev) => {
      if (prev && detectedPlate) {
        // Cancel edit — revert to original
        setEditedPlate(detectedPlate.plateNumber);
      }
      return !prev;
    });
  }, [detectedPlate]);

  /** Update the edited plate text. */
  const updateEditedPlate = useCallback((value: string) => {
    setEditedPlate(value);
  }, []);

  /** Confirm the plate (using edited text if modified) and trigger navigation. */
  const confirmPlate = useCallback(() => {
    if (!detectedPlate) return;

    const finalPlate: DetectedPlate = {
      ...detectedPlate,
      plateNumber: isEditing ? editedPlate : detectedPlate.plateNumber,
    };

    setIsEditing(false);

    if (onConfirm) {
      onConfirm(finalPlate);
    }
  }, [detectedPlate, isEditing, editedPlate, onConfirm]);

  return {
    // State
    state,
    capturedImageSrc,
    detectedPlate,
    editedPlate,
    isEditing,
    error,

    // Actions
    captureAndProcess,
    retry,
    toggleEdit,
    updateEditedPlate,
    confirmPlate,
  };
}
