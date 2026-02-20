import { renderHook, act } from '@testing-library/react';
import { useStabilityDetection } from '@/hooks/feature/useStabilityDetection';
import { describe, it, expect } from 'vitest';

describe('useStabilityDetection', () => {
    it('should confirm result after 3 identical matches with high confidence', () => {
        const { result } = renderHook(() => useStabilityDetection({ requiredMatches: 3, minConfidence: 75 }));

        act(() => {
            result.current.addDetection({ plateNumber: 'CE128BC', confidence: 80 });
        });
        expect(result.current.stableResult).toBeNull();

        act(() => {
            result.current.addDetection({ plateNumber: 'CE128BC', confidence: 90 });
        });
        expect(result.current.stableResult).toBeNull();

        act(() => {
            result.current.addDetection({ plateNumber: 'CE128BC', confidence: 100 });
        });
        expect(result.current.stableResult).toEqual({
            plateNumber: 'CE128BC',
            confidence: 90
        });
    });

    it('should reset history on low confidence detection', () => {
        const { result } = renderHook(() => useStabilityDetection({ requiredMatches: 3, minConfidence: 75 }));

        act(() => {
            result.current.addDetection({ plateNumber: 'CE128BC', confidence: 80 });
        });
        act(() => {
            result.current.addDetection({ plateNumber: 'CE128BC', confidence: 85 });
        });

        // Low confidence should reset
        act(() => {
            result.current.addDetection({ plateNumber: 'CE128BC', confidence: 50 });
        });

        act(() => {
            result.current.addDetection({ plateNumber: 'CE128BC', confidence: 80 });
        });
        expect(result.current.stableResult).toBeNull(); // Need 2 more
    });

    it('should reset history on mismatching plate number', () => {
        const { result } = renderHook(() => useStabilityDetection({ requiredMatches: 3, minConfidence: 75 }));

        act(() => {
            result.current.addDetection({ plateNumber: 'CE128BC', confidence: 80 });
        });
        act(() => {
            result.current.addDetection({ plateNumber: 'CE128BC', confidence: 85 });
        });

        // Mismatch should reset logic (it actually just keeps the last N mismatched ones, but stability check fails)
        act(() => {
            result.current.addDetection({ plateNumber: 'LT390HN', confidence: 80 });
        });

        expect(result.current.stableResult).toBeNull();
    });
});
