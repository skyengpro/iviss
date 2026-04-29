import {
  Area,
  AreaChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
  CartesianGrid,
  TooltipProps,
} from 'recharts';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { ControlActivityPoint, DashboardRange } from '@/openapi-rq/requests/types.gen';
import { Activity } from 'lucide-react';
import { useTranslation } from 'react-i18next';

interface ControlActivityChartProps {
  data: ControlActivityPoint[];
  range: DashboardRange;
  onRangeChange: (range: DashboardRange) => void;
  loading?: boolean;
}

function CustomTooltip({ active, payload, label }: TooltipProps<number, string>) {
  const { t } = useTranslation();
  if (active && payload && payload.length) {
    return (
      <div className="min-w-[140px] rounded-xl border border-border bg-background p-3 shadow-xl">
        <p className="mb-1 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
          {label}
        </p>
        <p className="text-2xl font-bold text-foreground">{payload[0].value}</p>
        <p className="text-[10px] text-muted-foreground">{t('common.controls')}</p>
      </div>
    );
  }
  return null;
}

export function ControlActivityChart({
  data,
  range,
  onRangeChange,
  loading,
}: ControlActivityChartProps) {
  const { t } = useTranslation();
  const maxCount = data.length > 0 ? Math.max(...data.map((d) => d.count)) : 0;

  const rangeLabel =
    range === '24h'
      ? t('common.last24h')
      : range === '7d'
        ? t('common.last7d')
        : t('common.last30d');

  return (
    <Card className="col-span-1 lg:col-span-3 overflow-hidden rounded-2xl border border-border/60 bg-card shadow-md">
      <CardHeader className="pb-0 pt-6 px-6">
        <CardTitle className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-accent/10">
              <Activity className="h-5 w-5 text-accent" />
            </div>
            <div>
              <p className="text-sm font-bold text-foreground">
                {t('backOfficeDashboard.controlActivity24h')}
              </p>
              <p className="text-[10px] text-muted-foreground uppercase tracking-widest">
                {rangeLabel}
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Select value={range} onValueChange={(v) => onRangeChange(v as DashboardRange)}>
              <SelectTrigger className="h-8 w-[140px] rounded-xl">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="24h">{t('common.last24h')}</SelectItem>
                <SelectItem value="7d">{t('common.last7d')}</SelectItem>
                <SelectItem value="30d">{t('common.last30d')}</SelectItem>
              </SelectContent>
            </Select>

            <div className="flex items-center gap-2 rounded-full border border-status-valid/20 bg-status-valid/10 px-3 py-1.5">
              <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-status-valid" />
              <span className="text-[10px] font-semibold uppercase tracking-widest text-status-valid">
                {t('common.live')}
              </span>
            </div>
          </div>
        </CardTitle>
      </CardHeader>
      <CardContent className="px-2 pb-4 pt-6 h-[280px]">
        {loading ? (
          <div className="flex h-full items-center justify-center">
            <p className="text-sm text-muted-foreground">{t('common.loadingActivity')}</p>
          </div>
        ) : data.length === 0 ? (
          <div className="flex h-full items-center justify-center">
            <p className="text-sm text-muted-foreground">{t('common.noActivityData')}</p>
          </div>
        ) : (
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={data} margin={{ top: 10, right: 20, left: -10, bottom: 0 }}>
              <defs>
                <linearGradient id="colorCountFill" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="hsl(var(--accent))" stopOpacity={0.5} />
                  <stop offset="100%" stopColor="hsl(var(--accent))" stopOpacity={0} />
                </linearGradient>
                <filter id="glow">
                  <feGaussianBlur stdDeviation="3" result="coloredBlur" />
                  <feMerge>
                    <feMergeNode in="coloredBlur" />
                    <feMergeNode in="SourceGraphic" />
                  </feMerge>
                </filter>
              </defs>
              <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="hsl(var(--border))" />
              <XAxis
                dataKey="label"
                axisLine={false}
                tickLine={false}
                tick={{
                  fontSize: 10,
                  fill: 'hsl(var(--muted-foreground))',
                  fontFamily: 'Inter, sans-serif',
                }}
                minTickGap={40}
              />
              <YAxis
                axisLine={false}
                tickLine={false}
                tick={{
                  fontSize: 10,
                  fill: 'hsl(var(--muted-foreground))',
                  fontFamily: 'Inter, sans-serif',
                }}
                allowDecimals={false}
                domain={[0, maxCount + 1]}
              />
              <Tooltip
                content={<CustomTooltip />}
                cursor={{ stroke: 'rgba(255,255,255,0.08)', strokeWidth: 1 }}
              />
              <Area
                type="monotoneX"
                dataKey="count"
                stroke="hsl(var(--accent))"
                strokeWidth={2.5}
                fillOpacity={1}
                fill="url(#colorCountFill)"
                animationDuration={1200}
                animationEasing="ease-out"
                filter="url(#glow)"
                dot={false}
                activeDot={{
                  r: 5,
                  fill: 'hsl(186,72%,55%)',
                  stroke: 'hsl(222,47%,15%)',
                  strokeWidth: 2,
                }}
              />
            </AreaChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
}
