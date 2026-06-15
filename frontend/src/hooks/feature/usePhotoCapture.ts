import { useState, useCallback, useRef } from 'react';
import { ImageProcessor } from '@/utils/imageProcessor';
import { useTranslation } from 'react-i18next';
import { DetectedPlate, PlateStatus } from './useScanPlate';
import { photoPlate } from '@/openapi-rq/requests/services.gen';
import type { ScanPlateResponse } from '@/openapi-rq/requests/types.gen';

function normalizePlateCandidate(v: unknown): string {
  if (typeof v !== 'string') return '';
  return v
    .toUpperCase()
    .replace(/[^A-Z0-9]/g, '')
    .trim();
}

function findPlateInText(text: string): string {
  const cleaned = normalizePlateCandidate(text);
  for (let len = 12; len >= 6; len -= 1) {
    if (cleaned.length < len) continue;

    for (let start = 0; start <= cleaned.length - len; start += 1) {
      const candidate = cleaned.slice(start, start + len);
      const classified = ImageProcessor.classifyCameroonPlate(candidate);
      if (classified) return classified.plate;
    }
  }

  return '';
}

function extractPlateFromAny(json: unknown): {
  plate: string;
  confidence?: number;
  formatValid?: boolean;
} {
  const obj = json && typeof json === 'object' ? (json as Record<string, unknown>) : undefined;
  const data =
    obj?.data && typeof obj.data === 'object' ? (obj.data as Record<string, unknown>) : undefined;

  const fromData = normalizePlateCandidate(data?.plate);
  const fromTop = normalizePlateCandidate(obj?.plate);
  const fromAlt1 = normalizePlateCandidate(data?.plateNumber);
  const fromAlt2 = normalizePlateCandidate(data?.license_plate);

  const plate = fromData || fromTop || fromAlt1 || fromAlt2;

  const rawText = typeof data?.raw_text === 'string' ? (data.raw_text as string) : '';
  const finalPlate = plate || findPlateInText(rawText);

  const confRaw = (data?.confidence ?? obj?.confidence) as unknown;
  const confidence = typeof confRaw === 'number' ? confRaw : undefined;

  const fvRaw = (data?.format_valid ?? obj?.format_valid) as unknown;
  const formatValid = typeof fvRaw === 'boolean' ? fvRaw : undefined;

  return { plate: finalPlate, confidence, formatValid };
}

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
   * Capture a screenshot, assess quality, preprocess by cropping to the
   * viewfinder guide frame, and send to the backend OCR endpoint.
   * Updates state through idle → processing → result/error.
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

        // ── Quality gate — reject blurry / dark / bright images early ──────
        const quality = await ImageProcessor.assessImageQuality(imageSrc, t);
        if (!quality.isAcceptable) {
          setError(quality.feedback);
          setState('error');
          return;
        }

        const sendToOcr = async (processedDataUrl: string) => {
          const fetchRes = await fetch(processedDataUrl);
          const blob = await fetchRes.blob();

          const file = new File([blob], 'photo.jpg', {
            type: blob.type || 'image/jpeg',
          });

          const apiResponse = await photoPlate({
            body: { image: file },
            throwOnError: false,
          });

          if (apiResponse.error || !apiResponse.data) {
            throw new Error(t('mobileScan.photoError'));
          }

          return apiResponse.data as ScanPlateResponse;
        };

        // ── Primary attempt — viewfinder-cropped, plate-ratio image ───────
        const primaryProcessed = await ImageProcessor.preprocessForPhoto(imageSrc, t);
        let json = await sendToOcr(primaryProcessed);

        let extracted = extractPlateFromAny(json);

        // ── Fallback — only if primary returned nothing useful ────────────
        if ((!json?.success || extracted.plate === '') && json?.success !== false) {
          const fallbackProcessed = await ImageProcessor.preprocessForPhotoCapture(imageSrc, t);
          json = await sendToOcr(fallbackProcessed);
          extracted = extractPlateFromAny(json);
        }

        if (json?.success && extracted.plate !== '') {
          const confidence =
            (typeof extracted.confidence === 'number' ? extracted.confidence : 0) * 100;
          const status: PlateStatus = extracted.formatValid ? 'valid' : 'warning';

          const plate: DetectedPlate = {
            plateNumber: extracted.plate,
            confidence,
            status,
          };

          setDetectedPlate(plate);
          setEditedPlate(plate.plateNumber);
          setState('result');
        } else {
          setError(t('mobileScan.noPlateDetected'));
          setState('error');
        }
      } catch (err) {
        console.error('Photo capture failed:', err);
        if (err instanceof Error && err.message.startsWith('mobileScan.')) {
          setError(err.message);
        } else {
          setError(t('mobileScan.photoError'));
        }
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
