import { describe, it, expect } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useStabilityDetection } from '../feature/useStabilityDetection';

describe('useStabilityDetection', () => {
  it('should start with empty history and no stable result', () => {
    const { result } = renderHook(() => useStabilityDetection());

    expect(result.current.history).toEqual([]);
    expect(result.current.stableResult).toBeNull();
  });

  it('should return null when addDetection receives null', () => {
    const { result } = renderHook(() => useStabilityDetection());

    let returnValue: ReturnType<typeof result.current.addDetection>;
    act(() => {
      returnValue = result.current.addDetection(null);
    });

    expect(returnValue!).toBeNull();
    expect(result.current.stableResult).toBeNull();
  });

  it('should reset on low-confidence input', () => {
    const { result } = renderHook(() => useStabilityDetection({ minConfidence: 75 }));

    // Add a valid detection first
    act(() => {
      result.current.addDetection({ plateNumber: 'CE 128 BC', confidence: 90 });
    });

    // Low confidence should reset
    act(() => {
      result.current.addDetection({ plateNumber: 'CE 128 BC', confidence: 50 });
    });

    expect(result.current.history).toEqual([]);
    expect(result.current.stableResult).toBeNull();
  });

  it('should reset when plate number changes', () => {
    const { result } = renderHook(() => useStabilityDetection({ requiredMatches: 3 }));

    act(() => {
      result.current.addDetection({ plateNumber: 'CE 128 BC', confidence: 90 });
    });
    act(() => {
      result.current.addDetection({ plateNumber: 'CE 128 BC', confidence: 92 });
    });

    // Different plate — should restart
    act(() => {
      result.current.addDetection({ plateNumber: 'LT 390 HN', confidence: 88 });
    });

    // History should only contain the new plate
    expect(result.current.history).toHaveLength(1);
    expect(result.current.history[0].plateNumber).toBe('LT 390 HN');
    expect(result.current.stableResult).toBeNull();
  });

  it('should set stable result after requiredMatches consecutive identical detections', () => {
    const { result } = renderHook(() =>
      useStabilityDetection({ requiredMatches: 3, minConfidence: 75 })
    );

    act(() => {
      result.current.addDetection({ plateNumber: 'CE 128 BC', confidence: 90 });
    });
    act(() => {
      result.current.addDetection({ plateNumber: 'CE 128 BC', confidence: 92 });
    });
    act(() => {
      result.current.addDetection({ plateNumber: 'CE 128 BC', confidence: 88 });
    });

    expect(result.current.stableResult).not.toBeNull();
    expect(result.current.stableResult!.plateNumber).toBe('CE 128 BC');
  });

  it('should average confidence scores in stable result', () => {
    const { result } = renderHook(() =>
      useStabilityDetection({ requiredMatches: 3, minConfidence: 75 })
    );

    act(() => {
      result.current.addDetection({ plateNumber: 'CE 128 BC', confidence: 90 });
    });
    act(() => {
      result.current.addDetection({ plateNumber: 'CE 128 BC', confidence: 93 });
    });
    act(() => {
      result.current.addDetection({ plateNumber: 'CE 128 BC', confidence: 88 });
    });

    const expectedAvg = (90 + 93 + 88) / 3;
    expect(result.current.stableResult!.confidence).toBeCloseTo(expectedAvg);
  });

  it('should clear history and stableResult on resetStability', () => {
    const { result } = renderHook(() =>
      useStabilityDetection({ requiredMatches: 2, minConfidence: 50 })
    );

    act(() => {
      result.current.addDetection({ plateNumber: 'CE 128 BC', confidence: 90 });
    });
    act(() => {
      result.current.addDetection({ plateNumber: 'CE 128 BC', confidence: 88 });
    });

    expect(result.current.stableResult).not.toBeNull();

    act(() => {
      result.current.resetStability();
    });

    expect(result.current.history).toEqual([]);
    expect(result.current.stableResult).toBeNull();
  });

  it('should respect custom requiredMatches', () => {
    const { result } = renderHook(() =>
      useStabilityDetection({ requiredMatches: 2, minConfidence: 50 })
    );

    act(() => {
      result.current.addDetection({ plateNumber: 'AB 123 CD', confidence: 80 });
    });

    // Not stable yet with only 1 detection
    expect(result.current.stableResult).toBeNull();

    act(() => {
      result.current.addDetection({ plateNumber: 'AB 123 CD', confidence: 85 });
    });

    // Should be stable after 2 matches
    expect(result.current.stableResult).not.toBeNull();
    expect(result.current.stableResult!.plateNumber).toBe('AB 123 CD');
  });
});
