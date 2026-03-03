import { useState, useCallback, useRef, useEffect } from 'react';
import { ImageProcessor } from '@/utils/imageProcessor';
import { useTranslation } from 'react-i18next';
import { useStabilityDetection, DetectionResult } from './useStabilityDetection';

export type PlateStatus = 'valid' | 'warning' | 'critical';

export interface DetectedPlate {
  plateNumber: string;
  confidence: number;
  status: PlateStatus;
}

interface UseScanPlateProps {
  onSuccess?: (plate: DetectedPlate) => void;
}

/**
 * Hook to manage the license plate scanning process (The "Eyes" of the system).
 * Implements 500ms frame sampling and hybrid OCR communication.
 */
export function useScanPlate({ onSuccess }: UseScanPlateProps = {}) {
  const { t } = useTranslation();
  const [isScanning, setIsScanning] = useState(false);
  const [isLiveProcessing, setIsLiveProcessing] = useState(false);
  const [liveScanActive, setLiveScanActive] = useState(false);
  const [liveDetections, setLiveDetections] = useState<DetectedPlate[]>([]);
  const [scanError, setScanError] = useState<string | null>(null);

  const lastHandledStablePlateRef = useRef<string | null>(null);

  const activeRequestRef = useRef<AbortController | null>(null);

  const { addDetection, resetStability, stableResult } = useStabilityDetection({
    requiredMatches: 2,
    minConfidence: 40,
  });

  const scanIntervalRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Effect to handle success when a stable result is found
  useEffect(() => {
    if (stableResult?.plateNumber) {
      if (lastHandledStablePlateRef.current === stableResult.plateNumber) {
        return;
      }
      lastHandledStablePlateRef.current = stableResult.plateNumber;

      const confirmedPlate: DetectedPlate = {
        plateNumber: stableResult.plateNumber,
        confidence: stableResult.confidence,
        status: 'valid',
      };

      // Update local detections list if not already present
      setLiveDetections((prev) => {
        if (prev.some((d) => d.plateNumber === confirmedPlate.plateNumber)) return prev;
        return [confirmedPlate, ...prev].slice(0, 10);
      });

      if (onSuccess) {
        onSuccess(confirmedPlate);
      }

      // Stop scanning once success is reached
      setLiveScanActive(false);
      scanActiveRef.current = false; // Also stop the ref loop
      if (scanIntervalRef.current) {
        clearTimeout(scanIntervalRef.current);
        scanIntervalRef.current = null;
      }
    }
  }, [stableResult, onSuccess]);

  const processFrame = useCallback(
    async (imageSrc: string) => {
      setIsLiveProcessing(true);
      try {
        // Cancel any previous in-flight request so we don't wait on stale frames
        if (activeRequestRef.current) {
          activeRequestRef.current.abort();
          activeRequestRef.current = null;
        }

        const controller = new AbortController();
        activeRequestRef.current = controller;

        // 1. Optimize image (smaller crop + lower quality for speed)
        const compressedImage = await ImageProcessor.cropToViewfinderFast(imageSrc, t);

        // 2. Prepare for upload (convert data URL to blob)
        const response = await fetch(compressedImage);
        const blob = await response.blob();

        const formData = new FormData();
        formData.append('image', blob, 'frame.jpg');

        // 3. Call Backend OCR API
        const apiUrl = (import.meta.env.VITE_API_URL || '').replace(/\/api\/?$/, '');
        const apiResponse = await fetch(`${apiUrl}/api/v1/scan/plate`, {
          method: 'POST',
          body: formData,
          signal: controller.signal,
        });

        if (!apiResponse.ok) throw new Error('OCR API failed');

        const json = await apiResponse.json();

        if (json.success && json.data?.plate) {
          const result: DetectionResult = {
            plateNumber: json.data.plate,
            confidence: json.data.confidence * 100, // Assuming internal scale is 0-1
          };

          // Add to detections list for visual feedback (even if not stable yet)
          // Add to detections list for visual feedback (even if not stable yet)
          // ONLY add if there is actual text
          if (result.plateNumber.trim() !== '') {
            setLiveDetections((prev) => {
              if (prev.some((d) => d.plateNumber === result.plateNumber)) return prev;
              const status: PlateStatus = json.data.format_valid ? 'valid' : 'warning';
              return [
                {
                  plateNumber: result.plateNumber,
                  confidence: result.confidence,
                  status,
                },
                ...prev,
              ].slice(0, 10);
            });
            // 4. Update Stability Logic
            addDetection(result);
          }
        }
      } catch (error) {
        if (error instanceof DOMException && error.name === 'AbortError') {
          return;
        }
        console.error('Frame processing failed:', error);
        setScanError(error instanceof Error ? error.message : t('mobileScan.ocrError'));
      } finally {
        setIsLiveProcessing(false);
      }
    },
    [addDetection, t]
  );

  const isProcessingRef = useRef(false);
  const scanActiveRef = useRef(false);

  const startLiveScan = useCallback(
    (getScreenshot: () => string | null) => {
      if (scanActiveRef.current) return;

      scanActiveRef.current = true;
      lastHandledStablePlateRef.current = null;
      setLiveScanActive(true);
      resetStability();
      setScanError(null);

      const runScanLoop = async () => {
        if (!scanActiveRef.current) return;

        const imageSrc = getScreenshot();
        if (imageSrc && !isProcessingRef.current) {
          isProcessingRef.current = true;
          try {
            await processFrame(imageSrc);
          } finally {
            isProcessingRef.current = false;
          }
        }

        if (scanActiveRef.current) {
          scanIntervalRef.current = setTimeout(runScanLoop, 100);
        }
      };

      runScanLoop();
    },
    [processFrame, resetStability]
  );

  const stopLiveScan = useCallback(() => {
    scanActiveRef.current = false;
    setLiveScanActive(false);
    setIsLiveProcessing(false);
    lastHandledStablePlateRef.current = null;
    resetStability();
    setLiveDetections([]);
    setScanError(null);

    if (activeRequestRef.current) {
      activeRequestRef.current.abort();
      activeRequestRef.current = null;
    }

    if (scanIntervalRef.current) {
      clearTimeout(scanIntervalRef.current);
      scanIntervalRef.current = null;
    }
  }, [resetStability]);

  useEffect(() => {
    return () => {
      scanActiveRef.current = false;
      setIsLiveProcessing(false);
      if (activeRequestRef.current) {
        activeRequestRef.current.abort();
        activeRequestRef.current = null;
      }
      if (scanIntervalRef.current) {
        clearTimeout(scanIntervalRef.current);
      }
    };
  }, []);

  return {
    isScanning,
    setIsScanning,
    isLiveProcessing,
    liveScanActive,
    liveDetections,
    startLiveScan,
    stopLiveScan,
    setLiveDetections,
    scanError,
  };
}
