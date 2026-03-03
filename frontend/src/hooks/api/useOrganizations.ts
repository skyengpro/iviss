import { useListOrganizations } from '../../openapi-rq/queries/queries';

export function useOrganizations() {
  const { data: organizations, isLoading, error } = useListOrganizations();

  return {
    organizations,
    isLoading,
    error,
  };
}
