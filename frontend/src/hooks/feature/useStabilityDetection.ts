import { useState, useCallback } from 'react';

export interface DetectionResult {
  plateNumber: string;
  confidence: number;
}

interface UseStabilityDetectionProps {
  requiredMatches?: number;
  minConfidence?: number;
}

/**
 * Hook to manage the stability detection logic for license plate scanning.
 * It requires 3 consecutive identical results with >75% confidence to confirm a match.
 */
export function useStabilityDetection({
  requiredMatches = 3,
  minConfidence = 75,
}: UseStabilityDetectionProps = {}) {
  const [history, setHistory] = useState<DetectionResult[]>([]);
  const [stableResult, setStableResult] = useState<{
    plateNumber: string;
    confidence: number;
  } | null>(null);

  /**
   * Adds a new detection result and checks if stability is reached.
   * @param result Output from the OCR backend
   * @returns The stable result if confirmed, otherwise null
   */
  const addDetection = useCallback(
    (result: DetectionResult | null): { plateNumber: string; confidence: number } | null => {
      // If no result or low confidence, reset stability
      if (!result || !result.plateNumber || result.confidence < minConfidence) {
        setHistory([]);
        setStableResult(null);
        return null;
      }

      setHistory((prev) => {
        // If the plate changes, restart the consecutive-match counter
        if (prev.length > 0 && prev[prev.length - 1].plateNumber !== result.plateNumber) {
          setStableResult(null);
          return [result];
        }

        const newHistory = [...prev, result].slice(-requiredMatches);

        if (newHistory.length === requiredMatches) {
          const firstPlate = newHistory[0].plateNumber;
          const allMatch = newHistory.every((item) => item.plateNumber === firstPlate);
          if (allMatch) {
            const avgConfidence =
              newHistory.reduce((sum, item) => sum + item.confidence, 0) / newHistory.length;
            setStableResult({
              plateNumber: firstPlate,
              confidence: avgConfidence,
            });
          } else {
            setStableResult(null);
          }
        } else {
          setStableResult(null);
        }

        return newHistory;
      });

      return null;
    },
    [requiredMatches, minConfidence]
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
