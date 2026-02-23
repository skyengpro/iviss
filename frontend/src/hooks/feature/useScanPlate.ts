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
  initialUseDemoData?: boolean;
}

/**
 * Hook to manage the license plate scanning process (The "Eyes" of the system).
 * Implements 500ms frame sampling and hybrid OCR communication.
 */
export function useScanPlate({ onSuccess, initialUseDemoData = false }: UseScanPlateProps = {}) {
  const { t } = useTranslation();
  const [isScanning, setIsScanning] = useState(false);
  const [useDemoData, setUseDemoData] = useState(initialUseDemoData);
  const [liveScanActive, setLiveScanActive] = useState(false);
  const [liveDetections, setLiveDetections] = useState<DetectedPlate[]>([]);
  const [scanError, setScanError] = useState<string | null>(null);

  const { addDetection, resetStability, stableResult } = useStabilityDetection({
    requiredMatches: 3,
    minConfidence: 75,
  });

  const scanIntervalRef = useRef<NodeJS.Timeout | null>(null);

  // Effect to handle success when a stable result is found
  useEffect(() => {
    if (stableResult) {
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
      if (scanIntervalRef.current) {
        clearInterval(scanIntervalRef.current);
        scanIntervalRef.current = null;
      }
    }
  }, [stableResult, onSuccess]);

  const stopLiveScan = useCallback(() => {
    setLiveScanActive(false);
    resetStability();
    setLiveDetections([]); // Clear visual history on stop
    setScanError(null);

    if (scanIntervalRef.current) {
      clearInterval(scanIntervalRef.current);
      scanIntervalRef.current = null;
    }
  }, [resetStability]);

  const demoStateRef = useRef({ count: 0, currentPlate: 'CE 128 BC' });

  const processFrame = useCallback(
    async (imageSrc: string) => {
      if (useDemoData) {
        // Logic for demo mode (simulated backend)
        // To test stability logic, we return the same plate for 5 frames, then pick a new one
        const mockPlates = ['CE 128 BC', 'LT 390 HN', 'EN 555 AA'];

        if (demoStateRef.current.count >= 5) {
          demoStateRef.current.currentPlate =
            mockPlates[Math.floor(Math.random() * mockPlates.length)];
          demoStateRef.current.count = 0;
        }

        demoStateRef.current.count++;
        const currentPlate = demoStateRef.current.currentPlate;

        // Simulate network delay
        await new Promise((r) => setTimeout(r, 200));

        const result: DetectionResult = {
          plateNumber: currentPlate,
          confidence: 85 + Math.random() * 10,
        };

        // Add to detections list for visual feedback
        setLiveDetections((prev) => {
          if (prev.some((d) => d.plateNumber === result.plateNumber)) return prev;
          return [
            {
              plateNumber: result.plateNumber,
              confidence: result.confidence,
              status: 'valid' as PlateStatus,
            },
            ...prev,
          ].slice(0, 10);
        });

        addDetection(result);
        return;
      }

      try {
        // 1. Optimize image (800x600, 70% JPEG ~50KB)
        const compressedImage = await ImageProcessor.preprocessForOCR(imageSrc, t);

        // 2. Prepare for upload (convert data URL to blob)
        const response = await fetch(compressedImage);
        const blob = await response.blob();

        const formData = new FormData();
        formData.append('image', blob, 'frame.jpg');

        // 3. Call Backend OCR API
        const apiResponse = await fetch('/api/v1/scan/plate', {
          method: 'POST',
          body: formData,
        });

        if (!apiResponse.ok) throw new Error('OCR API failed');

        const json = await apiResponse.json();

        if (json.success && json.data) {
          const result: DetectionResult = {
            plateNumber: json.data.plate,
            confidence: json.data.confidence * 100, // Assuming internal scale is 0-1
          };

          // Add to detections list for visual feedback (even if not stable yet)
          if (result.confidence > 60) {
            setLiveDetections((prev) => {
              if (prev.some((d) => d.plateNumber === result.plateNumber)) return prev;
              return [
                {
                  plateNumber: result.plateNumber,
                  confidence: result.confidence,
                  status: 'valid' as PlateStatus,
                },
                ...prev,
              ].slice(0, 10);
            });
          }

          // 4. Update Stability Logic
          addDetection(result);
        }
      } catch (error) {
        console.error('Frame processing failed:', error);
        setScanError(error instanceof Error ? error.message : t('mobileScan.ocrError'));
      }

    },
    [useDemoData, addDetection, t]
  );

  const startLiveScan = useCallback(
    (getScreenshot: () => string | null) => {
      if (liveScanActive) return;

      setLiveScanActive(true);
      resetStability();
      setScanError(null);
      demoStateRef.current = { count: 0, currentPlate: 'CE 128 BC' }; // Reset demo state

      // Capture first frame immediately for better responsiveness
      const firstFrame = getScreenshot();
      if (firstFrame) {
        processFrame(firstFrame);
      }

      // Implement 500ms sampling loop for subsequent frames
      scanIntervalRef.current = setInterval(async () => {
        const imageSrc = getScreenshot();
        if (imageSrc) {
          await processFrame(imageSrc);
        }
      }, 500);
    },
    [liveScanActive, processFrame, resetStability]
  );

  useEffect(() => {
    return () => {
      if (scanIntervalRef.current) {
        clearInterval(scanIntervalRef.current);
      }
    };
  }, []);

  return {
    isScanning,
    setIsScanning,
    useDemoData,
    setUseDemoData,
    liveScanActive,
    liveDetections,
    startLiveScan,
    stopLiveScan,
    setLiveDetections,
    scanError,
  };
}
