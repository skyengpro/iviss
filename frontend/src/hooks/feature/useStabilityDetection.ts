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
      // If no result or low confidence, just skip this frame (don't reset history)
      // This allows matches to accumulate even if intermittent frames are empty
      if (!result || !result.plateNumber || result.confidence < minConfidence) {
        return null;
      }

      setHistory((prev) => {
        const newHistory = [...prev, result].slice(-requiredMatches);

        // Check if we have enough matches
        if (newHistory.length < requiredMatches) {
          return newHistory;
        }

        // Check if all items in history are identical
        const firstPlate = newHistory[0].plateNumber;
        const allMatch = newHistory.every((item) => item.plateNumber === firstPlate);

        if (allMatch) {
          // Calculate average confidence
          const avgConfidence =
            newHistory.reduce((sum, item) => sum + item.confidence, 0) / newHistory.length;
          setStableResult({
            plateNumber: firstPlate,
            confidence: avgConfidence,
          });
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