import { useState, useCallback } from 'react';

export interface DetectionResult {
  plateNumber: string;
  confidence: number;
}

interface UseStabilityDetectionProps {
  requiredMatches?: number;
  minConfidence?: number;
  windowSize?: number;
}

/**
 * Hook to manage the stability detection logic for license plate scanning.
 * Confirms a plate once `requiredMatches` of the last `windowSize` readings
 * agree — a sliding majority vote, not a consecutive-match streak, so a
 * single misread interleaved in an otherwise-agreeing stream doesn't wipe
 * out the count already accumulated by the others.
 */
export function useStabilityDetection({
  requiredMatches = 3,
  minConfidence = 0,
  windowSize = 5,
}: UseStabilityDetectionProps = {}) {
  const [history, setHistory] = useState<DetectionResult[]>([]);
  const [stableResult, setStableResult] = useState<{
    plateNumber: string;
    confidence: number;
  } | null>(null);

  /**
   * Adds a new detection result and checks if stability is reached.
   * Confidence is a pre-filter here, not the stability signal itself —
   * agreement across independent frames is.
   * @param result Output from the OCR backend
   */
  const addDetection = useCallback(
    (result: DetectionResult | null): void => {
      if (!result || !result.plateNumber || result.confidence < minConfidence) {
        return;
      }

      setHistory((prev) => {
        const newHistory = [...prev, result].slice(-windowSize);

        const agreeing = newHistory.filter((item) => item.plateNumber === result.plateNumber);

        if (agreeing.length >= requiredMatches) {
          setStableResult({
            plateNumber: result.plateNumber,
            confidence: Math.max(...agreeing.map((item) => item.confidence)),
          });
        } else {
          setStableResult(null);
        }

        return newHistory;
      });
    },
    [requiredMatches, minConfidence, windowSize]
  );

  const resetStability = useCallback(() => {
    setHistory([]);
    setStableResult(null);
  }, []);

  return {
    history,
    stableResult,
    addDetection,
    resetStability,
  };
}
