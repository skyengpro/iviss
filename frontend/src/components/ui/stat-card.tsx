import { cn } from '@/lib/utils';
import { LucideIcon } from 'lucide-react';
import { cva, type VariantProps } from 'class-variance-authority';

const statCardVariants = cva(
  'relative overflow-hidden rounded-2xl p-6 transition-all duration-300 hover:-translate-y-1',
  {
    variants: {
      variant: {
        default: 'bg-white border border-border/60 shadow-sm hover:shadow-md text-foreground',
        primary:
          'bg-gradient-to-br from-[hsl(222,47%,20%)] to-[hsl(222,47%,32%)] text-white shadow-lg hover:shadow-xl',
        accent:
          'bg-gradient-to-br from-[hsl(186,72%,32%)] to-[hsl(186,72%,48%)] text-white shadow-lg hover:shadow-xl',
        valid:
          'bg-gradient-to-br from-[hsl(142,71%,38%)] to-[hsl(142,71%,52%)] text-white shadow-lg hover:shadow-xl',
        warning:
          'bg-gradient-to-br from-[hsl(38,92%,44%)] to-[hsl(38,92%,58%)] text-white shadow-lg hover:shadow-xl',
        critical:
          'bg-gradient-to-br from-[hsl(0,84%,54%)] to-[hsl(0,84%,66%)] text-white shadow-lg hover:shadow-xl',
        gradient:
          'bg-gradient-to-br from-[hsl(222,47%,20%)] to-[hsl(222,47%,32%)] text-white shadow-lg hover:shadow-xl',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  }
);

interface StatCardProps extends VariantProps<typeof statCardVariants> {
  title: string;
  value: string | number;
  subtitle?: string;
  icon?: LucideIcon;
  trend?: {
    value: number;
    isPositive: boolean;
  };
  className?: string;
  loading?: boolean;
}

export function StatCard({
  title,
  value,
  subtitle,
  icon: Icon,
  trend,
  variant,
  className,
  loading,
}: StatCardProps) {
  const isColored = variant && variant !== 'default';

  if (loading) {
    return (
      <div className={cn(statCardVariants({ variant: 'default' }), className)}>
        <div className="space-y-3 animate-pulse">
          <div className="h-4 w-24 rounded-lg bg-muted" />
          <div className="h-9 w-16 rounded-xl bg-muted" />
          <div className="h-3 w-32 rounded-lg bg-muted" />
        </div>
      </div>
    );
  }

  return (
    <div className={cn(statCardVariants({ variant }), className)}>
      {/* Decorative background glow */}
      {isColored && (
        <div className="absolute -right-6 -top-6 h-32 w-32 rounded-full bg-white/10 blur-2xl" />
      )}
      {/* Large faint icon in background */}
      {Icon && (
        <div className="absolute right-4 bottom-4 opacity-[0.07] pointer-events-none">
          <Icon className="h-20 w-20" />
        </div>
      )}

      <div className="relative z-10 flex flex-col gap-4">
        <div className="flex items-start justify-between gap-3">
          <div className="flex flex-col gap-1">
            <p
              className={cn(
                'text-xs font-semibold uppercase tracking-widest',
                isColored ? 'text-white/70' : 'text-muted-foreground'
              )}
            >
              {title}
            </p>
          </div>
          {Icon && (
            <div
              className={cn(
                'flex h-10 w-10 shrink-0 items-center justify-center rounded-xl',
                isColored ? 'bg-white/15 backdrop-blur-sm' : 'bg-primary/10'
              )}
            >
              <Icon className={cn('h-5 w-5', isColored ? 'text-white' : 'text-primary')} />
            </div>
          )}
        </div>

        <div>
          <p
            className={cn(
              'text-4xl font-bold tracking-tight leading-none',
              isColored ? 'text-white' : 'text-foreground'
            )}
          >
            {value}
          </p>
          <div className="mt-2 flex items-center gap-2">
            {trend && (
              <span
                className={cn(
                  'inline-flex items-center gap-0.5 rounded-full px-2 py-0.5 text-xs font-semibold',
                  trend.isPositive
                    ? isColored
                      ? 'bg-white/20 text-white'
                      : 'bg-status-valid/10 text-status-valid'
                    : isColored
                      ? 'bg-black/20 text-white/80'
                      : 'bg-status-critical/10 text-status-critical'
                )}
              >
                {trend.isPositive ? '↑' : '↓'} {Math.abs(trend.value)}%
              </span>
            )}
            {subtitle && (
              <span
                className={cn('text-xs', isColored ? 'text-white/60' : 'text-muted-foreground')}
              >
                {subtitle}
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
