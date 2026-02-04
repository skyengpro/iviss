import { useTranslation } from "react-i18next";
import { BackOfficeLayout } from "@/components/layout/BackOfficeLayout";
import { StatCard } from "@/components/ui/stat-card";
import { StatusBadge } from "@/components/ui/status-badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  ClipboardCheck,
  AlertTriangle,
  Users,
  Car,
  TrendingUp,
  MapPin,
  ArrowUpRight,
  Clock,
} from 'lucide-react';

// Mock data for charts and lists
import { useQuery } from "@tanstack/react-query";
import { mockControlService, Translatable } from "@/services/mockControls";
import { mockAuthService } from "@/services/mockAuth";

export default function BackOfficeDashboard() {
  const { t } = useTranslation();
  const { data: stats, isLoading: statsLoading } = useQuery({
    queryKey: ['dashboard-stats'],
    queryFn: () => mockControlService.getStats(),
  });

  const { data: recentAlerts = [], isLoading: alertsLoading } = useQuery({
    queryKey: ['recent-alerts'],
    queryFn: () => mockControlService.getRecentAlerts(5),
  });

  const { data: users = [], isLoading: usersLoading } = useQuery({
    queryKey: ['users'],
    queryFn: () => mockAuthService.getAllUsers(),
  });

  const renderNotes = (notes: Translatable) => {
    if (!notes) return null;
    if (typeof notes === 'string') {
      return notes;
    }
    return t(notes.key, notes.params);
  };
  return (
    <BackOfficeLayout
      title={t('backOfficeDashboard.title')}
      subtitle={t('backOfficeDashboard.subtitle')}
      actions={
        <Button className="gap-2 bg-accent text-accent-foreground hover:bg-accent/90">
          <TrendingUp className="h-4 w-4" />
          {t('backOfficeDashboard.generateReport')}
        </Button>
      }
    >
      <div className="space-y-6">
        {/* Stats Grid */}
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          <StatCard
            title={t('backOfficeDashboard.todayControls')}
            value={stats?.todayControls.toString() || "0"}
            subtitle={t('backOfficeDashboard.totalControlsProcessed')}
            icon={ClipboardCheck}
            variant="gradient"
          />
          <StatCard
            title={t('backOfficeDashboard.activeAlerts')}
            value={stats?.activeAlerts.toString() || "0"}
            subtitle={t('backOfficeDashboard.requiresImmediateAction')}
            icon={AlertTriangle}
            variant="critical"
          />
          <StatCard
            title={t('backOfficeDashboard.vehiclesScanned')}
            value={stats?.totalVehicles.toString() || "0"}
            subtitle={t('backOfficeDashboard.historicalScannedVolume')}
            icon={Car}
            variant="default"
          />
          <StatCard
            title={t('backOfficeDashboard.onlineAgents')}
            value={users.filter(u => u.isActive).length.toString()}
            subtitle={t('backOfficeDashboard.currentlyActive')}
            icon={Users}
            variant="warning"
          />
        </div>

        {/* Main content grid */}
        <div className="grid gap-6 lg:grid-cols-3">
          {/* Live Activity Map Placeholder */}
          <Card className="lg:col-span-2">
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle className="flex items-center gap-2">
                <MapPin className="h-5 w-5 text-accent" />
                {t('backOfficeDashboard.liveControlMap')}
              </CardTitle>
              <div className="flex items-center gap-2">
                <span className="h-2 w-2 animate-pulse rounded-full bg-status-valid" />
                <span className="text-sm text-muted-foreground">{t('backOfficeDashboard.live')}</span>
              </div>
            </CardHeader>
            <CardContent>
              <div className="flex h-[400px] items-center justify-center rounded-lg bg-muted">
                <div className="text-center">
                  <MapPin className="mx-auto h-12 w-12 text-muted-foreground/50" />
                  <p className="mt-2 text-sm text-muted-foreground">
                    {t('backOfficeDashboard.mapPlaceholder')}
                  </p>
                  <Button variant="outline" className="mt-4">
                    {t('backOfficeDashboard.enableLiveView')}
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Recent Alerts */}
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle className="flex items-center gap-2">
                <AlertTriangle className="h-5 w-5 text-status-critical" />
                {t('backOfficeDashboard.recentAlerts')}
              </CardTitle>
              <Button variant="ghost" size="sm" className="gap-1">
                {t('backOfficeDashboard.viewAll')} <ArrowUpRight className="h-3 w-3" />
              </Button>
            </CardHeader>
            <CardContent className="space-y-4">
              {recentAlerts.map((alert) => (
                <div
                  key={alert.id}
                  className="flex items-start gap-3 rounded-lg border border-status-critical/20 bg-status-critical/5 p-3"
                >
                  <div className="mt-0.5 h-2 w-2 shrink-0 animate-pulse rounded-full bg-status-critical" />
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center justify-between gap-2">
                      <span className="font-mono font-semibold tracking-wider">
                        {alert.plateNumber}
                      </span>
                      <span className="text-xs text-muted-foreground">
                        {new Date(alert.timestamp).toLocaleTimeString()}
                      </span>
                    </div>
                    <p className="mt-0.5 text-sm font-medium text-status-critical">
                      {renderNotes(alert.notes) || t('backOfficeDashboard.criticalAlert')}
                    </p>
                    <p className="mt-1 text-xs text-muted-foreground">
                      {alert.location.address} • {alert.agentName}
                    </p>
                  </div>
                </div>
              ))}
            </CardContent>
          </Card>
        </div>

        {/* Secondary content grid */}
        <div className="grid gap-6 lg:grid-cols-2">
          {/* Activity Chart Placeholder */}
          <Card>
            <CardHeader>
              <CardTitle>{t('backOfficeDashboard.controlActivity24h')}</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex h-[200px] items-center justify-center rounded-lg bg-muted">
                <div className="text-center">
                  <TrendingUp className="mx-auto h-8 w-8 text-muted-foreground/50" />
                  <p className="mt-2 text-sm text-muted-foreground">
                    {t('backOfficeDashboard.activityChartPlaceholder')}
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Top Performing Agents */}
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>{t('backOfficeDashboard.topAgentsToday')}</CardTitle>
              <Button variant="ghost" size="sm" className="gap-1">
                {t('backOfficeDashboard.viewAll')} <ArrowUpRight className="h-3 w-3" />
              </Button>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {users.slice(0, 4).map((user, index) => (
                  <div key={user.id} className="flex items-center gap-4">
                    <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary text-primary-foreground text-sm font-semibold">
                      {index + 1}
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="font-medium truncate">{user.name}</p>
                      <p className="text-sm text-muted-foreground">{user.organization}</p>
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

        {/* Recent Activity Feed */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <CardTitle className="flex items-center gap-2">
              <Clock className="h-5 w-5 text-accent" />
              {t('backOfficeDashboard.realTimeActivityFeed')}
            </CardTitle>
            <div className="flex items-center gap-2">
              <span className="h-2 w-2 animate-pulse rounded-full bg-status-valid" />
              <span className="text-sm text-muted-foreground">
                {t('backOfficeDashboard.autoUpdating')}
              </span>
            </div>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {[
                { agent: "Agent Dupont", action: t('backOfficeDashboard.controlCompleted'), plate: "AB-123-CD", status: "valid", time: t('backOfficeDashboard.justNow') },
                { agent: "Agent Martin", action: t('backOfficeDashboard.alertTriggered'), plate: "XY-789-ZW", status: "warning", time: t('backOfficeDashboard.minutesAgo', { count: 2 }) },
                { agent: "Agent Bernard", action: t('backOfficeDashboard.vehicleFlagged'), plate: "EF-456-GH", status: "critical", time: t('backOfficeDashboard.minutesAgo', { count: 5 }) },
                { agent: "Agent Leroy", action: t('backOfficeDashboard.controlCompleted'), plate: "JK-321-LM", status: "valid", time: t('backOfficeDashboard.minutesAgo', { count: 8 }) },
              ].map((item, index) => (
                <div key={index} className="flex items-center gap-4 rounded-lg bg-muted/50 p-3">
                  <div
                    className={`h-2 w-2 rounded-full ${
                      item.status === 'valid'
                        ? 'bg-status-valid'
                        : item.status === 'warning'
                          ? 'bg-status-warning'
                          : 'bg-status-critical'
                    }`}
                  />
                  <div className="flex-1 min-w-0">
                    <p className="text-sm">
                      <span className="font-medium">{item.agent}</span>
                      {' • '}
                      <span className="text-muted-foreground">{item.action}</span>
                    </p>
                    <p className="font-mono text-sm font-semibold tracking-wider">{item.plate}</p>
                  </div>
                  <span className="text-xs text-muted-foreground">{item.time}</span>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </BackOfficeLayout>
  );
}
