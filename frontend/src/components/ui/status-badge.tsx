import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '@/lib/utils';
import { CheckCircle, AlertTriangle, XCircle, Clock, Shield } from 'lucide-react';

const statusBadgeVariants = cva(
  'inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-xs font-semibold uppercase tracking-wide transition-all duration-200',
  {
    variants: {
      variant: {
        valid: 'bg-status-valid text-status-valid-foreground shadow-sm',
        warning: 'bg-status-warning text-status-warning-foreground shadow-sm',
        critical: 'bg-status-critical text-status-critical-foreground shadow-sm',
        pending: 'bg-status-pending text-status-pending-foreground shadow-sm',
        neutral: 'bg-secondary text-secondary-foreground',
      },
      size: {
        sm: 'px-2 py-0.5 text-[10px]',
        default: 'px-3 py-1 text-xs',
        lg: 'px-4 py-1.5 text-sm',
      },
    },
    defaultVariants: {
      variant: 'neutral',
      size: 'default',
    },
  }
);

interface StatusBadgeProps extends VariantProps<typeof statusBadgeVariants> {
  children: React.ReactNode;
  className?: string;
  showIcon?: boolean;
}

const statusIcons = {
  valid: CheckCircle,
  warning: AlertTriangle,
  critical: XCircle,
  pending: Clock,
  neutral: Shield,
};

export function StatusBadge({
  variant = 'neutral',
  size,
  children,
  className,
  showIcon = true,
}: StatusBadgeProps) {
  const Icon = statusIcons[variant || 'neutral'];

  return (
    <span className={cn(statusBadgeVariants({ variant, size }), className)}>
      {showIcon && <Icon className="h-3 w-3" />}
      {children}
    </span>
  );
}
