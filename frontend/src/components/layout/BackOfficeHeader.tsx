import { Bell, Search, ChevronDown, Globe } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { cn } from '@/lib/utils';
import { useTranslation } from 'react-i18next';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

interface BackOfficeHeaderProps {
  readonly title?: string;
  readonly subtitle?: string;
  readonly actions?: React.ReactNode;
  readonly className?: string;
}

export function BackOfficeHeader({ title, subtitle, actions, className }: BackOfficeHeaderProps) {
  const { t, i18n } = useTranslation();
  return (
    <header
      className={cn(
        'sticky top-0 z-30 flex h-16 items-center justify-between border-b border-border bg-card/95 px-6 backdrop-blur-sm',
        className
      )}
    >
      {/* Left: Title */}
      <div>
        {title && <h1 className="text-xl font-semibold text-foreground">{title}</h1>}
        {subtitle && <p className="text-sm text-muted-foreground">{subtitle}</p>}
      </div>

      {/* Right: Search and actions */}
      <div className="flex items-center gap-4">
        {/* Global search */}
        <div className="relative hidden lg:block">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input placeholder={t('backOfficeHeader.searchPlaceholder')} className="w-80 pl-9" />
        </div>

        {/* Language Switcher */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon">
              <Globe className="h-5 w-5" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={() => i18n.changeLanguage('en')}>English</DropdownMenuItem>
            <DropdownMenuItem onClick={() => i18n.changeLanguage('fr')}>Français</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>

        {/* Notifications */}
        <Button variant="ghost" size="icon" className="relative">
          <Bell className="h-5 w-5" />
          <span className="absolute right-1.5 top-1.5 h-2 w-2 rounded-full bg-status-critical" />
        </Button>

        {/* Custom actions */}
        {actions}

        {/* Organization selector */}
        <Button variant="outline" className="hidden md:flex gap-2">
          <span className="max-w-32 truncate">{t('backOfficeHeader.nationalPoliceHQ')}</span>
          <ChevronDown className="h-4 w-4" />
        </Button>
      </div>
    </header>
  );
}
