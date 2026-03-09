import { useState, useEffect } from 'react';
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
import { Link } from 'react-router-dom';

import { cn } from '@/lib/utils';
import { useAuth } from '@/hooks/auth/use-auth';
import { useDashboard } from '@/hooks/api/useDashboard';
import { useControls } from '@/hooks/api/useControls';
import { useGeolocation } from '@/hooks/useGeolocation';

export default function MobileDashboard() {
  const { user } = useAuth();
  const { t } = useTranslation();

  const { stats, isLoading: statsLoading } = useDashboard();
  const { lat, lng, error: geoError, loading: geoLoading } = useGeolocation();
  const [address, setAddress] = useState<string>('');
  const [isReverseGeocoding, setIsReverseGeocoding] = useState(false);

  useEffect(() => {
    const fetchAddress = async () => {
      if (lat && lng) {
        setIsReverseGeocoding(true);
        try {
          const response = await fetch(
            `https://nominatim.openstreetmap.org/reverse?format=json&lat=${lat}&lon=${lng}&zoom=18&addressdetails=1`
          );
          const data = await response.json();
          setAddress(data.display_name || '');
        } catch (error) {
          console.error('Error fetching address:', error);
        } finally {
          setIsReverseGeocoding(false);
        }
      }
    };

    fetchAddress();
  }, [lat, lng]);

  const { controls: recentControls = [], isLoading: controlsLoading } = useControls({
    query: {
      agent_id: user?.id,
    },
  });

  const startOfDay = new Date();
  startOfDay.setHours(0, 0, 0, 0);

  const todayControlsCount = (recentControls || []).filter(
    (c) => new Date(c.timestamp).getTime() >= startOfDay.getTime()
  ).length;

  const todayAlertsCount = (recentControls || []).filter(
    (c) =>
      new Date(c.timestamp).getTime() >= startOfDay.getTime() &&
      (c.status === 'critical' || c.status === 'warning')
  ).length;

  const isLoading = statsLoading || controlsLoading;

  const formatTimeAgo = (dateString: string) => {
    const date = new Date(dateString);
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
              value={isLoading ? '-' : String(todayControlsCount)}
              subtitle={t('mobileDashboard.today')}
              icon={ClipboardCheck}
              variant="gradient"
            />
            <StatCard
              title={t('mobileDashboard.alerts')}
              value={isLoading ? '-' : String(todayAlertsCount)}
              subtitle={t('mobileDashboard.flaggedVehicles')}
              icon={AlertTriangle}
              variant={todayAlertsCount > 0 ? 'critical' : 'default'}
            />
          </div>
        </section>

        {/* Location Status */}
        <section className="rounded-xl border border-border bg-card p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div
                className={cn(
                  'flex h-10 w-10 items-center justify-center rounded-lg transition-colors',
                  geoError ? 'bg-destructive/10 text-destructive' : 'bg-accent/10 text-accent'
                )}
              >
                <MapPin className="h-5 w-5" />
              </div>
              <div>
                <p className="text-sm font-medium">{t('mobileDashboard.currentLocation')}</p>
                <p className="text-xs text-muted-foreground">
                  {geoLoading
                    ? t('mobileDashboard.locationRequesting')
                    : geoError
                      ? t('mobileDashboard.locationError')
                      : t('mobileDashboard.gpsActive')}
                </p>
              </div>
            </div>
            <StatusBadge variant={geoError ? 'critical' : 'valid'} size="sm">
              {geoError ? t('mobileDashboard.locationDenied') : t('mobileDashboard.online')}
            </StatusBadge>
          </div>
          <div className="mt-3 rounded-lg bg-muted p-3">
            {geoLoading ? (
              <p className="text-sm animate-pulse">{t('mobileDashboard.locationRequesting')}</p>
            ) : geoError ? (
              <p className="text-sm text-destructive font-medium">{geoError}</p>
            ) : (
              <>
                <p className="text-sm font-medium break-words">
                  {isReverseGeocoding ? t('mobileDashboard.searchingLocation') : address || '...'}
                </p>
                {lat && lng && (
                  <p className="text-xs text-muted-foreground mt-1">
                    {lat.toFixed(4)}° {lat >= 0 ? 'N' : 'S'}, {lng.toFixed(4)}°{' '}
                    {lng >= 0 ? 'E' : 'W'}
                  </p>
                )}
              </>
            )}
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
                <Link key={control.id} to={`/mobile/history/${control.id}`} state={{ control }}>
                  <RecentControlItem
                    plate={control.plate_number}
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
