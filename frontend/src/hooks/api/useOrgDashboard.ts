import { useState } from 'react';
import {
  useGetOrgDashboardStats,
  useGetOrgActivityFeed,
  useGetOrgRecentAlerts,
  useGetOrgTopAgents,
  useGetOrgControlActivity,
} from '../../openapi-rq/queries/queries';
import { DashboardRange } from '../../openapi-rq/requests/types.gen';

export function useOrgDashboard() {
  const [range, setRange] = useState<DashboardRange>('24h');

  const statsQuery = useGetOrgDashboardStats({}, undefined, {
    refetchInterval: 30000,
  });

  const activityFeedQuery = useGetOrgActivityFeed(
    { query: { limit: 8 } },
    undefined,
    { refetchInterval: 30000 }
  );

  const recentAlertsQuery = useGetOrgRecentAlerts(
    { query: { limit: 5 } },
    undefined,
    { refetchInterval: 30000 }
  );

  const topAgentsQuery = useGetOrgTopAgents(
    { query: { range, limit: 5 } },
    undefined,
    { refetchInterval: 30000 }
  );

  const controlActivityQuery = useGetOrgControlActivity(
    { query: { range } },
    undefined,
    { refetchInterval: 30000 }
  );

  return {
    // State
    range,
    setRange,

    // Stats
    stats: statsQuery.data,
    statsLoading: statsQuery.isLoading,
    statsError: statsQuery.error,
    refetchStats: statsQuery.refetch,

    // Activity Feed
    activityFeed: activityFeedQuery.data?.items ?? [],
    activityFeedLoading: activityFeedQuery.isLoading,

    // Recent Alerts
    recentAlerts: recentAlertsQuery.data?.items ?? [],
    recentAlertsLoading: recentAlertsQuery.isLoading,

    // Top Agents
    topAgents: topAgentsQuery.data?.agents ?? [],
    topAgentsLoading: topAgentsQuery.isLoading,

    // Control Activity (Chart data)
    controlActivity: controlActivityQuery.data,
    controlActivityLoading: controlActivityQuery.isLoading,

    // Combined loading state for initial load
    isInitialLoading:
      statsQuery.isLoading ||
      activityFeedQuery.isLoading ||
      recentAlertsQuery.isLoading ||
      topAgentsQuery.isLoading ||
      controlActivityQuery.isLoading,
  };
}
