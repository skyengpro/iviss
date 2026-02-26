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
    requiredMatches: 2,
    minConfidence: 60,
  });

  const scanIntervalRef = useRef<NodeJS.Timeout | null>(null);

  // Effect to handle success when a stable result is found
  useEffect(() => {
    if (stableResult?.plateNumber) {
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
        // 1. Optimize image (1200x400 crop of center)
        const compressedImage = await ImageProcessor.cropToViewfinder(imageSrc, t);

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
        console.error('Frame processing failed:', error);
        setScanError(error instanceof Error ? error.message : t('mobileScan.ocrError'));
      }

    },
    [useDemoData, addDetection, t]
  );

  const isProcessingRef = useRef(false);
  const scanActiveRef = useRef(false);

  const startLiveScan = useCallback(
    (getScreenshot: () => string | null) => {
      if (scanActiveRef.current) return;

      scanActiveRef.current = true;
      setLiveScanActive(true);
      resetStability();
      setScanError(null);
      demoStateRef.current = { count: 0, currentPlate: 'CE 128 BC' };

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
    resetStability();
    setLiveDetections([]);
    setScanError(null);

    if (scanIntervalRef.current) {
      clearTimeout(scanIntervalRef.current);
      scanIntervalRef.current = null;
    }
  }, [resetStability]);

  useEffect(() => {
    return () => {
      scanActiveRef.current = false;
      if (scanIntervalRef.current) {
        clearTimeout(scanIntervalRef.current as any);
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
