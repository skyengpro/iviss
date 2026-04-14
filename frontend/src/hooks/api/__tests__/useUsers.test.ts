import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useUsers, useUser } from '../useUsers';
import { createQueryWrapper } from '@/test/queryWrapper';

// Mock dependencies
vi.mock('@/openapi-rq/queries/queries', () => ({
  useListUsers: vi.fn(),
  useListOrgUsers: vi.fn(),
  useProvisionUser: vi.fn(),
  useProvisionOrgUser: vi.fn(),
  useUpdateUser: vi.fn(),
  useDeleteUser: vi.fn(),
  useGetUser: vi.fn(),
}));

import {
  useListUsers,
  useProvisionUser,
  useProvisionOrgUser,
  useUpdateUser,
  useDeleteUser,
  useGetUser,
} from '@/openapi-rq/queries/queries';

vi.mock('@tanstack/react-query', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@tanstack/react-query')>();
  return {
    ...actual,
    useQueryClient: vi.fn(),
  };
});
import { useQueryClient } from '@tanstack/react-query';

describe('useUsers', () => {
  let mockInvalidateQueries: any;
  let mockProvisionMutate: any;
  let mockUpdateMutate: any;
  let mockDeleteMutate: any;

  beforeEach(() => {
    vi.clearAllMocks();

    mockInvalidateQueries = vi.fn();
    vi.mocked(useQueryClient).mockReturnValue({
      invalidateQueries: mockInvalidateQueries,
    } as any);

    mockProvisionMutate = vi.fn();
    vi.mocked(useProvisionUser).mockImplementation(
      (_q, options: any) =>
        ({
          mutateAsync: (...args: any[]) => {
            if (options?.onSuccess) options.onSuccess();
            return mockProvisionMutate(...args);
          },
          isPending: false,
          error: null,
        }) as any
    );

    vi.mocked(useProvisionOrgUser).mockImplementation(
      (_q, options: any) =>
        ({
          mutateAsync: (...args: any[]) => {
            if (options?.onSuccess) options.onSuccess();
            return vi.fn()(...args);
          },
          isPending: false,
          error: null,
        }) as any
    );

    mockUpdateMutate = vi.fn();
    vi.mocked(useUpdateUser).mockImplementation(
      (_q, options: any) =>
        ({
          mutateAsync: (...args: any[]) => {
            if (options?.onSuccess) options.onSuccess();
            return mockUpdateMutate(...args);
          },
          isPending: false,
          error: null,
        }) as any
    );

    mockDeleteMutate = vi.fn();
    vi.mocked(useDeleteUser).mockImplementation(
      (_q, options: any) =>
        ({
          mutateAsync: (...args: any[]) => {
            if (options?.onSuccess) options.onSuccess();
            return mockDeleteMutate(...args);
          },
          isPending: false,
          error: null,
        }) as any
    );

    vi.mocked(useListUsers).mockReturnValue({
      data: {
        data: {
          items: [{ id: '1', email: 'test@example.com' }],
          total: 1,
        },
      },
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as any);
  });

  it('returns users and loading state from useListUsers', () => {
    const { Wrapper } = createQueryWrapper();
    const { result } = renderHook(() => useUsers(), { wrapper: Wrapper });

    expect(useListUsers).toHaveBeenCalled();
    expect(result.current.users).toEqual({
      data: { items: [{ id: '1', email: 'test@example.com' }], total: 1 },
    });
    expect(result.current.isLoadingUsers).toBe(false);
  });

  it('provision() calls mutateAsync with correct body and invalidates queries', async () => {
    const { Wrapper } = createQueryWrapper();
    const { result } = renderHook(() => useUsers(), { wrapper: Wrapper });

    mockProvisionMutate.mockResolvedValueOnce({});

    await act(async () => {
      await result.current.provision({
        email: 'new@admin.com',
        role: 'admin',
        fullName: 'Bob',
        organizationId: 'o-1',
        phoneNumber: '123',
        username: 'bob',
      } as any);
    });

    expect(mockProvisionMutate).toHaveBeenCalledWith({
      body: {
        email: 'new@admin.com',
        role: 'admin',
        fullName: 'Bob',
        organizationId: 'o-1',
        phoneNumber: '123',
        username: 'bob',
      },
      throwOnError: true,
    });
    expect(mockInvalidateQueries).toHaveBeenCalledWith({ queryKey: ['ListUsers'] });
  });

  it('update() calls mutateAsync with correct path + body and invalidates queries', async () => {
    const { Wrapper } = createQueryWrapper();
    const { result } = renderHook(() => useUsers(), { wrapper: Wrapper });

    mockUpdateMutate.mockResolvedValueOnce({});

    await act(async () => {
      await result.current.update('u-123', { role: 'agent' } as any);
    });

    expect(mockUpdateMutate).toHaveBeenCalledWith({
      path: { id: 'u-123' },
      body: { role: 'agent' },
      throwOnError: true,
    });
    expect(mockInvalidateQueries).toHaveBeenCalledWith({ queryKey: ['ListUsers'] });
  });

  it('remove() calls mutateAsync with correct path and invalidates queries', async () => {
    const { Wrapper } = createQueryWrapper();
    const { result } = renderHook(() => useUsers(), { wrapper: Wrapper });

    mockDeleteMutate.mockResolvedValueOnce({});

    await act(async () => {
      await result.current.remove('u-456');
    });

    expect(mockDeleteMutate).toHaveBeenCalledWith({
      path: { id: 'u-456' },
      throwOnError: true,
    });
    expect(mockInvalidateQueries).toHaveBeenCalledWith({ queryKey: ['ListUsers'] });
  });
});

describe('useUser', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('delegates to useGetUser with the right path param', () => {
    vi.mocked(useGetUser).mockReturnValue({
      data: { data: { id: 'u-789', email: 'bob@example.com' } },
      isLoading: true,
      error: null,
    } as any);

    const { Wrapper } = createQueryWrapper();
    const { result } = renderHook(() => useUser('u-789'), { wrapper: Wrapper });

    expect(useGetUser).toHaveBeenCalledWith(
      expect.objectContaining({
        path: { id: 'u-789' },
      })
    );
    expect(result.current.data).toEqual({ data: { id: 'u-789', email: 'bob@example.com' } });
    expect(result.current.isLoading).toBe(true);
  });
});
