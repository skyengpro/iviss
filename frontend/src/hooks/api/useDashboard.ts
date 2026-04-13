import { useGetDashboardStats } from '../../openapi-rq/queries/queries';

export function useDashboard() {
  const query = useGetDashboardStats(undefined, {
    refetchInterval: 30000,
  });
  const { data: stats, isLoading, error, refetch, isFetching, isRefetching, dataUpdatedAt } = query;

  return {
    stats,
    isLoading,
    error,
    refetch,
    isFetching,
    isRefetching,
    dataUpdatedAt,
  };
}
