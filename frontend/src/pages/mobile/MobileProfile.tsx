import { MobileLayout } from '@/components/layout/MobileLayout';
import { Button } from '@/components/ui/button';
import { StatusBadge } from '@/components/ui/status-badge';
import {
  User,
  Shield,
  Smartphone,
  Building2,
  Mail,
  BadgeCheck,
  LogOut,
  ChevronRight,
  HelpCircle,
  FileText,
  Settings,
} from 'lucide-react';
import { useAuth } from '@/hooks/auth/use-auth';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useGetControls } from '@/openapi-rq/queries/queries';
import { useGetUserProfile } from '@/openapi-rq/queries/queries';

export default function MobileProfile() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const { t } = useTranslation();

  const { data: me } = useGetUserProfile({}, undefined, {
    enabled: true,
    staleTime: 0,
    refetchOnMount: 'always',
  });

  const startOfDay = new Date();
  startOfDay.setHours(0, 0, 0, 0);

  const { data: todayControls = [] } = useGetControls(
    {
      query: {
        agent_id: user?.id || null,
        start_date: startOfDay.toISOString(),
        end_date: null,
        status: null,
        plate: null,
      },
    },
    undefined,
    {
      enabled: !!user?.id,
    }
  );

  const handleLogout = async () => {
    await logout();
    navigate('/daily-login');
  };

  const displayUser = me || user;
  if (!displayUser) return null;

  return (
    <MobileLayout title={t('mobileProfile.title')}>
      <div className="p-4 space-y-6">
        {/* Profile Header */}
        <div className="rounded-xl bg-gradient-to-br from-primary to-primary/80 p-6 text-primary-foreground">
          <div className="flex items-center gap-4">
            <div className="flex h-16 w-16 items-center justify-center rounded-full bg-white/20 text-2xl font-bold">
              {displayUser.avatarInitials ? (
                displayUser.avatarInitials
              ) : (
                <User className="h-8 w-8" />
              )}
            </div>
            <div>
              <h2 className="text-xl font-bold">{displayUser.name}</h2>
              <p className="text-sm opacity-80">{displayUser.role.toUpperCase()}</p>
              <StatusBadge variant="valid" size="sm" className="mt-1">
                {t('mobileProfile.activeStatus')}
              </StatusBadge>
            </div>
          </div>
        </div>

        {/* User Info */}
        <section className="rounded-xl border border-border bg-card overflow-hidden">
          <div className="p-4 border-b border-border">
            <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
              {t('mobileProfile.accountInfo')}
            </h3>
          </div>

          <InfoRow
            icon={BadgeCheck}
            label={t('mobileProfile.badgeId')}
            value={displayUser.badgeId || '—'}
          />
          <InfoRow icon={Mail} label="Email" value={displayUser.email || '—'} />
          <InfoRow icon={Shield} label={t('mobileProfile.role')} value={displayUser.role} />
          <InfoRow
            icon={Building2}
            label={t('mobileProfile.organization')}
            value={displayUser.organization}
          />
          <InfoRow icon={Smartphone} label="Phone" value={displayUser.phoneNumber || '—'} />
        </section>

        {/* Quick Links */}
        <section className="rounded-xl border border-border bg-card overflow-hidden">
          <div className="p-4 border-b border-border">
            <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
              {t('mobileProfile.quickActions')}
            </h3>
          </div>

          <MenuLink
            icon={FileText}
            label={t('mobileProfile.myControlsToday')}
            badge={String(todayControls.length)}
            onClick={() => navigate('/mobile/history', { state: { filter: 'today' } })}
          />
          <MenuLink
            icon={HelpCircle}
            label={t('mobileProfile.helpSupport')}
            onClick={() => navigate('/mobile/support')}
          />
          <MenuLink
            icon={Settings}
            label={t('mobileProfile.settings')}
            onClick={() => navigate('/mobile/settings')}
          />
        </section>

        {/* App Info */}
        <div className="text-center text-xs text-muted-foreground">
          <p>{t('mobileProfile.appVersion')}</p>
          <p className="mt-1">{t('mobileProfile.footer')}</p>
        </div>
      </div>
    </MobileLayout>
  );
}

function InfoRow({
  icon: Icon,
  label,
  value,
}: {
  icon: React.ElementType;
  label: string;
  value: string;
}) {
  return (
    <div className="flex items-center gap-3 px-4 py-3 border-b border-border last:border-b-0">
      <Icon className="h-5 w-5 text-muted-foreground" />
      <div className="flex-1 min-w-0">
        <p className="text-xs text-muted-foreground">{label}</p>
        <p className="font-medium truncate">{value}</p>
      </div>
    </div>
  );
}

function MenuLink({
  icon: Icon,
  label,
  badge,
  onClick,
}: {
  icon: React.ElementType;
  label: string;
  badge?: string;
  onClick?: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className="flex w-full items-center gap-3 px-4 py-3 border-b border-border last:border-b-0 hover:bg-muted transition-colors"
    >
      <Icon className="h-5 w-5 text-muted-foreground" />
      <span className="flex-1 text-left">{label}</span>
      {badge && (
        <span className="rounded-full bg-accent px-2 py-0.5 text-xs font-medium text-accent-foreground">
          {badge}
        </span>
      )}
      <ChevronRight className="h-4 w-4 text-muted-foreground" />
    </button>
  );
}
