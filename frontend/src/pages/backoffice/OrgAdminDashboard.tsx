import { useTranslation } from 'react-i18next';
import { BackOfficeLayout } from '@/components/layout/BackOfficeLayout';
import { StatCard } from '@/components/ui/stat-card';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  ClipboardCheck,
  AlertTriangle,
  Users,
  ArrowUpRight,
  Clock,
  Zap,
  Plus,
  BarChart3,
} from 'lucide-react';

import { useOrgDashboard } from '@/hooks/api/useOrgDashboard';
import { LiveControlMap } from '@/components/dashboard/LiveControlMap';
import { ControlActivityChart } from '@/components/dashboard/ControlActivityChart';
import { useAuth } from '@/hooks/auth/use-auth';
import { formatDistanceToNow } from 'date-fns';
import { useNavigate } from 'react-router-dom';

export default function OrgAdminDashboard() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { user } = useAuth();
  const {
    range,
    setRange,
    stats,
    statsLoading,
    activityFeed,
    activityFeedLoading,
    recentAlerts,
    recentAlertsLoading,
    topAgents,
    topAgentsLoading,
    controlActivity,
    controlActivityLoading,
  } = useOrgDashboard();

  const activityLabel = (status?: string) => {
    if (status === 'critical') return t('orgAdminDashboard.vehicleFlagged');
    if (status === 'warning') return t('orgAdminDashboard.alertTriggered');
    return t('orgAdminDashboard.controlCompleted');
  };

  return (
    <BackOfficeLayout
      title={t('orgAdminDashboard.title')}
      subtitle={t('orgAdminDashboard.subtitle', {
        organization: user?.organization || 'Organization',
      })}
    >
      <div className="space-y-6">
        {/* ── Quick Actions ── */}
        <div className="flex flex-wrap gap-3">
          <Button
            className="rounded-xl h-10 gap-2 px-4 shadow-sm"
            onClick={() => navigate('/backoffice/users')}
          >
            <Plus className="h-4 w-4" />
            {t('orgAdminDashboard.addNewUser')}
          </Button>
          <Button
            variant="outline"
            className="rounded-xl h-10 gap-2 px-4 shadow-sm bg-card"
            onClick={() => navigate('/backoffice/reports')}
          >
            <BarChart3 className="h-4 w-4" />
            {t('orgAdminDashboard.viewReports')}
          </Button>
        </div>

        {/* ── Stats Grid ── */}
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
          <div className="animate-slide-up" style={{ animationDelay: '0ms' }}>
            <StatCard
              title={t('orgAdminDashboard.todayControls')}
              value={statsLoading ? '…' : (stats?.todayControls ?? 0)}
              subtitle={t('orgAdminDashboard.totalControlsProcessed')}
              icon={ClipboardCheck}
              variant="softPrimary"
              loading={statsLoading}
            />
          </div>
          <div className="animate-slide-up" style={{ animationDelay: '80ms' }}>
            <StatCard
              title={t('orgAdminDashboard.activeAlerts')}
              value={statsLoading ? '…' : (stats?.activeAlerts ?? 0)}
              subtitle={t('orgAdminDashboard.requiresImmediateAction')}
              icon={AlertTriangle}
              variant="softCritical"
              loading={statsLoading}
            />
          </div>
          <div className="animate-slide-up" style={{ animationDelay: '160ms' }}>
            <StatCard
              title={t('orgAdminDashboard.pendingSubmissions')}
              value={statsLoading ? '…' : (stats?.pendingSubmissions ?? 0)}
              subtitle={t('orgAdminDashboard.awaitingReview')}
              icon={Clock}
              variant="softWarning"
              loading={statsLoading}
            />
          </div>
          <div className="animate-slide-up" style={{ animationDelay: '240ms' }}>
            <StatCard
              title={t('orgAdminDashboard.onlineAgents')}
              value={statsLoading ? '…' : (stats?.onlineAgents ?? 0)}
              subtitle={t('orgAdminDashboard.currentlyActive')}
              icon={Users}
              variant="softAccent"
              loading={statsLoading}
            />
          </div>
          <div className="animate-slide-up" style={{ animationDelay: '320ms' }}>
            <StatCard
              title={t('orgAdminDashboard.totalUsers')}
              value={statsLoading ? '…' : (stats?.totalUsers ?? 0)}
              subtitle={t('orgAdminDashboard.totalOrganizationUsers')}
              icon={Users}
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
                    <span className="text-sm font-bold">{t('orgAdminDashboard.recentAlerts')}</span>
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 gap-1 rounded-lg px-2 text-xs"
                    onClick={() => navigate('/backoffice/controls?status=alerts')}
                  >
                    {t('orgAdminDashboard.viewAll')} <ArrowUpRight className="h-3 w-3" />
                  </Button>
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-2.5 px-4 pb-4">
                {recentAlertsLoading ? (
                  <div className="flex flex-col items-center gap-2 py-8 text-center text-muted-foreground">
                    <Zap className="h-8 w-8 opacity-30" />
                    <p className="text-sm">Loading alerts…</p>
                  </div>
                ) : recentAlerts.length === 0 ? (
                  <div className="flex flex-col items-center gap-2 py-8 text-center text-muted-foreground">
                    <Zap className="h-8 w-8 opacity-30" />
                    <p className="text-sm">No recent alerts</p>
                  </div>
                ) : (
                  recentAlerts.map((alert) => (
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
                            ? t('orgAdminDashboard.alertTriggered')
                            : t('orgAdminDashboard.criticalAlert')}
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
              data={controlActivity?.series ?? []}
              range={range}
              onRangeChange={setRange}
              loading={controlActivityLoading}
            />
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
                      {t('orgAdminDashboard.topAgentsToday')}
                    </span>
                  </div>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {topAgentsLoading ? (
                    <div className="flex items-center justify-center py-10">
                      <p className="text-sm text-muted-foreground">Loading agents…</p>
                    </div>
                  ) : topAgents.length === 0 ? (
                    <div className="flex items-center justify-center py-10 text-center">
                      <p className="text-sm text-muted-foreground">No activity yet</p>
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
                    {t('orgAdminDashboard.realTimeActivityFeed')}
                  </span>
                </div>
                <div className="flex items-center gap-2 rounded-full border border-status-valid/20 bg-status-valid/10 px-3 py-1">
                  <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-status-valid" />
                  <span className="text-[10px] font-semibold uppercase tracking-widest text-status-valid">
                    {t('orgAdminDashboard.autoUpdating')}
                  </span>
                </div>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid gap-2.5 sm:grid-cols-2 lg:grid-cols-4">
                {activityFeedLoading ? (
                  <div className="col-span-full flex items-center justify-center py-10">
                    <p className="text-sm text-muted-foreground">Loading activity…</p>
                  </div>
                ) : activityFeed.length === 0 ? (
                  <div className="col-span-full flex items-center justify-center py-10">
                    <p className="text-sm text-muted-foreground">No activity yet</p>
                  </div>
                ) : (
                  activityFeed.map((item) => {
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
            </CardContent>
          </Card>
        </div>
      </div>
    </BackOfficeLayout>
  );
}
