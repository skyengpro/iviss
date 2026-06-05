import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { X, Shield, User, LogOut, HelpCircle, FileText, ChevronRight } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { useAuth } from '@/hooks/auth/use-auth';

interface MobileSidebarProps {
  open: boolean;
  onClose: () => void;
}

export function MobileSidebar({ open, onClose }: MobileSidebarProps) {
  const { t } = useTranslation();
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  const handleLogout = async () => {
    await logout();
    navigate('/daily-login');
    onClose();
  };

  return (
    <>
      {/* Overlay */}
      <div
        className={cn(
          'fixed inset-0 z-50 bg-black/60 backdrop-blur-sm transition-opacity duration-300',
          open ? 'opacity-100' : 'pointer-events-none opacity-0'
        )}
        onClick={onClose}
      />

      {/* Sidebar panel */}
      <aside
        className={cn(
          'fixed left-0 top-0 z-50 h-full w-80 bg-sidebar text-sidebar-foreground shadow-xl transition-transform duration-300 ease-out',
          open ? 'translate-x-0' : '-translate-x-full'
        )}
      >
        {/* Header */}
        <div className="flex h-16 items-center justify-between border-b border-sidebar-border px-4">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-sidebar-primary">
              <Shield className="h-6 w-6 text-sidebar-primary-foreground" />
            </div>
            <div>
              <p className="font-bold">IVISS</p>
              <p className="text-xs text-sidebar-foreground/70">{t('mobileSidebar.frontOffice')}</p>
            </div>
          </div>
          <Button
            variant="ghost"
            size="icon"
            onClick={onClose}
            className="text-sidebar-foreground hover:bg-sidebar-accent"
          >
            <X className="h-5 w-5" />
          </Button>
        </div>

        {/* User info */}
        <div className="border-b border-sidebar-border p-4">
          <div className="flex items-center gap-3">
            <div className="flex h-12 w-12 items-center justify-center rounded-full bg-sidebar-accent">
              <User className="h-6 w-6" />
            </div>
            <div className="flex-1">
              <p className="font-semibold">{user?.name || '—'}</p>
              <p className="text-sm text-sidebar-foreground/70">{user?.organization || '—'}</p>
            </div>
            <div
              className="h-2 w-2 rounded-full bg-status-valid"
              title={t('mobileSidebar.online')}
            />
          </div>
        </div>

        {/* Menu items */}
        <nav className="flex-1 p-4 space-y-1">
          <SidebarLink
            icon={FileText}
            label={t('mobileProfile.myControlsToday')}
            onClick={() => {
              navigate('/mobile/history', { state: { filter: 'today' } });
              onClose();
            }}
          />
          <SidebarLink
            icon={HelpCircle}
            label={t('mobileProfile.helpSupport')}
            onClick={() => {
              navigate('/mobile/support');
              onClose();
            }}
          />
        </nav>

        <div className="border-t border-sidebar-border p-4">
          <p className="mt-4 text-center text-xs text-sidebar-foreground/50">
            {t('mobileSidebar.footer')}
          </p>
        </div>
      </aside>
    </>
  );
}

function SidebarLink({
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
      className="flex w-full items-center gap-3 rounded-lg px-3 py-3 text-left text-sidebar-foreground transition-colors hover:bg-sidebar-accent"
    >
      <Icon className="h-5 w-5" />
      <span className="flex-1">{label}</span>
      {badge && (
        <span className="rounded-full bg-sidebar-primary px-2 py-0.5 text-xs font-medium text-sidebar-primary-foreground">
          {badge}
        </span>
      )}
      <ChevronRight className="h-4 w-4 opacity-50" />
    </button>
  );
}
