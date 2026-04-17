import {
  useListOrganizations,
  useCreateOrganization,
  useUpdateOrganization,
  useDeleteOrganization,
} from '../../openapi-rq/queries/queries';
import type { CreateOrganizationRequest, UpdateOrganizationRequest } from '@/openapi-rq/types.gen';

export function useOrganizations() {
  const { data: organizations, isLoading, error, refetch } = useListOrganizations({}, [], {});

  const createMutation = useCreateOrganization([], {
    onSuccess: () => {
      refetch();
    },
  });

  const updateMutation = useUpdateOrganization([], {
    onSuccess: () => {
      refetch();
    },
  });

  const deleteMutation = useDeleteOrganization([], {
    onSuccess: () => {
      refetch();
    },
  });

  return {
    organizations,
    isLoading,
    error,
    refetch,
    createOrganization: (data: CreateOrganizationRequest) =>
      createMutation.mutateAsync({ body: data }),
    updateOrganization: (id: string, data: UpdateOrganizationRequest) =>
      updateMutation.mutateAsync({ path: { id }, body: data }),
    deleteOrganization: (id: string) => deleteMutation.mutateAsync({ path: { id } }),
    isCreating: createMutation.isPending,
    isUpdating: updateMutation.isPending,
    isDeleting: deleteMutation.isPending,
  };
}
