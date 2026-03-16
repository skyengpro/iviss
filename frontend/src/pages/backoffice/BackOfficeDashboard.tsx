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
  Car,
  TrendingUp,
  ArrowUpRight,
  Clock,
  RefreshCw,
  Zap,
} from 'lucide-react';

import { useDashboard } from '@/hooks/api/useDashboard';
import { LiveControlMap } from '@/components/dashboard/LiveControlMap';
import { ControlActivityChart } from '@/components/dashboard/ControlActivityChart';
import { useEffect } from 'react';

// Mock data for charts and lists
import { useQuery } from '@tanstack/react-query';
import { mockControlService, Translatable } from '@/services/mock/mockControls';
import { mockAuthService } from '@/services/mock/mockAuth';

export default function BackOfficeDashboard() {
  const { t } = useTranslation();
  const { stats, isLoading: statsLoading, refetch: refetchStats } = useDashboard();

  // Auto-refresh stats every 30 seconds for "live" feel
  useEffect(() => {
    const interval = setInterval(() => {
      refetchStats();
    }, 30000);
    return () => clearInterval(interval);
  }, [refetchStats]);

  const { data: recentAlerts = [] } = useQuery({
    queryKey: ['recent-alerts'],
    queryFn: () => mockControlService.getRecentAlerts(5),
  });

  const { data: users = [] } = useQuery({
    queryKey: ['users'],
    queryFn: () => mockAuthService.getAllUsers(),
  });

  const renderNotes = (notes: Translatable) => {
    if (!notes) return null;
    if (typeof notes === 'string') return notes;
    return t(notes.key, notes.params);
  };

  return (
    <BackOfficeLayout
      title={t('backOfficeDashboard.title')}
      subtitle={t('backOfficeDashboard.subtitle')}
      actions={
        <div className="flex items-center gap-3">
          <button
            onClick={() => refetchStats()}
            className="flex items-center gap-1.5 rounded-xl border border-border bg-background px-4 py-2 text-sm font-medium text-muted-foreground transition hover:bg-muted hover:text-foreground"
          >
            <RefreshCw className="h-3.5 w-3.5" />
            Refresh
          </button>
          <Button className="gap-2 rounded-xl bg-gradient-to-r from-primary to-[hsl(222,47%,32%)] text-white shadow-lg hover:opacity-90 transition-opacity">
            <TrendingUp className="h-4 w-4" />
            {t('backOfficeDashboard.generateReport')}
          </Button>
        </div>
      }
    >
      <div className="space-y-6">
        {/* ── Stats Grid ── */}
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <div className="animate-slide-up" style={{ animationDelay: '0ms' }}>
            <StatCard
              title={t('backOfficeDashboard.todayControls')}
              value={statsLoading ? '…' : (stats?.todayControls ?? 0)}
              subtitle={t('backOfficeDashboard.totalControlsProcessed')}
              icon={ClipboardCheck}
              variant="gradient"
              loading={statsLoading}
            />
          </div>
          <div className="animate-slide-up" style={{ animationDelay: '80ms' }}>
            <StatCard
              title={t('backOfficeDashboard.activeAlerts')}
              value={statsLoading ? '…' : (stats?.activeAlerts ?? 0)}
              subtitle={t('backOfficeDashboard.requiresImmediateAction')}
              icon={AlertTriangle}
              variant="critical"
              loading={statsLoading}
            />
          </div>
          <div className="animate-slide-up" style={{ animationDelay: '160ms' }}>
            <StatCard
              title={t('backOfficeDashboard.vehiclesScanned')}
              value={statsLoading ? '…' : (stats?.totalVehicles ?? 0)}
              subtitle={t('backOfficeDashboard.historicalScannedVolume')}
              icon={Car}
              variant="default"
              loading={statsLoading}
            />
          </div>
          <div className="animate-slide-up" style={{ animationDelay: '240ms' }}>
            <StatCard
              title={t('backOfficeDashboard.onlineAgents')}
              value={statsLoading ? '…' : (stats?.onlineAgents ?? 0)}
              subtitle={t('backOfficeDashboard.currentlyActive')}
              icon={Users}
              variant="warning"
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
                  <Button variant="ghost" size="sm" className="h-7 gap-1 rounded-lg px-2 text-xs">
                    {t('backOfficeDashboard.viewAll')} <ArrowUpRight className="h-3 w-3" />
                  </Button>
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-2.5 px-4 pb-4">
                {recentAlerts.length === 0 ? (
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
                      <span className="mt-1.5 h-2 w-2 shrink-0 animate-pulse rounded-full bg-status-critical" />
                      <div className="min-w-0 flex-1">
                        <div className="flex items-baseline justify-between gap-1">
                          <span className="font-mono text-xs font-bold tracking-widest text-foreground">
                            {alert.plateNumber}
                          </span>
                          <span className="shrink-0 text-[10px] text-muted-foreground">
                            {new Date(alert.timestamp).toLocaleTimeString([], {
                              hour: '2-digit',
                              minute: '2-digit',
                            })}
                          </span>
                        </div>
                        <p className="mt-0.5 line-clamp-2 text-xs font-medium text-status-critical">
                          {renderNotes(alert.notes) || t('backOfficeDashboard.criticalAlert')}
                        </p>
                        <p className="mt-1 truncate text-[10px] text-muted-foreground">
                          {alert.location.address} · {alert.agentName}
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
            <ControlActivityChart data={stats?.activity24h || []} />
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
                  <Button variant="ghost" size="sm" className="h-7 gap-1 rounded-lg px-2 text-xs">
                    {t('backOfficeDashboard.viewAll')} <ArrowUpRight className="h-3 w-3" />
                  </Button>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {users.slice(0, 5).map((user, index) => (
                    <div
                      key={user.id}
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
                        {user.name?.charAt(0).toUpperCase()}
                      </div>

                      {/* Info */}
                      <div className="min-w-0 flex-1">
                        <p className="truncate text-sm font-semibold text-foreground">
                          {user.name}
                        </p>
                        <p className="truncate text-xs text-muted-foreground">
                          {user.organization}
                        </p>
                      </div>

                      {user.isActive && (
                        <StatusBadge variant="valid" size="sm">
                          {t('backOfficeDashboard.online')}
                        </StatusBadge>
                      )}
                    </div>
                  ))}
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
                <div className="flex items-center gap-2 rounded-full border border-status-valid/20 bg-status-valid/10 px-3 py-1">
                  <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-status-valid" />
                  <span className="text-[10px] font-semibold uppercase tracking-widest text-status-valid">
                    {t('backOfficeDashboard.autoUpdating')}
                  </span>
                </div>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid gap-2.5 sm:grid-cols-2 lg:grid-cols-4">
                {[
                  {
                    agent: 'Agent Dupont',
                    action: t('backOfficeDashboard.controlCompleted'),
                    plate: 'AB-123-CD',
                    status: 'valid',
                    time: t('backOfficeDashboard.justNow'),
                  },
                  {
                    agent: 'Agent Martin',
                    action: t('backOfficeDashboard.alertTriggered'),
                    plate: 'XY-789-ZW',
                    status: 'warning',
                    time: t('backOfficeDashboard.minutesAgo', { count: 2 }),
                  },
                  {
                    agent: 'Agent Bernard',
                    action: t('backOfficeDashboard.vehicleFlagged'),
                    plate: 'EF-456-GH',
                    status: 'critical',
                    time: t('backOfficeDashboard.minutesAgo', { count: 5 }),
                  },
                  {
                    agent: 'Agent Leroy',
                    action: t('backOfficeDashboard.controlCompleted'),
                    plate: 'JK-321-LM',
                    status: 'valid',
                    time: t('backOfficeDashboard.minutesAgo', { count: 8 }),
                  },
                ].map((item, index) => (
                  <div
                    key={index}
                    className="group flex cursor-pointer flex-col gap-2 rounded-xl border border-border/50 bg-muted/30 p-3.5 transition-all hover:bg-muted hover:shadow-sm"
                  >
                    <div className="flex items-center justify-between gap-2">
                      <div
                        className={`h-2 w-2 shrink-0 rounded-full ${
                          item.status === 'valid'
                            ? 'bg-status-valid'
                            : item.status === 'warning'
                              ? 'bg-status-warning'
                              : 'bg-status-critical'
                        }`}
                      />
                      <span className="ml-auto text-[10px] text-muted-foreground">{item.time}</span>
                    </div>
                    <div>
                      <p className="font-mono text-sm font-bold tracking-widest text-foreground">
                        {item.plate}
                      </p>
                      <p className="mt-0.5 text-xs text-muted-foreground line-clamp-1">
                        {item.action}
                      </p>
                    </div>
                    <p className="text-[10px] font-medium text-muted-foreground">{item.agent}</p>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </BackOfficeLayout>
  );
}
