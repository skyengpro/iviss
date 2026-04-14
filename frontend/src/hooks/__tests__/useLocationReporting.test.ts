import { renderHook } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useLocationReporting } from '../useLocationReporting';
import { useGeolocation } from '../useGeolocation';
import { useAuth } from '@/hooks/auth/use-auth';

vi.mock('../useGeolocation', () => ({
  useGeolocation: vi.fn(),
}));

vi.mock('@/hooks/auth/use-auth', () => ({
  useAuth: vi.fn(),
}));

vi.mock('../../openapi-rq/queries/queries', () => ({
  useUpdateLocation: vi.fn(),
}));

import { useUpdateLocation } from '../../openapi-rq/queries/queries';

describe('useLocationReporting', () => {
  let mockMutate: any;

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();

    vi.mocked(useAuth).mockReturnValue({
      user: { id: 'agent-1', role: 'agent' },
      isAuthenticated: true,
    } as any);

    mockMutate = vi.fn();
    vi.mocked(useUpdateLocation).mockReturnValue({
      mutate: (args: any, options: any) => {
        mockMutate(args);
        if (options?.onSuccess) options.onSuccess();
      },
    } as any);

    vi.mocked(useGeolocation).mockReturnValue({
      lat: 4.0,
      lng: 9.0,
      error: null,
    } as any);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('does not call updateLocation when user is null', () => {
    vi.mocked(useAuth).mockReturnValue({
      user: null,
      isAuthenticated: false,
    } as any);

    renderHook(() => useLocationReporting());

    expect(mockMutate).not.toHaveBeenCalled();
  });

  it('does not call updateLocation when user role is not agent', () => {
    vi.mocked(useAuth).mockReturnValue({
      user: { id: 'admin-1', role: 'admin' },
      isAuthenticated: true,
    } as any);

    renderHook(() => useLocationReporting());

    expect(mockMutate).not.toHaveBeenCalled();
  });

  it('does not call updateLocation when coordinates are null', () => {
    vi.mocked(useGeolocation).mockReturnValue({
      lat: null,
      lng: null,
      error: null,
    } as any);

    renderHook(() => useLocationReporting());

    expect(mockMutate).not.toHaveBeenCalled();
  });

  it('calls updateLocation with correct payload on first valid location', () => {
    renderHook(() => useLocationReporting());

    expect(mockMutate).toHaveBeenCalledTimes(1);
    expect(mockMutate).toHaveBeenCalledWith({
      requestBody: { latitude: 4.0, longitude: 9.0 },
    });
  });

  it('skips report when movement < threshold and interval not elapsed', () => {
    const { rerender } = renderHook(() => useLocationReporting());

    expect(mockMutate).toHaveBeenCalledTimes(1);
    mockMutate.mockClear();

    // Small movement
    vi.mocked(useGeolocation).mockReturnValue({
      lat: 4.00005,
      lng: 9.0,
      error: null,
    } as any);

    rerender();

    expect(mockMutate).not.toHaveBeenCalled();
  });

  it('reports again when movement > threshold (0.0001)', () => {
    const { rerender } = renderHook(() => useLocationReporting());

    expect(mockMutate).toHaveBeenCalledTimes(1);
    mockMutate.mockClear();

    // Large movement
    vi.mocked(useGeolocation).mockReturnValue({
      lat: 4.0002,
      lng: 9.0,
      error: null,
    } as any);

    rerender();

    expect(mockMutate).toHaveBeenCalledTimes(1);
    expect(mockMutate).toHaveBeenCalledWith({
      requestBody: { latitude: 4.0002, longitude: 9.0 },
    });
  });

  it('reports again when interval has elapsed, even without large movement', () => {
    const { rerender } = renderHook(() => useLocationReporting());

    expect(mockMutate).toHaveBeenCalledTimes(1);
    mockMutate.mockClear();

    // Advance time by 5 minutes + 1 ms
    vi.advanceTimersByTime(5 * 60 * 1000 + 1);

    // Small movement keeps us below the distance threshold while still retriggering the effect
    vi.mocked(useGeolocation).mockReturnValue({
      lat: 4.00001,
      lng: 9.0,
      error: null,
    } as any);

    rerender();

    expect(mockMutate).toHaveBeenCalledTimes(1);
  });
});
