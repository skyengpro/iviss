import { useCallback } from 'react';
import {
  useListUsers,
  useProvisionUser,
  useUpdateUser,
  useDeleteUser,
  useGetUser,
} from '../../openapi-rq/queries/queries';
import { ProvisionUserRequest, UpdateUserRequest } from '../../openapi-rq/types.gen';
import { useQueryClient } from '@tanstack/react-query';

export function useUsers() {
  const queryClient = useQueryClient();

  const { data: users, isLoading: isLoadingUsers, error: usersError } = useListUsers();

  const {
    mutateAsync: provisionMutate,
    isPending: isProvisioning,
    error: provisionError,
  } = useProvisionUser(undefined, {
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ListUsers'] });
      queryClient.invalidateQueries({ queryKey: ['GetDashboardStats'] });
    },
  });

  const {
    mutateAsync: updateMutate,
    isPending: isUpdating,
    error: updateError,
  } = useUpdateUser(undefined, {
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ListUsers'] });
      queryClient.invalidateQueries({ queryKey: ['GetDashboardStats'] });
    },
  });

  const {
    mutateAsync: deleteMutate,
    isPending: isDeleting,
    error: deleteError,
  } = useDeleteUser(undefined, {
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ListUsers'] });
      queryClient.invalidateQueries({ queryKey: ['GetDashboardStats'] });
    },
  });

  const provision = useCallback(
    async (request: ProvisionUserRequest) => {
      return provisionMutate({
        body: request,
        throwOnError: true,
      });
    },
    [provisionMutate]
  );

  const update = useCallback(
    async (id: string, request: UpdateUserRequest) => {
      return updateMutate({
        path: { id },
        body: request,
        throwOnError: true,
      });
    },
    [updateMutate]
  );

  const remove = useCallback(
    async (id: string) => {
      return deleteMutate({
        path: { id },
        throwOnError: true,
      });
    },
    [deleteMutate]
  );

  return {
    users,
    isLoadingUsers,
    usersError,

    provision,
    isProvisioning,
    provisionError,

    update,
    isUpdating,
    updateError,

    remove,
    isDeleting,
    deleteError,
  };
}

export function useUser(id: string) {
  return useGetUser({
    path: { id },
  });
}
