import { useGetDashboardStats } from '../../openapi-rq/queries/queries';

export function useDashboard() {
    const { data: stats, isLoading, error, refetch } = useGetDashboardStats();

    return {
        stats,
        isLoading,
        error,
        refetch,
    };
}
