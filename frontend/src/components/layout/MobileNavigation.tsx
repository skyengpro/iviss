import { Link, useLocation } from 'react-router-dom';
import { Home, Camera, Search, ClipboardList, Settings } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useTranslation } from 'react-i18next';

export function MobileNavigation() {
  const location = useLocation();
  const { t } = useTranslation();

  const navItems = [
    { href: '/mobile', icon: Home, label: t('mobileNav.home') },
    { href: '/mobile/scan', icon: Camera, label: t('mobileNav.scan') },
    { href: '/mobile/search', icon: Search, label: t('mobileNav.search') },
    { href: '/mobile/history', icon: ClipboardList, label: t('mobileNav.history') },
    { href: '/mobile/settings', icon: Settings, label: t('mobileNav.settings') },
  ];

  return (
    <nav className="fixed bottom-0 left-0 right-0 z-50 border-t border-border bg-card/95 backdrop-blur-lg">
      <div className="flex h-16 items-center justify-around px-2">
        {navItems.map((item) => {
          const isActive =
            item.href === '/mobile'
              ? location.pathname === item.href
              : location.pathname.startsWith(item.href);
          return (
            <Link
              key={item.href}
              to={item.href}
              className={cn(
                'flex flex-1 flex-col items-center justify-center gap-1 py-2 transition-all duration-200 touch-target',
                isActive ? 'text-accent' : 'text-muted-foreground hover:text-foreground'
              )}
            >
              <div
                className={cn(
                  'flex h-8 w-8 items-center justify-center rounded-lg transition-all duration-200',
                  isActive && 'bg-accent/10'
                )}
              >
                <item.icon
                  className={cn(
                    'h-5 w-5 transition-transform duration-200',
                    isActive && 'scale-110'
                  )}
                />
              </div>
              <span className="text-[10px] font-medium">{item.label}</span>
            </Link>
          );
        })}
      </div>
      {/* Safe area padding for iOS */}
      <div className="h-safe-area-bottom bg-card" />
    </nav>
  );
}
