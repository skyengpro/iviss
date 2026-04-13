import { useTranslation } from 'react-i18next';
import { BackOfficeLayout } from '@/components/layout/BackOfficeLayout';
import { StatCard } from '@/components/ui/stat-card';
import { StatusBadge } from '@/components/ui/status-badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  ClipboardCheck,
  AlertTriangle,
  Users,
  Building2,
  ArrowUpRight,
  Clock,
  Zap,
} from 'lucide-react';

import { useDashboard } from '@/hooks/api/useDashboard';
import { LiveControlMap } from '@/components/dashboard/LiveControlMap';
import { ControlActivityChart } from '@/components/dashboard/ControlActivityChart';
import { useMemo, useState } from 'react';
import {
  useGetActivityFeed,
  useGetControlActivity,
  useGetRecentAlerts,
  useGetTopAgents,
} from '@/openapi-rq/queries/queries';
import { DashboardRange } from '@/openapi-rq/requests/types.gen';

// Mock data for charts and lists
import { useQuery } from '@tanstack/react-query';
import { mockControlService, Translatable } from '@/services/mock/mockControls';
import { formatDistanceToNow } from 'date-fns';
import { useNavigate } from 'react-router-dom';

export default function BackOfficeDashboard() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const {
    stats,
    isLoading: statsLoading,
    refetch: refetchStats,
    isFetching: statsIsFetching,
    isRefetching: statsIsRefetching,
    dataUpdatedAt: statsUpdatedAt,
  } = useDashboard();

  const recentAlertsQuery = useGetRecentAlerts(
    {
      limit: 5,
    },
    undefined,
    {
      refetchInterval: 30000,
    }
  );

  const recentAlertsItems = useMemo(
    () => recentAlertsQuery.data?.items ?? [],
    [recentAlertsQuery.data]
  );

  const [dashboardRange, setDashboardRange] = useState<DashboardRange>('24h');

  const controlActivityQuery = useGetControlActivity(
    {
      range: dashboardRange,
    },
    undefined,
    {
      refetchInterval: 30000,
    }
  );

  const topAgentsQuery = useGetTopAgents(
    {
      range: dashboardRange,
      limit: 5,
    },
    undefined,
    {
      refetchInterval: 30000,
    }
  );

  const topAgents = useMemo(() => topAgentsQuery.data?.agents ?? [], [topAgentsQuery.data]);

  const activityFeedQuery = useGetActivityFeed(
    {
      limit: 8,
    },
    undefined,
    {
      refetchInterval: 15000,
    }
  );

  const activityFeedItems = useMemo(
    () => (activityFeedQuery.data?.items ?? []).slice(0, 8),
    [activityFeedQuery.data]
  );

  const activityLabel = (status?: string) => {
    if (status === 'critical') return t('backOfficeDashboard.vehicleFlagged');
    if (status === 'warning') return t('backOfficeDashboard.alertTriggered');
    return t('backOfficeDashboard.controlCompleted');
  };

  const renderNotes = (notes: Translatable) => {
    if (!notes) return null;
    if (typeof notes === 'string') return notes;
    return t(notes.key, notes.params);
  };

  const isAnyUpdating =
    statsIsFetching ||
    statsIsRefetching ||
    recentAlertsQuery.isFetching ||
    recentAlertsQuery.isRefetching ||
    controlActivityQuery.isFetching ||
    controlActivityQuery.isRefetching ||
    topAgentsQuery.isFetching ||
    topAgentsQuery.isRefetching ||
    activityFeedQuery.isFetching ||
    activityFeedQuery.isRefetching;

  const handleRefreshAll = async () => {
    await Promise.all([
      refetchStats(),
      recentAlertsQuery.refetch(),
      controlActivityQuery.refetch(),
      topAgentsQuery.refetch(),
      activityFeedQuery.refetch(),
    ]);
  };

  const lastUpdatedText = (dataUpdatedAt?: number) => {
    if (!dataUpdatedAt) return null;
    return `Last updated ${formatDistanceToNow(new Date(dataUpdatedAt), { addSuffix: true })}`;
  };

  return (
    <BackOfficeLayout
      title={t('backOfficeDashboard.title')}
      subtitle={t('backOfficeDashboard.subtitle')}
      actions={
        <div className="flex items-center gap-3">
          {isAnyUpdating && <span className="text-xs text-muted-foreground">Updating…</span>}
          <Button
            variant="outline"
            size="sm"
            className="h-8 rounded-xl"
            onClick={handleRefreshAll}
            disabled={isAnyUpdating}
          >
            Refresh
          </Button>
        </div>
      }
    >
      <div className="space-y-6">
        {/* ── Stats Grid ── */}
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
          <div className="animate-slide-up" style={{ animationDelay: '0ms' }}>
            <StatCard
              title={t('backOfficeDashboard.todayControls')}
              value={statsLoading ? '…' : (stats?.todayControls ?? 0)}
              subtitle={t('backOfficeDashboard.totalControlsProcessed')}
              icon={ClipboardCheck}
              variant="softPrimary"
              loading={statsLoading}
            />
          </div>
          <div className="animate-slide-up" style={{ animationDelay: '80ms' }}>
            <StatCard
              title={t('backOfficeDashboard.activeAlerts')}
              value={statsLoading ? '…' : (stats?.activeAlerts ?? 0)}
              subtitle={t('backOfficeDashboard.requiresImmediateAction')}
              icon={AlertTriangle}
              variant="softCritical"
              loading={statsLoading}
            />
          </div>
          <div className="animate-slide-up" style={{ animationDelay: '160ms' }}>
            <StatCard
              title={t('backOfficeDashboard.pendingSubmissions')}
              value={statsLoading ? '…' : (stats?.pendingSubmissions ?? 0)}
              subtitle={t('backOfficeDashboard.awaitingReview')}
              icon={Clock}
              variant="softWarning"
              loading={statsLoading}
            />
          </div>
          <div className="animate-slide-up" style={{ animationDelay: '240ms' }}>
            <StatCard
              title={t('backOfficeDashboard.onlineAgents')}
              value={statsLoading ? '…' : (stats?.onlineAgents ?? 0)}
              subtitle={t('backOfficeDashboard.currentlyActive')}
              icon={Users}
              variant="softAccent"
              loading={statsLoading}
            />
          </div>
          <div className="animate-slide-up" style={{ animationDelay: '320ms' }}>
            <StatCard
              title={t('backOfficeDashboard.organizations')}
              value={statsLoading ? '…' : (stats?.organizationsCount ?? 0)}
              subtitle={t('backOfficeDashboard.totalOrganizations')}
              icon={Building2}
              variant="soft"
              loading={statsLoading}
            />
          </div>
        </div>

        {/* ── Map + Alerts row ── */}
        <div className="grid gap-6 lg:grid-cols-3">
          <div className="animate-fade-in lg:col-span-2" style={{ animationDelay: '100ms' }}>
            <LiveControlMap agents={stats?.liveAgents || []} />
          </div>

          {/* Recent Alerts */}
          <div className="animate-fade-in" style={{ animationDelay: '200ms' }}>
            <Card className="h-full rounded-2xl border-none bg-gradient-to-br from-[hsl(0,84%,97%)] to-white shadow-md dark:from-[hsl(0,30%,15%)] dark:to-[hsl(222,47%,12%)]">
              <CardHeader className="pb-3">
                <CardTitle className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-status-critical/10">
                      <AlertTriangle className="h-4 w-4 text-status-critical" />
                    </div>
                    <span className="text-sm font-bold">
                      {t('backOfficeDashboard.recentAlerts')}
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    {recentAlertsQuery.isFetching && !recentAlertsQuery.isLoading && (
                      <span className="text-xs text-muted-foreground">Updating…</span>
                    )}
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-7 gap-1 rounded-lg px-2 text-xs"
                      onClick={() => navigate('/backoffice/controls?status=alerts')}
                    >
                      {t('backOfficeDashboard.viewAll')} <ArrowUpRight className="h-3 w-3" />
                    </Button>
                  </div>
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-2.5 px-4 pb-4">
                {recentAlertsQuery.isError ? (
                  <div className="flex flex-col items-center gap-3 py-8 text-center text-muted-foreground">
                    <p className="text-sm">Failed to load alerts</p>
                    <p className="text-xs">
                      {lastUpdatedText(recentAlertsQuery.dataUpdatedAt) ?? '—'}
                    </p>
                    <Button
                      variant="outline"
                      size="sm"
                      className="h-8 rounded-xl"
                      onClick={() => recentAlertsQuery.refetch()}
                      disabled={recentAlertsQuery.isFetching || recentAlertsQuery.isRefetching}
                    >
                      Retry
                    </Button>
                  </div>
                ) : recentAlertsQuery.isLoading ? (
                  <div className="flex flex-col items-center gap-2 py-8 text-center text-muted-foreground">
                    <Zap className="h-8 w-8 opacity-30" />
                    <p className="text-sm">Loading alerts…</p>
                  </div>
                ) : recentAlertsItems.length === 0 ? (
                  <div className="flex flex-col items-center gap-2 py-8 text-center text-muted-foreground">
                    <Zap className="h-8 w-8 opacity-30" />
                    <p className="text-sm">No recent alerts</p>
                    <p className="text-xs">
                      {lastUpdatedText(recentAlertsQuery.dataUpdatedAt) ?? '—'}
                    </p>
                  </div>
                ) : (
                  recentAlertsItems.map((alert) => (
                    <div
                      key={alert.id}
                      className="group flex cursor-pointer items-start gap-3 rounded-xl border border-status-critical/15 bg-status-critical/5 p-3 transition-all hover:border-status-critical/30 hover:bg-status-critical/10"
                    >
                      <span
                        className={`mt-1.5 h-2 w-2 shrink-0 animate-pulse rounded-full ${
                          alert.overallStatus === 'warning'
                            ? 'bg-status-warning'
                            : 'bg-status-critical'
                        }`}
                      />
                      <div className="min-w-0 flex-1">
                        <div className="flex items-baseline justify-between gap-1">
                          <span className="font-mono text-xs font-bold tracking-widest text-foreground">
                            {alert.plateNumber}
                          </span>
                          <span className="shrink-0 text-[10px] text-muted-foreground">
                            {formatDistanceToNow(new Date(alert.createdAt), { addSuffix: true })}
                          </span>
                        </div>
                        <p
                          className={`mt-0.5 line-clamp-2 text-xs font-medium ${
                            alert.overallStatus === 'warning'
                              ? 'text-status-warning'
                              : 'text-status-critical'
                          }`}
                        >
                          {alert.overallStatus === 'warning'
                            ? t('backOfficeDashboard.alertTriggered')
                            : t('backOfficeDashboard.criticalAlert')}
                        </p>
                        <p className="mt-1 truncate text-[10px] text-muted-foreground">
                          {(alert.address ?? '—') + ' · ' + alert.agentName}
                        </p>
                      </div>
                    </div>
                  ))
                )}
              </CardContent>
            </Card>
          </div>
        </div>

        {/* ── Chart + Agents row ── */}
        <div className="grid gap-6 lg:grid-cols-2">
          <div className="animate-fade-in" style={{ animationDelay: '150ms' }}>
            <ControlActivityChart
              data={controlActivityQuery.data?.series ?? []}
              range={dashboardRange}
              onRangeChange={setDashboardRange}
              loading={controlActivityQuery.isLoading}
            />
            {controlActivityQuery.isError && (
              <div className="mt-2 flex items-center justify-between rounded-xl border border-border/60 bg-card px-3 py-2 text-xs text-muted-foreground">
                <span>{lastUpdatedText(controlActivityQuery.dataUpdatedAt) ?? '—'}</span>
                <Button
                  variant="outline"
                  size="sm"
                  className="h-7 rounded-xl"
                  onClick={() => controlActivityQuery.refetch()}
                  disabled={controlActivityQuery.isFetching || controlActivityQuery.isRefetching}
                >
                  Retry
                </Button>
              </div>
            )}
          </div>

          {/* Top Performing Agents */}
          <div className="animate-fade-in" style={{ animationDelay: '250ms' }}>
            <Card className="h-full rounded-2xl border border-border/60 bg-card shadow-md">
              <CardHeader className="pb-3">
                <CardTitle className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-primary/10">
                      <Users className="h-4 w-4 text-primary" />
                    </div>
                    <span className="text-sm font-bold">
                      {t('backOfficeDashboard.topAgentsToday')}
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    {topAgentsQuery.isFetching && !topAgentsQuery.isLoading && (
                      <span className="text-xs text-muted-foreground">Updating…</span>
                    )}
                    <Button variant="ghost" size="sm" className="h-7 gap-1 rounded-lg px-2 text-xs">
                      {t('backOfficeDashboard.viewAll')} <ArrowUpRight className="h-3 w-3" />
                    </Button>
                  </div>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {topAgentsQuery.isError ? (
                    <div className="flex flex-col items-center justify-center gap-3 py-10 text-center">
                      <p className="text-sm text-muted-foreground">Failed to load agents</p>
                      <p className="text-xs text-muted-foreground">
                        {lastUpdatedText(topAgentsQuery.dataUpdatedAt) ?? '—'}
                      </p>
                      <Button
                        variant="outline"
                        size="sm"
                        className="h-8 rounded-xl"
                        onClick={() => topAgentsQuery.refetch()}
                        disabled={topAgentsQuery.isFetching || topAgentsQuery.isRefetching}
                      >
                        Retry
                      </Button>
                    </div>
                  ) : topAgents.length === 0 ? (
                    <div className="flex items-center justify-center py-10">
                      <p className="text-sm text-muted-foreground">No activity yet</p>
                      <p className="text-xs text-muted-foreground">
                        {lastUpdatedText(topAgentsQuery.dataUpdatedAt) ?? '—'}
                      </p>
                    </div>
                  ) : (
                    topAgents.map((agent, index) => (
                      <div
                        key={agent.agentId}
                        className="group flex cursor-pointer items-center gap-3 rounded-xl p-2.5 transition-all hover:bg-muted/60"
                      >
                        {/* Rank */}
                        <div
                          className={`flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-xs font-bold ${
                            index === 0
                              ? 'bg-gradient-to-br from-amber-400 to-amber-600 text-white'
                              : index === 1
                                ? 'bg-gradient-to-br from-slate-300 to-slate-500 text-white'
                                : index === 2
                                  ? 'bg-gradient-to-br from-orange-400 to-orange-600 text-white'
                                  : 'bg-muted text-muted-foreground'
                          }`}
                        >
                          {index + 1}
                        </div>

                        {/* Avatar */}
                        <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-primary/15 text-sm font-bold text-primary">
                          {agent.agentName?.charAt(0).toUpperCase()}
                        </div>

                        {/* Info */}
                        <div className="min-w-0 flex-1">
                          <p className="truncate text-sm font-semibold text-foreground">
                            {agent.agentName}
                          </p>
                          <p className="truncate text-xs text-muted-foreground">
                            {agent.organizationName}
                          </p>
                        </div>

                        {agent.isOnline && (
                          <StatusBadge variant="valid" size="sm">
                            {t('backOfficeDashboard.online')}
                          </StatusBadge>
                        )}
                      </div>
                    ))
                  )}
                </div>
              </CardContent>
            </Card>
          </div>
        </div>

        {/* ── Real-time Activity Feed ── */}
        <div className="animate-fade-in" style={{ animationDelay: '300ms' }}>
          <Card className="rounded-2xl border border-border/60 bg-card shadow-md">
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-accent/10">
                    <Clock className="h-4 w-4 text-accent" />
                  </div>
                  <span className="text-sm font-bold">
                    {t('backOfficeDashboard.realTimeActivityFeed')}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  {activityFeedQuery.isFetching && !activityFeedQuery.isLoading && (
                    <span className="text-xs text-muted-foreground">Updating…</span>
                  )}
                  <div className="flex items-center gap-2 rounded-full border border-status-valid/20 bg-status-valid/10 px-3 py-1">
                    <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-status-valid" />
                    <span className="text-[10px] font-semibold uppercase tracking-widest text-status-valid">
                      {t('backOfficeDashboard.autoUpdating')}
                    </span>
                  </div>
                </div>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid gap-2.5 sm:grid-cols-2 lg:grid-cols-4">
                {activityFeedQuery.isError ? (
                  <div className="col-span-full flex flex-col items-center justify-center gap-3 py-10 text-center">
                    <p className="text-sm text-muted-foreground">Failed to load activity</p>
                    <p className="text-xs text-muted-foreground">
                      {lastUpdatedText(activityFeedQuery.dataUpdatedAt) ?? '—'}
                    </p>
                    <Button
                      variant="outline"
                      size="sm"
                      className="h-8 rounded-xl"
                      onClick={() => activityFeedQuery.refetch()}
                      disabled={activityFeedQuery.isFetching || activityFeedQuery.isRefetching}
                    >
                      Retry
                    </Button>
                  </div>
                ) : activityFeedQuery.isLoading ? (
                  <div className="col-span-full flex items-center justify-center py-10">
                    <p className="text-sm text-muted-foreground">Loading activity…</p>
                  </div>
                ) : activityFeedItems.length === 0 ? (
                  <div className="col-span-full flex items-center justify-center py-10">
                    <div className="text-center">
                      <p className="text-sm text-muted-foreground">No activity yet</p>
                      <p className="mt-1 text-xs text-muted-foreground">
                        {lastUpdatedText(activityFeedQuery.dataUpdatedAt) ?? '—'}
                      </p>
                    </div>
                  </div>
                ) : (
                  activityFeedItems.map((item) => {
                    const status = item.overallStatus;
                    const timeAgo = formatDistanceToNow(new Date(item.createdAt), {
                      addSuffix: true,
                    });
                    return (
                      <div
                        key={item.id}
                        className="group flex cursor-pointer flex-col gap-2 rounded-xl border border-border/50 bg-muted/30 p-3.5 transition-all hover:bg-muted hover:shadow-sm"
                      >
                        <div className="flex items-center justify-between gap-2">
                          <div
                            className={`h-2 w-2 shrink-0 rounded-full ${
                              status === 'valid'
                                ? 'bg-status-valid'
                                : status === 'warning'
                                  ? 'bg-status-warning'
                                  : 'bg-status-critical'
                            }`}
                          />
                          <span className="ml-auto text-[10px] text-muted-foreground">
                            {timeAgo}
                          </span>
                        </div>
                        <div>
                          <p className="font-mono text-sm font-bold tracking-widest text-foreground">
                            {item.plateNumber}
                          </p>
                          <p className="mt-0.5 text-xs text-muted-foreground line-clamp-1">
                            {activityLabel(status)}
                          </p>
                        </div>
                        <p className="text-[10px] font-medium text-muted-foreground">
                          {item.agentName}
                        </p>
                      </div>
                    );
                  })
                )}
              </div>
              <div className="mt-3 text-[10px] text-muted-foreground">
                {lastUpdatedText(activityFeedQuery.dataUpdatedAt) ?? ''}
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </BackOfficeLayout>
  );
}
