import {
  useListOrganizations,
  useCreateOrganization,
  useUpdateOrganization,
  useDeleteOrganization,
} from '../../openapi-rq/queries/queries';

export function useOrganizations() {
  const { data: organizations, isLoading, error, refetch } = useListOrganizations();

  const createMutation = useCreateOrganization(undefined, {
    onSuccess: () => {
      refetch();
    },
  });

  const updateMutation = useUpdateOrganization(undefined, {
    onSuccess: () => {
      refetch();
    },
  });

  const deleteMutation = useDeleteOrganization(undefined, {
    onSuccess: () => {
      refetch();
    },
  });

  return {
    organizations,
    isLoading,
    error,
    refetch,
    createOrganization: (data: any) => createMutation.mutateAsync({ body: data }),
    updateOrganization: (id: string, data: any) =>
      updateMutation.mutateAsync({ path: { id }, body: data }),
    deleteOrganization: (id: string) => deleteMutation.mutateAsync({ path: { id } }),
    isCreating: createMutation.isPending,
    isUpdating: updateMutation.isPending,
    isDeleting: deleteMutation.isPending,
  };
}
