import { useTranslation } from "react-i18next";
import { Link, useLocation, useNavigate } from "react-router-dom";
import {
  LayoutDashboard,
  Users,
  Building2,
  Car,
  ClipboardList,
  FileSearch,
  Settings,
  Shield,
  ChevronDown,
  LogOut,
  Bell,
  HelpCircle,
} from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { useState } from 'react';
import { useAuth } from '@/contexts/AuthContext';

export function BackOfficeSidebar() {
  const { t } = useTranslation();
  const location = useLocation();
  const navigate = useNavigate();
  const [adminOpen, setAdminOpen] = useState(true);
  const { logout } = useAuth();

  const mainNavItems = [
    { href: "/backoffice", icon: LayoutDashboard, label: t('backOfficeSidebar.dashboard') },
    { href: "/backoffice/controls", icon: ClipboardList, label: t('backOfficeSidebar.controlHistory') },
    { href: "/backoffice/vehicles", icon: Car, label: t('backOfficeSidebar.vehicleDatabase') },
    { href: "/backoffice/validation", icon: FileSearch, label: t('backOfficeSidebar.pendingValidation') },
  ];

  const adminNavItems = [
    { href: "/backoffice/users", icon: Users, label: t('backOfficeSidebar.userManagement') },
    { href: "/backoffice/organizations", icon: Building2, label: t('backOfficeSidebar.organizations') },
    { href: "/backoffice/audit", icon: Shield, label: t('backOfficeSidebar.auditLogs') },
    { href: "/backoffice/settings", icon: Settings, label: t('backOfficeSidebar.settings') },
  ];

  const handleLogout = async () => {
    await logout();
    navigate('/login');
  };

  return (
    <aside className="fixed left-0 top-0 z-40 flex h-screen w-64 flex-col bg-sidebar text-sidebar-foreground">
      {/* Logo */}
      <div className="flex h-16 items-center gap-3 border-b border-sidebar-border px-6">
        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-sidebar-primary">
          <Shield className="h-6 w-6 text-sidebar-primary-foreground" />
        </div>
        <div>
          <p className="font-bold tracking-wide">IVISS</p>
          <p className="text-[10px] uppercase tracking-wider text-sidebar-foreground/60">
            {t('backOfficeSidebar.backOffice')}
          </p>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto p-4 scrollbar-thin">
        {/* Main section */}
        <div className="space-y-1">
          <p className="mb-2 px-3 text-[10px] font-semibold uppercase tracking-wider text-sidebar-foreground/50">
            {t('backOfficeSidebar.main')}
          </p>
          {mainNavItems.map((item) => (
            <NavLink
              key={item.href}
              href={item.href}
              icon={item.icon}
              label={item.label}
              isActive={location.pathname === item.href}
            />
          ))}
        </div>

        {/* Admin section */}
        <Collapsible open={adminOpen} onOpenChange={setAdminOpen} className="mt-6">
          <CollapsibleTrigger asChild>
            <button className="flex w-full items-center justify-between px-3 py-2 text-[10px] font-semibold uppercase tracking-wider text-sidebar-foreground/50 hover:text-sidebar-foreground/70">
              {t('backOfficeSidebar.administration')}
              <ChevronDown
                className={cn(
                  'h-4 w-4 transition-transform duration-200',
                  adminOpen && 'rotate-180'
                )}
              />
            </button>
          </CollapsibleTrigger>
          <CollapsibleContent className="space-y-1">
            {adminNavItems.map((item) => (
              <NavLink
                key={item.href}
                href={item.href}
                icon={item.icon}
                label={item.label}
                isActive={location.pathname === item.href}
              />
            ))}
          </CollapsibleContent>
        </Collapsible>
      </nav>

      {/* User section */}
      <div className="border-t border-sidebar-border p-4">
        <div className="flex items-center gap-3 rounded-lg bg-sidebar-accent p-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-full bg-sidebar-primary text-sidebar-primary-foreground">
            <span className="text-sm font-semibold">AD</span>
          </div>
          <div className="flex-1 min-w-0">
            <p className="truncate text-sm font-medium">{t('backOfficeSidebar.adminUser')}</p>
            <p className="truncate text-xs text-sidebar-foreground/60">{t('backOfficeSidebar.superAdmin')}</p>
          </div>
        </div>
        <div className="mt-3 flex gap-2">
          <Button
            variant="ghost"
            size="icon"
            className="flex-1 text-sidebar-foreground hover:bg-sidebar-accent"
          >
            <Bell className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="flex-1 text-sidebar-foreground hover:bg-sidebar-accent"
          >
            <HelpCircle className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={handleLogout}
            className="flex-1 text-sidebar-foreground hover:bg-sidebar-accent"
          >
            <LogOut className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </aside>
  );
}

function NavLink({
  href,
  icon: Icon,
  label,
  isActive,
}: {
  href: string;
  icon: React.ElementType;
  label: string;
  isActive: boolean;
}) {
  return (
    <Link
      to={href}
      className={cn(
        'flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-all duration-200',
        isActive
          ? 'bg-sidebar-primary text-sidebar-primary-foreground'
          : 'text-sidebar-foreground/80 hover:bg-sidebar-accent hover:text-sidebar-foreground'
      )}
    >
      <Icon className="h-5 w-5" />
      <span>{label}</span>
    </Link>
  );
}
