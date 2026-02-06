import { useTranslation } from 'react-i18next';
import { MobileLayout } from '@/components/layout/MobileLayout';
import { StatCard } from '@/components/ui/stat-card';
import { StatusBadge } from '@/components/ui/status-badge';
import {
  Camera,
  Keyboard,
  Radio,
  ClipboardCheck,
  AlertTriangle,
  ArrowRight,
  MapPin,
  Clock,
} from 'lucide-react';
import { Link, useNavigate, useLocation } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { useAuth } from '@/hooks/use-auth';
import { mockControlService } from '@/services/mockControls';

export default function MobileDashboard() {
  const { user } = useAuth();
  const { t } = useTranslation();

  const { data: stats, isLoading: statsLoading } = useQuery({
    queryKey: ['mobile-stats', user?.organizationId],
    queryFn: () => (user ? mockControlService.getStats(user.organizationId) : null),
    enabled: !!user,
  });

  const { data: recentControls = [], isLoading: controlsLoading } = useQuery({
    queryKey: ['recent-controls', user?.id],
    queryFn: () =>
      user ? mockControlService.getTodayControlsByAgent(user.id) : Promise.resolve([]),
    enabled: !!user,
  });

  const isLoading = statsLoading || controlsLoading;

  const formatTimeAgo = (date: Date) => {
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.round(diffMs / 60000);

    if (diffMins < 1) return t('mobileDashboard.justNow');
    if (diffMins < 60) return t('mobileDashboard.minutesAgo', { count: diffMins });

    const diffHours = Math.round(diffMins / 60);
    if (diffHours < 24) return t('mobileDashboard.hoursAgo', { count: diffHours });

    return date.toLocaleDateString();
  };

  return (
    <MobileLayout title="titles.dashboard">
      <div className="p-4 space-y-6">
        {/* Welcome message */}
        {user && (
          <div className="rounded-xl bg-gradient-to-r from-primary to-primary/80 p-4 text-primary-foreground">
            <p className="text-sm opacity-80">{t('mobileDashboard.welcome')}</p>
            <p className="text-lg font-semibold">{user.name}</p>
            <p className="text-xs opacity-70 mt-1">{user.organization}</p>
          </div>
        )}

        {/* Quick Actions */}
        <section>
          <h2 className="mb-3 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
            {t('mobileDashboard.newControl')}
          </h2>
          <div className="grid grid-cols-3 gap-3">
            <QuickActionButton
              icon={Keyboard}
              label={t('mobileDashboard.manualEntry')}
              href="/mobile/search"
            />
            <QuickActionButton
              icon={Camera}
              label={t('mobileDashboard.photoScan')}
              href="/mobile/scan?mode=photo"
              primary
            />
            <QuickActionButton
              icon={Radio}
              label={t('mobileDashboard.liveScan')}
              href="/mobile/scan?mode=live"
            />
          </div>
        </section>

        {/* Today's Stats */}
        <section>
          <h2 className="mb-3 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
            {t('mobileDashboard.todaysActivity')}
          </h2>
          <div className="grid grid-cols-2 gap-3">
            <StatCard
              title={t('mobileDashboard.controls')}
              value={isLoading ? '-' : String(stats?.today || 0)}
              subtitle={t('mobileDashboard.today')}
              icon={ClipboardCheck}
              variant="gradient"
            />
            <StatCard
              title={t('mobileDashboard.alerts')}
              value={isLoading ? '-' : String(stats?.alerts || 0)}
              subtitle={t('mobileDashboard.flaggedVehicles')}
              icon={AlertTriangle}
              variant={(stats?.alerts || 0) > 0 ? 'critical' : 'default'}
            />
          </div>
        </section>

        {/* Location Status */}
        <section className="rounded-xl border border-border bg-card p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-accent/10 text-accent">
                <MapPin className="h-5 w-5" />
              </div>
              <div>
                <p className="text-sm font-medium">{t('mobileDashboard.currentLocation')}</p>
                <p className="text-xs text-muted-foreground">{t('mobileDashboard.gpsActive')}</p>
              </div>
            </div>
            <StatusBadge variant="valid" size="sm">
              {t('mobileDashboard.online')}
            </StatusBadge>
          </div>
          <div className="mt-3 rounded-lg bg-muted p-3">
            <p className="text-sm font-medium">Highway A1, KM 42</p>
            <p className="text-xs text-muted-foreground">48.8566° N, 2.3522° E</p>
          </div>
        </section>

        {/* Recent Controls */}
        <section>
          <div className="mb-3 flex items-center justify-between">
            <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
              {t('mobileDashboard.recentControls')}
            </h2>
            <Link to="/mobile/history" className="text-sm text-accent">
              {t('mobileDashboard.viewAll')}
            </Link>
          </div>

          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
            </div>
          ) : recentControls.length > 0 ? (
            <div className="space-y-2">
              {recentControls.map((control) => (
                <Link
                  key={control.id}
                  to={`/mobile/vehicle/${encodeURIComponent(control.plateNumber)}`}
                >
                  <RecentControlItem
                    plate={control.plateNumber}
                    time={formatTimeAgo(control.timestamp)}
                    status={control.status as 'valid' | 'warning' | 'critical'}
                  />
                </Link>
              ))}
            </div>
          ) : (
            <div className="rounded-lg border border-dashed border-border p-6 text-center">
              <ClipboardCheck className="mx-auto h-8 w-8 text-muted-foreground/50" />
              <p className="mt-2 text-sm text-muted-foreground">
                {t('mobileDashboard.noControls')}
              </p>
            </div>
          )}
        </section>
      </div>
    </MobileLayout>
  );
}

function QuickActionButton({
  icon: Icon,
  label,
  href,
  primary,
}: {
  icon: React.ElementType;
  label: string;
  href: string;
  primary?: boolean;
}) {
  return (
    <Link to={href}>
      <div
        className={`flex flex-col items-center justify-center gap-2 rounded-xl p-4 transition-all duration-200 active:scale-95 touch-target ${
          primary
            ? 'bg-accent text-accent-foreground shadow-lg'
            : 'bg-card border border-border hover:bg-muted'
        }`}
      >
        <Icon className="h-6 w-6" />
        <span className="text-xs font-medium">{label}</span>
      </div>
    </Link>
  );
}

function RecentControlItem({
  plate,
  time,
  status,
}: {
  plate: string;
  time: string;
  status: 'valid' | 'warning' | 'critical';
}) {
  return (
    <div className="flex items-center justify-between rounded-lg border border-border bg-card p-3 hover:bg-muted transition-colors">
      <div className="flex items-center gap-3">
        <div
          className={`h-2 w-2 rounded-full ${
            status === 'valid'
              ? 'bg-status-valid'
              : status === 'warning'
                ? 'bg-status-warning'
                : 'bg-status-critical'
          }`}
        />
        <div>
          <p className="font-mono font-semibold tracking-wider">{plate}</p>
          <div className="flex items-center gap-1 text-xs text-muted-foreground">
            <Clock className="h-3 w-3" />
            <span>{time}</span>
          </div>
        </div>
      </div>
      <ArrowRight className="h-4 w-4 text-muted-foreground" />
    </div>
  );
}
