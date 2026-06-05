import { ChevronDown, Globe } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { useTranslation } from 'react-i18next';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

interface BackOfficeHeaderProps {
  title?: string;
  subtitle?: string;
  actions?: React.ReactNode;
  className?: string;
}

export function BackOfficeHeader({ title, subtitle, actions, className }: BackOfficeHeaderProps) {
  const { t, i18n } = useTranslation();

  return (
    <header
      className={cn(
        'sticky top-0 z-30 flex h-14 items-center gap-4 border-b border-border bg-card/95 px-5 backdrop-blur-sm',
        className
      )}
    >
      {/* Left: Title */}
      <div className="min-w-0 flex-1">
        {title && (
          <h1 className="truncate text-base font-semibold leading-none text-foreground">{title}</h1>
        )}
        {subtitle && <p className="mt-0.5 truncate text-xs text-muted-foreground">{subtitle}</p>}
      </div>

      {/* Right: compact action zone */}
      <div className="flex shrink-0 items-center gap-1.5">
        {/* Language */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon" className="h-8 w-8 rounded-lg">
              <Globe className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={() => i18n.changeLanguage('en')}>English</DropdownMenuItem>
            <DropdownMenuItem onClick={() => i18n.changeLanguage('fr')}>Français</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>

        {/* Custom page actions */}
        {actions && <div className="ml-1 flex items-center gap-1.5">{actions}</div>}

        {/* Org selector */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant="outline"
              size="sm"
              className="hidden h-8 gap-1.5 rounded-lg text-xs md:flex"
            >
              <span className="max-w-[120px] truncate">
                {t('backOfficeHeader.nationalPoliceHQ')}
              </span>
              <ChevronDown className="h-3 w-3 shrink-0" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem>National Police HQ</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </header>
  );
}
