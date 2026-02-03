import { cn } from "@/lib/utils";
import { LucideIcon } from "lucide-react";
import { cva, type VariantProps } from "class-variance-authority";

const statCardVariants = cva(
  "relative overflow-hidden rounded-xl p-6 transition-all duration-300 card-elevated",
  {
    variants: {
      variant: {
        default: "bg-card text-card-foreground border border-border",
        primary: "bg-primary text-primary-foreground",
        accent: "bg-accent text-accent-foreground",
        valid: "bg-status-valid text-status-valid-foreground",
        warning: "bg-status-warning text-status-warning-foreground",
        critical: "bg-status-critical text-status-critical-foreground",
        gradient: "bg-gradient-to-br from-primary to-navy-600 text-primary-foreground",
      },
    },
    defaultVariants: {
      variant: "default",
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
}

export function StatCard({
  title,
  value,
  subtitle,
  icon: Icon,
  trend,
  variant,
  className,
}: StatCardProps) {
  return (
    <div className={cn(statCardVariants({ variant }), className)}>
      {/* Background decoration */}
      <div className="absolute right-0 top-0 h-24 w-24 -translate-y-4 translate-x-4 opacity-10">
        {Icon && <Icon className="h-full w-full" />}
      </div>

      <div className="relative z-10">
        <div className="flex items-center justify-between">
          <p className="text-sm font-medium opacity-80">{title}</p>
          {Icon && (
            <div className="rounded-lg bg-white/10 p-2">
              <Icon className="h-5 w-5" />
            </div>
          )}
        </div>

        <div className="mt-4">
          <p className="text-3xl font-bold tracking-tight">{value}</p>
          
          <div className="mt-2 flex items-center gap-2">
            {trend && (
              <span
                className={cn(
                  "text-xs font-medium",
                  trend.isPositive ? "text-green-400" : "text-red-400"
                )}
              >
                {trend.isPositive ? "↑" : "↓"} {Math.abs(trend.value)}%
              </span>
            )}
            {subtitle && (
              <span className="text-xs opacity-70">{subtitle}</span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
