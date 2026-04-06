import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useGeolocation } from '../useGeolocation';

describe('useGeolocation', () => {
  let mockGetCurrentPosition: ReturnType<typeof vi.fn>;
  let mockWatchPosition: ReturnType<typeof vi.fn>;
  let mockClearWatch: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    mockGetCurrentPosition = vi.fn();
    mockWatchPosition = vi.fn().mockReturnValue(42); // watchId
    mockClearWatch = vi.fn();

    Object.defineProperty(navigator, 'geolocation', {
      value: {
        getCurrentPosition: mockGetCurrentPosition,
        watchPosition: mockWatchPosition,
        clearWatch: mockClearWatch,
      },
      writable: true,
      configurable: true,
    });
  });

  it('should start in loading state', () => {
    const { result } = renderHook(() => useGeolocation());

    expect(result.current.loading).toBe(true);
    expect(result.current.lat).toBeNull();
    expect(result.current.lng).toBeNull();
    expect(result.current.error).toBeNull();
  });

  it('should return coordinates on successful geolocation', () => {
    mockGetCurrentPosition.mockImplementation((success: PositionCallback) => {
      success({
        coords: {
          latitude: 48.8566,
          longitude: 2.3522,
          accuracy: 10,
        },
      } as GeolocationPosition);
    });

    const { result } = renderHook(() => useGeolocation());

    expect(result.current.lat).toBe(48.8566);
    expect(result.current.lng).toBe(2.3522);
    expect(result.current.accuracy).toBe(10);
    expect(result.current.loading).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it('should return error on permission denied', () => {
    mockGetCurrentPosition.mockImplementation(
      (_success: PositionCallback, error: PositionErrorCallback) => {
        error({
          code: 1, // PERMISSION_DENIED
          PERMISSION_DENIED: 1,
          POSITION_UNAVAILABLE: 2,
          TIMEOUT: 3,
          message: 'User denied',
        } as GeolocationPositionError);
      }
    );

    const { result } = renderHook(() => useGeolocation());

    expect(result.current.error).toBe('User denied the request for Geolocation');
    expect(result.current.loading).toBe(false);
  });

  it('should return error when position is unavailable', () => {
    mockGetCurrentPosition.mockImplementation(
      (_success: PositionCallback, error: PositionErrorCallback) => {
        error({
          code: 2, // POSITION_UNAVAILABLE
          PERMISSION_DENIED: 1,
          POSITION_UNAVAILABLE: 2,
          TIMEOUT: 3,
          message: 'Position unavailable',
        } as GeolocationPositionError);
      }
    );

    const { result } = renderHook(() => useGeolocation());

    expect(result.current.error).toBe('Location information is unavailable');
    expect(result.current.loading).toBe(false);
  });

  it('should return error on timeout', () => {
    mockGetCurrentPosition.mockImplementation(
      (_success: PositionCallback, error: PositionErrorCallback) => {
        error({
          code: 3, // TIMEOUT
          PERMISSION_DENIED: 1,
          POSITION_UNAVAILABLE: 2,
          TIMEOUT: 3,
          message: 'Timeout',
        } as GeolocationPositionError);
      }
    );

    const { result } = renderHook(() => useGeolocation());

    expect(result.current.error).toBe('The request to get user location timed out');
    expect(result.current.loading).toBe(false);
  });

  it('should return error when geolocation is not supported', () => {
    Object.defineProperty(navigator, 'geolocation', {
      value: undefined,
      writable: true,
      configurable: true,
    });

    const { result } = renderHook(() => useGeolocation());

    expect(result.current.error).toBe('Geolocation not supported');
    expect(result.current.loading).toBe(false);
  });

  it('should clean up watch on unmount', () => {
    const { unmount } = renderHook(() => useGeolocation());
    unmount();

    expect(mockClearWatch).toHaveBeenCalledWith(42);
  });
});
