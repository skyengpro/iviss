import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useVehicles } from '../useVehicles';
import { createQueryWrapper } from '@/test/queryWrapper';
import { useAuth } from '@/hooks/auth/use-auth';

// Mock dependencies
vi.mock('@/openapi-rq/queries/queries', () => ({
  useSearchVehicleV1: vi.fn(),
  useSubmitVehicleV1: vi.fn(),
}));

import { useSearchVehicleV1, useSubmitVehicleV1 } from '@/openapi-rq/queries/queries';

vi.mock('@/hooks/auth/use-auth', () => ({
  useAuth: vi.fn(),
}));

// Mock geolocation
const mockGeolocation = {
  getCurrentPosition: vi.fn(),
};

describe('useVehicles', () => {
  let mockSearchMutate: any;
  let mockSubmitMutate: any;
  let originalGeolocation: any;
  let originalFetch: any;

  beforeEach(() => {
    vi.clearAllMocks();

    vi.mocked(useAuth).mockReturnValue({
      user: { id: 'agent-1', organizationId: 'org-1' },
    } as any);

    mockSearchMutate = vi.fn().mockResolvedValue({
      data: { plateNumber: 'CE123AB', status: 'valid' },
    });
    vi.mocked(useSearchVehicleV1).mockReturnValue({
      mutateAsync: mockSearchMutate,
      isPending: false,
      error: null,
    } as any);

    mockSubmitMutate = vi.fn().mockResolvedValue({
      data: { success: true },
    });
    vi.mocked(useSubmitVehicleV1).mockReturnValue({
      mutateAsync: mockSubmitMutate,
      isPending: false,
      error: null,
    } as any);

    originalGeolocation = navigator.geolocation;
    Object.defineProperty(navigator, 'geolocation', {
      value: mockGeolocation,
      writable: true,
    });

    originalFetch = globalThis.fetch;
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () =>
        Promise.resolve({
          display_name: 'Main St, Douala, Cameroon',
        }),
    });
  });

  afterEach(() => {
    Object.defineProperty(navigator, 'geolocation', {
      value: originalGeolocation,
      writable: true,
    });
    globalThis.fetch = originalFetch;
  });

  it('search() calls mutateAsync with agent/organization injected from user', async () => {
    mockGeolocation.getCurrentPosition.mockImplementationOnce((success: any) =>
      success({ coords: { latitude: 4.05, longitude: 9.7 } })
    );

    const { Wrapper } = createQueryWrapper();
    const { result } = renderHook(() => useVehicles(), { wrapper: Wrapper });

    await act(async () => {
      // Mocking request matching VehicleSearchRequest slightly
      await result.current.search({ plate_number: 'CE123AB' } as any);
    });

    expect(mockSearchMutate).toHaveBeenCalledWith({
      body: {
        plate_number: 'CE123AB',
        agent_id: 'agent-1',
        organization_id: 'org-1',
        latitude: 4.05,
        longitude: 9.7,
        address: 'Main St, Douala, Cameroon',
      },
      throwOnError: true,
    });
  });

  it('search() proceeds with undefined lat/lng/address when geolocation fails', async () => {
    mockGeolocation.getCurrentPosition.mockImplementationOnce((_, error: any) =>
      error(new Error('User denied Geolocation'))
    );

    const { Wrapper } = createQueryWrapper();
    const { result } = renderHook(() => useVehicles(), { wrapper: Wrapper });

    await act(async () => {
      await result.current.search({ plate_number: 'LT999HN' } as any);
    });

    expect(mockSearchMutate).toHaveBeenCalledWith({
      body: {
        plate_number: 'LT999HN',
        agent_id: 'agent-1',
        organization_id: 'org-1',
        latitude: undefined,
        longitude: undefined,
        address: undefined,
      },
      throwOnError: true,
    });
  });

  it('search() skips geolocation if latitude/longitude already in request', async () => {
    const { Wrapper } = createQueryWrapper();
    const { result } = renderHook(() => useVehicles(), { wrapper: Wrapper });

    await act(async () => {
      await result.current.search({
        plate_number: 'OU123',
        latitude: 3.8,
        longitude: 11.5,
      } as any);
    });

    expect(mockGeolocation.getCurrentPosition).not.toHaveBeenCalled();
    expect(mockSearchMutate).toHaveBeenCalledWith({
      body: {
        plate_number: 'OU123',
        agent_id: 'agent-1',
        organization_id: 'org-1',
        latitude: 3.8,
        longitude: 11.5,
        address: 'Main St, Douala, Cameroon',
      },
      throwOnError: true,
    });
  });

  it('submit() delegates directly to submitMutate', async () => {
    const { Wrapper } = createQueryWrapper();
    const { result } = renderHook(() => useVehicles(), { wrapper: Wrapper });

    await act(async () => {
      await result.current.submit({ status: 'valid', plate_number: 'X1' } as any);
    });

    expect(mockSubmitMutate).toHaveBeenCalledWith({
      body: { status: 'valid', plate_number: 'X1' },
      throwOnError: true,
    });
  });

  it('returns correctly extracted pending and error states', () => {
    vi.mocked(useSearchVehicleV1).mockReturnValue({
      isPending: true,
      error: new Error('Search failed'),
    } as any);

    vi.mocked(useSubmitVehicleV1).mockReturnValue({
      isPending: false,
      error: new Error('Submit failed'),
    } as any);

    const { Wrapper } = createQueryWrapper();
    const { result } = renderHook(() => useVehicles(), { wrapper: Wrapper });

    expect(result.current.isSearching).toBe(true);
    expect(result.current.searchError).toEqual(new Error('Search failed'));
    expect(result.current.isSubmitting).toBe(false);
    expect(result.current.submitError).toEqual(new Error('Submit failed'));
  });
});
