import { useTranslation } from 'react-i18next';
import { Link, useLocation, useNavigate } from 'react-router-dom';
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
  PanelLeftClose,
  PanelLeftOpen,
} from 'lucide-react';
import { cn } from '@/lib/utils';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { useState } from 'react';
import { useAuth } from '@/hooks/auth/use-auth';
import { useSidebar } from '@/context/SidebarContext';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';

export function BackOfficeSidebar() {
  const { t } = useTranslation();
  const location = useLocation();
  const navigate = useNavigate();
  const [adminOpen, setAdminOpen] = useState(true);
  const { logout, user } = useAuth();
  const { collapsed, toggle } = useSidebar();

  const isAdmin = user?.role === 'admin';

  const mainNavItems = [
    { href: '/backoffice', icon: LayoutDashboard, label: t('backOfficeSidebar.dashboard') },
    {
      href: '/backoffice/controls',
      icon: ClipboardList,
      label: t('backOfficeSidebar.controlHistory'),
    },
    { href: '/backoffice/vehicles', icon: Car, label: t('backOfficeSidebar.vehicleDatabase') },
    {
      href: '/backoffice/validation',
      icon: FileSearch,
      label: t('backOfficeSidebar.pendingValidation'),
    },
  ];

  const adminNavItems = [
    { href: '/backoffice/users', icon: Users, label: t('backOfficeSidebar.userManagement') },
    {
      href: '/backoffice/organizations',
      icon: Building2,
      label: t('backOfficeSidebar.organizations'),
    },
    { href: '/backoffice/audit', icon: Shield, label: t('backOfficeSidebar.auditLogs') },
    { href: '/backoffice/settings', icon: Settings, label: t('backOfficeSidebar.settings') },
  ];

  const handleLogout = async () => {
    await logout();
    navigate('/admin-login');
  };

  return (
    <TooltipProvider delayDuration={0}>
      <aside
        className={cn(
          'fixed left-0 top-0 z-40 flex h-screen flex-col bg-sidebar text-sidebar-foreground transition-all duration-300 ease-in-out',
          collapsed ? 'w-[4.5rem]' : 'w-64'
        )}
      >
        {/* Logo + collapse toggle */}
        <div
          className={cn(
            'flex h-16 items-center border-b border-sidebar-border px-4 transition-all duration-300',
            collapsed ? 'justify-center' : 'justify-between gap-3'
          )}
        >
          {/* Logo mark */}
          <div className="flex shrink-0 items-center gap-3">
            <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-sidebar-primary">
              <Shield className="h-5 w-5 text-sidebar-primary-foreground" />
            </div>
            {!collapsed && (
              <div className="overflow-hidden">
                <p className="whitespace-nowrap font-bold tracking-wide">IVISS</p>
                <p className="whitespace-nowrap text-[10px] uppercase tracking-wider text-sidebar-foreground/60">
                  {t('backOfficeSidebar.backOffice')}
                </p>
              </div>
            )}
          </div>

          {/* Collapse toggle */}
          <button
            onClick={toggle}
            className="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg text-sidebar-foreground/60 transition hover:bg-sidebar-accent hover:text-sidebar-foreground"
            aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          >
            {collapsed ? (
              <PanelLeftOpen className="h-4 w-4" />
            ) : (
              <PanelLeftClose className="h-4 w-4" />
            )}
          </button>
        </div>

        {/* Navigation */}
        <nav className="flex-1 overflow-y-auto overflow-x-hidden p-3 scrollbar-thin">
          {/* Main section */}
          <div className="space-y-0.5">
            {!collapsed && (
              <p className="mb-1.5 px-3 text-[9px] font-bold uppercase tracking-widest text-sidebar-foreground/40">
                {t('backOfficeSidebar.main')}
              </p>
            )}
            {mainNavItems.map((item) => (
              <NavLink
                key={item.href}
                href={item.href}
                icon={item.icon}
                label={item.label}
                isActive={location.pathname === item.href}
                collapsed={collapsed}
              />
            ))}
          </div>

          {/* Admin section — only visible to admin users */}
          {isAdmin && (
          <div className="mt-4">
            {collapsed ? (
              <div className="space-y-0.5">
                {adminNavItems.map((item) => (
                  <NavLink
                    key={item.href}
                    href={item.href}
                    icon={item.icon}
                    label={item.label}
                    isActive={location.pathname === item.href}
                    collapsed={collapsed}
                  />
                ))}
              </div>
            ) : (
              <Collapsible open={adminOpen} onOpenChange={setAdminOpen}>
                <CollapsibleTrigger asChild>
                  <button className="flex w-full items-center justify-between px-3 py-1.5 text-[9px] font-bold uppercase tracking-widest text-sidebar-foreground/40 hover:text-sidebar-foreground/60 transition-colors">
                    {t('backOfficeSidebar.administration')}
                    <ChevronDown
                      className={cn(
                        'h-3.5 w-3.5 transition-transform duration-200',
                        adminOpen && 'rotate-180'
                      )}
                    />
                  </button>
                </CollapsibleTrigger>
                <CollapsibleContent className="space-y-0.5">
                  {adminNavItems.map((item) => (
                    <NavLink
                      key={item.href}
                      href={item.href}
                      icon={item.icon}
                      label={item.label}
                      isActive={location.pathname === item.href}
                      collapsed={collapsed}
                    />
                  ))}
                </CollapsibleContent>
              </Collapsible>
            )}
          </div>
          )}
        </nav>

        {/* User section */}
        <div
          className={cn('border-t border-sidebar-border p-3', collapsed && 'flex justify-center')}
        >
          {collapsed ? (
            <Tooltip>
              <TooltipTrigger asChild>
                <button
                  onClick={handleLogout}
                  className="flex h-9 w-9 items-center justify-center rounded-lg text-sidebar-foreground/60 transition hover:bg-sidebar-accent hover:text-sidebar-foreground"
                >
                  <LogOut className="h-4 w-4" />
                </button>
              </TooltipTrigger>
              <TooltipContent side="right">Log out</TooltipContent>
            </Tooltip>
          ) : (
            <div className="flex items-center gap-3 rounded-xl bg-sidebar-accent p-2.5">
              <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-sidebar-primary text-xs font-bold text-sidebar-primary-foreground">
                AD
              </div>
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-semibold">{t('backOfficeSidebar.adminUser')}</p>
                <p className="truncate text-[10px] text-sidebar-foreground/50">
                  {t('backOfficeSidebar.superAdmin')}
                </p>
              </div>
              <button
                onClick={handleLogout}
                className="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg text-sidebar-foreground/50 transition hover:bg-sidebar-border hover:text-sidebar-foreground"
              >
                <LogOut className="h-3.5 w-3.5" />
              </button>
            </div>
          )}
        </div>
      </aside>
    </TooltipProvider>
  );
}

function NavLink({
  href,
  icon: Icon,
  label,
  isActive,
  collapsed,
}: {
  href: string;
  icon: React.ElementType;
  label: string;
  isActive: boolean;
  collapsed: boolean;
}) {
  const link = (
    <Link
      to={href}
      className={cn(
        'flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm font-medium transition-all duration-150',
        collapsed && 'justify-center px-0',
        isActive
          ? 'bg-sidebar-primary text-sidebar-primary-foreground shadow-sm'
          : 'text-sidebar-foreground/70 hover:bg-sidebar-accent hover:text-sidebar-foreground'
      )}
    >
      <Icon className="h-[18px] w-[18px] shrink-0" />
      {!collapsed && <span className="truncate">{label}</span>}
    </Link>
  );

  if (collapsed) {
    return (
      <TooltipProvider delayDuration={0}>
        <Tooltip>
          <TooltipTrigger asChild>{link}</TooltipTrigger>
          <TooltipContent side="right" className="font-medium">
            {label}
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  }

  return link;
}
