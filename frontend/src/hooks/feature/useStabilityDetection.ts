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

        const byPlate = new Map<string, DetectionResult[]>();
        for (const item of newHistory) {
          const bucket = byPlate.get(item.plateNumber);
          if (bucket) bucket.push(item);
          else byPlate.set(item.plateNumber, [item]);
        }

        let winner: DetectionResult[] | null = null;
        for (const bucket of byPlate.values()) {
          if (bucket.length >= requiredMatches && (!winner || bucket.length > winner.length)) {
            winner = bucket;
          }
        }

        setStableResult(
          winner
            ? {
                plateNumber: winner[0].plateNumber,
                confidence: Math.max(...winner.map((item) => item.confidence)),
              }
            : null
        );

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
