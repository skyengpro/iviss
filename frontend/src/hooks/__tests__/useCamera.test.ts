import { describe, it, expect } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useCamera } from '../feature/useCamera';

describe('useCamera', () => {
  it('should default to environment facing mode', () => {
    const { result } = renderHook(() => useCamera());

    expect(result.current.facingMode).toBe('environment');
    expect(result.current.isCameraReady).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it('should accept custom initial facing mode', () => {
    const { result } = renderHook(() => useCamera({ initialFacingMode: 'user' }));

    expect(result.current.facingMode).toBe('user');
  });

  it('should toggle facing mode between user and environment', () => {
    const { result } = renderHook(() => useCamera());

    expect(result.current.facingMode).toBe('environment');

    act(() => {
      result.current.toggleFacingMode();
    });

    expect(result.current.facingMode).toBe('user');

    act(() => {
      result.current.toggleFacingMode();
    });

    expect(result.current.facingMode).toBe('environment');
  });

  it('should set camera ready state via handleUserMedia', () => {
    const { result } = renderHook(() => useCamera());

    expect(result.current.isCameraReady).toBe(false);

    act(() => {
      result.current.handleUserMedia();
    });

    expect(result.current.isCameraReady).toBe(true);
    expect(result.current.error).toBeNull();
  });

  it('should set error state via handleUserMediaError', () => {
    const { result } = renderHook(() => useCamera());

    act(() => {
      result.current.handleUserMediaError('Camera not available');
    });

    expect(result.current.error).toBe('Camera not available');
    expect(result.current.isCameraReady).toBe(false);
  });

  it('should handle DOMException errors', () => {
    const { result } = renderHook(() => useCamera());

    act(() => {
      result.current.handleUserMediaError(new DOMException('NotAllowedError'));
    });

    expect(result.current.error).toBe('NotAllowedError');
    expect(result.current.isCameraReady).toBe(false);
  });

  it('should return null from getScreenshot when webcam ref is null', () => {
    const { result } = renderHook(() => useCamera());

    const screenshot = result.current.getScreenshot();
    expect(screenshot).toBeNull();
  });
});
