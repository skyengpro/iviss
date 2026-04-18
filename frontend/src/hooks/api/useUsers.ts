import { useCallback } from 'react';
import {
  useListUsers,
  useListOrgUsers,
  useProvisionUser,
  useProvisionOrgUser,
  useUpdateUser,
  useDeleteUser,
  useGetUser,
} from '../../openapi-rq/queries/queries';
import { ProvisionUserRequest, UpdateUserRequest } from '../../openapi-rq/requests/types.gen';
import { useQueryClient } from '@tanstack/react-query';

export function useUsers() {
  const queryClient = useQueryClient();

  const { data: users, isLoading: isLoadingUsers, error: usersError } = useListUsers();

  const {
    mutateAsync: provisionMutate,
    isPending: isProvisioning,
    error: provisionError,
<<<<<<< HEAD
  } = useProvisionUser(undefined, {
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ListUsers'] });
      queryClient.invalidateQueries({ queryKey: ['GetDashboardStats'] });
    },
  });

  const { mutateAsync: provisionOrgMutate, isPending: isProvisioningOrg } = useProvisionOrgUser(
    undefined,
    {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ['ListUsers'] });
        queryClient.invalidateQueries({ queryKey: ['ListOrgUsers'] });
        queryClient.invalidateQueries({ queryKey: ['GetDashboardStats'] });
        queryClient.invalidateQueries({ queryKey: ['GetOrgDashboardStats'] });
      },
    }
  );

  const {
    mutateAsync: updateMutate,
    isPending: isUpdating,
    error: updateError,
<<<<<<< HEAD
  } = useUpdateUser(undefined, {
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ListUsers'] });
      queryClient.invalidateQueries({ queryKey: ['ListOrgUsers'] });
      queryClient.invalidateQueries({ queryKey: ['GetDashboardStats'] });
      queryClient.invalidateQueries({ queryKey: ['GetOrgDashboardStats'] });
    },
  });

  const {
    mutateAsync: deleteMutate,
    isPending: isDeleting,
    error: deleteError,
<<<<<<< HEAD
  } = useDeleteUser(undefined, {
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ListUsers'] });
      queryClient.invalidateQueries({ queryKey: ['ListOrgUsers'] });
      queryClient.invalidateQueries({ queryKey: ['GetDashboardStats'] });
      queryClient.invalidateQueries({ queryKey: ['GetOrgDashboardStats'] });
    },
  });

  const provision = useCallback(
    async (request: ProvisionUserRequest) => {
      return provisionMutate({ body: request, throwOnError: true });
    },
    [provisionMutate]
  );

  const provisionOrg = useCallback(
    async (request: ProvisionUserRequest) => {
      return provisionOrgMutate({ body: request, throwOnError: true });
    },
    [provisionOrgMutate]
  );

  const update = useCallback(
    async (id: string, request: UpdateUserRequest) => {
      return updateMutate({ path: { id }, body: request, throwOnError: true });
    },
    [updateMutate]
  );

  const remove = useCallback(
    async (id: string) => {
      return deleteMutate({ path: { id }, throwOnError: true });
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
    provisionOrg,
    isProvisioningOrg,
    update,
    isUpdating,
    updateError,
    remove,
    isDeleting,
    deleteError,
  };
}

export function useUser(id: string) {
  return useGetUser({ path: { id } });
}

export function useOrgUsers() {
  return useListOrgUsers();
}
