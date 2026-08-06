import { describe, it, expect } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useStabilityDetection } from '../feature/useStabilityDetection';

describe('useStabilityDetection', () => {
  it('should start with empty history and no stable result', () => {
    const { result } = renderHook(() => useStabilityDetection());

    expect(result.current.history).toEqual([]);
    expect(result.current.stableResult).toBeNull();
  });

  it('should ignore a null detection without touching history or stableResult', () => {
    const { result } = renderHook(() => useStabilityDetection());

    act(() => {
      result.current.addDetection(null);
    });

    expect(result.current.history).toEqual([]);
    expect(result.current.stableResult).toBeNull();
  });

  it('defaults minConfidence to 0, so a zero-confidence reading is not filtered out', () => {
    const { result } = renderHook(() => useStabilityDetection());

    act(() => {
      result.current.addDetection({ plateNumber: 'CE568LR', confidence: 0 });
    });

    expect(result.current.history).toHaveLength(1);
  });

  it('ignores readings below minConfidence without resetting accumulated history', () => {
    const { result } = renderHook(() =>
      useStabilityDetection({ requiredMatches: 3, minConfidence: 50 })
    );

    act(() => {
      result.current.addDetection({ plateNumber: 'CE128BC', confidence: 80 });
    });
    act(() => {
      result.current.addDetection({ plateNumber: 'CE128BC', confidence: 90 });
    });
    // A low-confidence reading is a pre-filter, not a decision — it must be
    // dropped silently, not wipe out the two agreeing readings above.
    act(() => {
      result.current.addDetection({ plateNumber: 'CE128BC', confidence: 10 });
    });

    expect(result.current.history).toHaveLength(2);
    expect(result.current.stableResult).toBeNull();

    act(() => {
      result.current.addDetection({ plateNumber: 'CE128BC', confidence: 85 });
    });

    expect(result.current.stableResult).toEqual({ plateNumber: 'CE128BC', confidence: 90 });
  });

  it('tolerates a misread interleaved between agreeing readings', () => {
    // CE568LR (16), CE568LB (0), CE568LR (42), CE568LR (4)
    // With the old consecutive-match counter, the misread in the middle
    // reset the counter to zero and this case never resolved.
    const { result } = renderHook(() => useStabilityDetection());

    act(() => {
      result.current.addDetection({ plateNumber: 'CE568LR', confidence: 16 });
    });
    expect(result.current.stableResult).toBeNull();

    act(() => {
      result.current.addDetection({ plateNumber: 'CE568LB', confidence: 0 });
    });
    expect(result.current.stableResult).toBeNull();

    act(() => {
      result.current.addDetection({ plateNumber: 'CE568LR', confidence: 42 });
    });
    expect(result.current.stableResult).toBeNull();

    act(() => {
      result.current.addDetection({ plateNumber: 'CE568LR', confidence: 4 });
    });

    expect(result.current.stableResult).not.toBeNull();
    expect(result.current.stableResult!.plateNumber).toBe('CE568LR');
  });

  it('reports the strongest agreeing reading, not the average', () => {
    const { result } = renderHook(() => useStabilityDetection());

    act(() => {
      result.current.addDetection({ plateNumber: 'CE568LR', confidence: 0 });
    });
    act(() => {
      result.current.addDetection({ plateNumber: 'CE568LR', confidence: 63 });
    });
    act(() => {
      result.current.addDetection({ plateNumber: 'CE568LR', confidence: 0 });
    });

    expect(result.current.stableResult).toEqual({ plateNumber: 'CE568LR', confidence: 63 });
  });

  it('does not clear an already-stable plate when a single different plate is interleaved', () => {
    const { result } = renderHook(() => useStabilityDetection({ requiredMatches: 2 }));

    act(() => {
      result.current.addDetection({ plateNumber: 'CE128BC', confidence: 80 });
    });
    act(() => {
      result.current.addDetection({ plateNumber: 'CE128BC', confidence: 85 });
    });
    expect(result.current.stableResult?.plateNumber).toBe('CE128BC');

    // One reading of an unrelated plate must not undo a still-qualifying majority.
    act(() => {
      result.current.addDetection({ plateNumber: 'LT390HN', confidence: 90 });
    });

    expect(result.current.stableResult?.plateNumber).toBe('CE128BC');
  });

  it('evicts the oldest reading once the sliding window is full', () => {
    const { result } = renderHook(() =>
      useStabilityDetection({ requiredMatches: 2, windowSize: 3 })
    );

    act(() => {
      result.current.addDetection({ plateNumber: 'AAA0000AA', confidence: 80 });
    });
    act(() => {
      result.current.addDetection({ plateNumber: 'AAA0000AA', confidence: 80 });
    });
    act(() => {
      result.current.addDetection({ plateNumber: 'BBB0000BB', confidence: 80 });
    });
    // Window is now [A, A, B] — A is still the majority (2/2 still present).
    expect(result.current.stableResult?.plateNumber).toBe('AAA0000AA');

    act(() => {
      result.current.addDetection({ plateNumber: 'BBB0000BB', confidence: 80 });
    });
    // Window slides to [A, B, B] — the first A falls out, B now has 2/3.
    expect(result.current.stableResult?.plateNumber).toBe('BBB0000BB');
    expect(result.current.history).toHaveLength(3);
  });

  it('should clear history and stableResult on resetStability', () => {
    const { result } = renderHook(() => useStabilityDetection({ requiredMatches: 2 }));

    act(() => {
      result.current.addDetection({ plateNumber: 'CE128BC', confidence: 90 });
    });
    act(() => {
      result.current.addDetection({ plateNumber: 'CE128BC', confidence: 88 });
    });

    expect(result.current.stableResult).not.toBeNull();

    act(() => {
      result.current.resetStability();
    });

    expect(result.current.history).toEqual([]);
    expect(result.current.stableResult).toBeNull();
  });

  it('should respect a custom requiredMatches', () => {
    const { result } = renderHook(() => useStabilityDetection({ requiredMatches: 2 }));

    act(() => {
      result.current.addDetection({ plateNumber: 'AB123CD', confidence: 80 });
    });
    expect(result.current.stableResult).toBeNull();

    act(() => {
      result.current.addDetection({ plateNumber: 'AB123CD', confidence: 85 });
    });

    expect(result.current.stableResult).not.toBeNull();
    expect(result.current.stableResult!.plateNumber).toBe('AB123CD');
  });
});
