import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { BackOfficeLayout } from '@/components/layout/BackOfficeLayout';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Search,
  Filter,
  Calendar,
  ChevronLeft,
  ChevronRight,
  Download,
  Eye,
  Shield,
  ShieldAlert,
  UserCheck,
  UserMinus,
  UserPlus,
  LogIn,
  LogOut,
  RefreshCw,
  KeyRound,
  ArrowRightLeft,
} from 'lucide-react';
import { useQuery } from '@tanstack/react-query';
import { fetchWithAuth } from '@/services/api/backendFetch';

// ── Types ───────────────────────────────────────────────────────────────

interface AuditLogEntry {
  id: string;
  userId: string | null;
  userName: string | null;
  action: string;
  resourceType: string | null;
  resourceId: string | null;
  ipAddress: string | null;
  metadata: Record<string, unknown> | null;
  beforeSnapshot: Record<string, unknown> | null;
  afterSnapshot: Record<string, unknown> | null;
  createdAt: string;
}

interface AuditLogListResponse {
  items: AuditLogEntry[];
  total: number;
  page: number;
  pageSize: number;
}

// ── Action Config ────────────────────────────────────────────────────────

const ACTION_CONFIG: Record<
  string,
  { icon: React.ElementType; color: string; bg: string }
> = {
  LOGIN_SUCCESS: {
    icon: LogIn,
    color: 'text-emerald-400',
    bg: 'bg-emerald-500/10',
  },
  LOGIN_FAILED: {
    icon: ShieldAlert,
    color: 'text-red-400',
    bg: 'bg-red-500/10',
  },
  LOGOUT: {
    icon: LogOut,
    color: 'text-slate-400',
    bg: 'bg-slate-500/10',
  },
  TOKEN_REFRESHED: {
    icon: RefreshCw,
    color: 'text-blue-400',
    bg: 'bg-blue-500/10',
  },
  OTP_REQUESTED: {
    icon: KeyRound,
    color: 'text-amber-400',
    bg: 'bg-amber-500/10',
  },
  OTP_VERIFIED: {
    icon: UserCheck,
    color: 'text-emerald-400',
    bg: 'bg-emerald-500/10',
  },
  OTP_FAILED: {
    icon: ShieldAlert,
    color: 'text-red-400',
    bg: 'bg-red-500/10',
  },
  DEVICE_REGISTERED: {
    icon: Shield,
    color: 'text-blue-400',
    bg: 'bg-blue-500/10',
  },
  DEVICE_REVOKED: {
    icon: ShieldAlert,
    color: 'text-red-400',
    bg: 'bg-red-500/10',
  },
  DEVICE_SUSPENDED: {
    icon: ShieldAlert,
    color: 'text-orange-400',
    bg: 'bg-orange-500/10',
  },
  USER_CREATED: {
    icon: UserPlus,
    color: 'text-emerald-400',
    bg: 'bg-emerald-500/10',
  },
  USER_UPDATED: {
    icon: ArrowRightLeft,
    color: 'text-blue-400',
    bg: 'bg-blue-500/10',
  },
  USER_SUSPENDED: {
    icon: UserMinus,
    color: 'text-orange-400',
    bg: 'bg-orange-500/10',
  },
  USER_ACTIVATED: {
    icon: UserCheck,
    color: 'text-emerald-400',
    bg: 'bg-emerald-500/10',
  },
  USER_DELETED: {
    icon: UserMinus,
    color: 'text-red-400',
    bg: 'bg-red-500/10',
  },
  SESSION_TERMINATED: {
    icon: ShieldAlert,
    color: 'text-red-400',
    bg: 'bg-red-500/10',
  },
  SESSION_RESTARTED: {
    icon: RefreshCw,
    color: 'text-blue-400',
    bg: 'bg-blue-500/10',
  },
  ACTIVATION_CODE_RESENT: {
    icon: KeyRound,
    color: 'text-amber-400',
    bg: 'bg-amber-500/10',
  },
  VEHICLE_SEARCHED: {
    icon: Search,
    color: 'text-sky-400',
    bg: 'bg-sky-500/10',
  },
  VEHICLE_NOT_FOUND: {
    icon: Search,
    color: 'text-orange-400',
    bg: 'bg-orange-500/10',
  },
  PENDING_SUBMISSION_CREATED: {
    icon: Shield,
    color: 'text-sky-400',
    bg: 'bg-sky-500/10',
  },
  PENDING_SUBMISSION_REVIEWED: {
    icon: UserCheck,
    color: 'text-emerald-400',
    bg: 'bg-emerald-500/10',
  },
};

const FILTER_ACTIONS = [
  'LOGIN_SUCCESS',
  'LOGIN_FAILED',
  'USER_CREATED',
  'USER_UPDATED',
  'USER_DELETED',
  'SESSION_TERMINATED',
  'SESSION_RESTARTED',
  'ACTIVATION_CODE_RESENT',
  'DEVICE_REGISTERED',
  'DEVICE_REVOKED',
];

function ActionBadge({ action }: { action: string }) {
  const { t } = useTranslation();
  const config = ACTION_CONFIG[action] || {
    icon: Shield,
    color: 'text-muted-foreground',
    bg: 'bg-muted',
  };
  const Icon = config.icon;

  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-semibold ${config.bg} ${config.color}`}
    >
      <Icon className="h-3 w-3" />
      {t(`backOfficeAuditLogs.actions.${action}`, { defaultValue: action })}
    </span>
  );
}

// ── JSON Diff View ───────────────────────────────────────────────────────

function JsonSnapshot({
  label,
  data,
  variant,
}: {
  label: string;
  data: Record<string, unknown> | null;
  variant: 'before' | 'after';
}) {
  if (!data) {
    return (
      <div className="flex-1">
        <p
          className={`mb-2 text-xs font-bold uppercase tracking-wider ${variant === 'before' ? 'text-red-400/70' : 'text-emerald-400/70'}`}
        >
          {label}
        </p>
        <div className="rounded-xl border border-border/40 bg-muted/30 p-4 font-mono text-xs text-muted-foreground">
          No data
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 min-w-0">
      <p
        className={`mb-2 text-xs font-bold uppercase tracking-wider ${variant === 'before' ? 'text-red-400/70' : 'text-emerald-400/70'}`}
      >
        {label}
      </p>
      <div
        className={`overflow-auto rounded-xl border p-4 font-mono text-xs leading-relaxed ${
          variant === 'before'
            ? 'border-red-500/20 bg-red-500/5'
            : 'border-emerald-500/20 bg-emerald-500/5'
        }`}
        style={{ maxHeight: '320px' }}
      >
        <pre className="whitespace-pre-wrap break-all">{JSON.stringify(data, null, 2)}</pre>
      </div>
    </div>
  );
}

// ── Main Page ────────────────────────────────────────────────────────────

export default function AuditLogs() {
  const { t } = useTranslation();
  const [userIdFilter, setUserIdFilter] = useState('');
  const [actionFilter, setActionFilter] = useState('all');
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');
  const [page, setPage] = useState(1);
  const [selectedLog, setSelectedLog] = useState<AuditLogEntry | null>(null);
  const [isDetailOpen, setIsDetailOpen] = useState(false);
  const pageSize = 15;

  useEffect(() => {
    setPage(1);
  }, [userIdFilter, actionFilter, startDate, endDate]);

  const { data, isLoading, error } = useQuery({
    queryKey: ['audit-logs', page, pageSize, userIdFilter, actionFilter, startDate, endDate],
    queryFn: async (): Promise<AuditLogListResponse> => {
      const qs = new URLSearchParams();
      qs.set('page', String(page));
      qs.set('page_size', String(pageSize));

      if (userIdFilter) qs.set('user_id', userIdFilter);
      if (actionFilter !== 'all') qs.set('action', actionFilter);
      if (startDate) qs.set('start_date', startDate);
      if (endDate) qs.set('end_date', endDate);

      const res = await fetchWithAuth(`/api/v1/admin/audit-logs?${qs.toString()}`);
      if (!res.ok) {
        const body = await res.text();
        throw new Error(body || `Failed to fetch audit logs: ${res.status}`);
      }
      return (await res.json()) as AuditLogListResponse;
    },
    refetchOnWindowFocus: false,
  });

  const logs: AuditLogEntry[] = data?.items ?? [];
  const total = data?.total ?? 0;
  const totalPages = Math.max(1, Math.ceil(total / pageSize));

  useEffect(() => {
    if (page > totalPages) setPage(totalPages);
  }, [page, totalPages]);

  const showingFrom = total === 0 ? 0 : (page - 1) * pageSize + 1;
  const showingTo = Math.min(total, page * pageSize);

  const isAnyFilterActive = userIdFilter || actionFilter !== 'all' || startDate || endDate;

  const clearFilters = () => {
    setUserIdFilter('');
    setActionFilter('all');
    setStartDate('');
    setEndDate('');
    setPage(1);
  };

  const handleExport = async () => {
    const qs = new URLSearchParams();
    if (userIdFilter) qs.set('user_id', userIdFilter);
    if (actionFilter !== 'all') qs.set('action', actionFilter);
    if (startDate) qs.set('start_date', startDate);
    if (endDate) qs.set('end_date', endDate);

    const res = await fetchWithAuth(`/api/v1/admin/audit-logs/export?${qs.toString()}`);
    if (!res.ok) return;

    const blob = await res.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `audit_logs_${new Date().toISOString().slice(0, 10)}.csv`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const openDetail = (entry: AuditLogEntry) => {
    setSelectedLog(entry);
    setIsDetailOpen(true);
  };

  const actionCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    logs.forEach((l) => {
      counts[l.action] = (counts[l.action] || 0) + 1;
    });
    return counts;
  }, [logs]);

  return (
    <BackOfficeLayout
      title={t('backOfficeAuditLogs.title')}
      subtitle={t('backOfficeAuditLogs.subtitle')}
    >
      {/* Detail Dialog */}
      <Dialog
        open={isDetailOpen}
        onOpenChange={(open) => {
          setIsDetailOpen(open);
          if (!open) setSelectedLog(null);
        }}
      >
        <DialogContent className="max-w-4xl rounded-2xl p-0">
          <div className="rounded-2xl border border-border/60 bg-gradient-to-br from-card to-muted/20">
            <div className="border-b border-border/60 px-6 py-5">
              <DialogHeader className="space-y-2">
                <DialogTitle className="flex flex-wrap items-center gap-3">
                  <span className="text-lg font-bold">
                    {t('backOfficeAuditLogs.logDetail')}
                  </span>
                  {selectedLog && <ActionBadge action={selectedLog.action} />}
                </DialogTitle>
                <DialogDescription className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm">
                  {selectedLog?.createdAt && (
                    <span>{new Date(selectedLog.createdAt).toLocaleString()}</span>
                  )}
                  {selectedLog?.userName && (
                    <span className="text-muted-foreground">• {selectedLog.userName}</span>
                  )}
                  {selectedLog?.ipAddress && (
                    <span className="font-mono text-muted-foreground">
                      • {selectedLog.ipAddress}
                    </span>
                  )}
                </DialogDescription>
              </DialogHeader>
            </div>

            {selectedLog && (
              <div className="max-h-[70vh] overflow-auto px-6 py-5">
                <div className="space-y-5">
                  {/* Meta */}
                  <div className="grid gap-3 sm:grid-cols-3">
                    <div className="rounded-xl border border-border/50 bg-background/70 p-3">
                      <p className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground">
                        {t('backOfficeAuditLogs.userId')}
                      </p>
                      <p className="mt-1 font-mono text-sm">
                        {selectedLog.userId?.slice(0, 8) ?? '—'}…
                      </p>
                    </div>
                    <div className="rounded-xl border border-border/50 bg-background/70 p-3">
                      <p className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground">
                        {t('backOfficeAuditLogs.resourceType')}
                      </p>
                      <p className="mt-1 text-sm font-medium">
                        {selectedLog.resourceType ?? '—'}
                      </p>
                    </div>
                    <div className="rounded-xl border border-border/50 bg-background/70 p-3">
                      <p className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground">
                        {t('backOfficeAuditLogs.resourceId')}
                      </p>
                      <p className="mt-1 font-mono text-sm">
                        {selectedLog.resourceId?.slice(0, 8) ?? '—'}…
                      </p>
                    </div>
                  </div>

                  {/* Metadata */}
                  {selectedLog.metadata &&
                    Object.keys(selectedLog.metadata).length > 0 && (
                      <div className="rounded-xl border border-border/50 bg-background/70 p-4">
                        <p className="mb-2 text-[10px] font-bold uppercase tracking-widest text-muted-foreground">
                          {t('backOfficeAuditLogs.metadata')}
                        </p>
                        <pre className="overflow-auto font-mono text-xs leading-relaxed text-foreground/80">
                          {JSON.stringify(selectedLog.metadata, null, 2)}
                        </pre>
                      </div>
                    )}

                  {/* Before / After Snapshots */}
                  {(selectedLog.beforeSnapshot || selectedLog.afterSnapshot) && (
                    <div>
                      <p className="mb-3 text-xs font-bold uppercase tracking-widest text-muted-foreground">
                        {t('backOfficeAuditLogs.dataChanges')}
                      </p>
                      <div className="flex flex-col gap-4 md:flex-row">
                        <JsonSnapshot
                          label={t('backOfficeAuditLogs.before')}
                          data={selectedLog.beforeSnapshot}
                          variant="before"
                        />
                        <JsonSnapshot
                          label={t('backOfficeAuditLogs.after')}
                          data={selectedLog.afterSnapshot}
                          variant="after"
                        />
                      </div>
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
        </DialogContent>
      </Dialog>

      <div className="space-y-6">
        {/* ── Toolbar ── */}
        <div className="rounded-2xl border border-border/60 bg-card p-4 shadow-md">
          <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
            {/* Filters */}
            <div className="flex flex-wrap items-center gap-2">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
                <input
                  type="text"
                  placeholder={t('backOfficeAuditLogs.userIdPlaceholder')}
                  value={userIdFilter}
                  onChange={(e) => setUserIdFilter(e.target.value)}
                  className="h-10 w-[240px] rounded-xl border border-border/80 bg-background pl-9 pr-3 text-sm outline-none ring-primary/20 transition-all focus:border-primary focus:ring-4"
                  id="audit-user-id-filter"
                />
              </div>

              <Select value={actionFilter} onValueChange={setActionFilter}>
                <SelectTrigger className="h-10 w-[200px] rounded-xl text-sm" id="audit-action-filter">
                  <Filter className="mr-1.5 h-3.5 w-3.5 text-muted-foreground" />
                  <SelectValue placeholder={t('backOfficeAuditLogs.actionType')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">{t('backOfficeAuditLogs.allActions')}</SelectItem>
                  {FILTER_ACTIONS.map((a) => (
                    <SelectItem key={a} value={a}>
                      {t(`backOfficeAuditLogs.actions.${a}`, { defaultValue: a })}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>

              <div className="flex items-center gap-2 rounded-xl border border-border bg-background px-3 py-2">
                <Calendar className="h-3.5 w-3.5 text-muted-foreground" />
                <span className="text-xs text-muted-foreground">
                  {t('backOfficeAuditLogs.from')}
                </span>
                <input
                  value={startDate}
                  onChange={(e) => setStartDate(e.target.value)}
                  type="date"
                  id="audit-start-date"
                  className="h-6 bg-transparent text-sm text-foreground outline-none"
                />
                <span className="text-sm text-muted-foreground">-</span>
                <span className="text-xs text-muted-foreground">
                  {t('backOfficeAuditLogs.to')}
                </span>
                <input
                  value={endDate}
                  onChange={(e) => setEndDate(e.target.value)}
                  type="date"
                  id="audit-end-date"
                  className="h-6 bg-transparent text-sm text-foreground outline-none"
                />
              </div>

              {isAnyFilterActive && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={clearFilters}
                  className="h-10 rounded-xl text-sm"
                  id="audit-clear-filters"
                >
                  {t('backOfficeAuditLogs.clearFilters')}
                </Button>
              )}
            </div>

            {/* Export */}
            <Button
              variant="outline"
              size="sm"
              onClick={handleExport}
              className="h-10 gap-2 rounded-xl text-sm"
              id="audit-export-csv"
            >
              <Download className="h-3.5 w-3.5" />
              {t('backOfficeAuditLogs.exportCsv')}
            </Button>
          </div>
        </div>

        {/* ── Summary ── */}
        <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <p className="text-sm text-muted-foreground">
            {t('backOfficeAuditLogs.showingLogs', { count: total })}
            {total > 0 && <span className="ml-2">{`${showingFrom}–${showingTo}`}</span>}
          </p>
          <div className="flex flex-wrap gap-1.5">
            {Object.entries(actionCounts)
              .slice(0, 5)
              .map(([action, count]) => (
                <button
                  key={action}
                  onClick={() => setActionFilter(action)}
                  className={`inline-flex items-center gap-1 rounded-full px-2.5 py-1 text-[11px] font-medium transition-colors ${
                    actionFilter === action
                      ? 'bg-primary text-primary-foreground'
                      : 'bg-muted/60 text-muted-foreground hover:bg-muted'
                  }`}
                >
                  {t(`backOfficeAuditLogs.actions.${action}`, { defaultValue: action })}
                  <span className="font-bold">({count})</span>
                </button>
              ))}
          </div>
        </div>

        {/* ── Table ── */}
        <div className="overflow-hidden rounded-2xl border border-border/60 bg-card shadow-sm">
          <Table>
            <TableHeader>
              <TableRow className="border-border/60 bg-muted/40 hover:bg-muted/40">
                <TableHead className="w-[60px] text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  #
                </TableHead>
                <TableHead className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  {t('backOfficeAuditLogs.timestamp')}
                </TableHead>
                <TableHead className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  {t('backOfficeAuditLogs.user')}
                </TableHead>
                <TableHead className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  {t('backOfficeAuditLogs.action')}
                </TableHead>
                <TableHead className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  {t('backOfficeAuditLogs.resource')}
                </TableHead>
                <TableHead className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  {t('backOfficeAuditLogs.ipAddress')}
                </TableHead>
                <TableHead className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  {t('backOfficeAuditLogs.changes')}
                </TableHead>
                <TableHead className="w-[60px]" />
              </TableRow>
            </TableHeader>
            <TableBody>
              {error ? (
                <TableRow>
                  <TableCell colSpan={8} className="h-32 text-center text-muted-foreground">
                    {t('backOfficeAuditLogs.errorLoading')}
                  </TableCell>
                </TableRow>
              ) : isLoading ? (
                Array.from({ length: 6 }).map((_, i) => (
                  <TableRow key={i} className="border-border/40">
                    {Array.from({ length: 8 }).map((_, j) => (
                      <TableCell key={j}>
                        <div className="h-4 animate-pulse rounded-lg bg-muted" />
                      </TableCell>
                    ))}
                  </TableRow>
                ))
              ) : logs.length > 0 ? (
                logs.map((entry, idx) => (
                  <TableRow
                    key={entry.id}
                    className="group cursor-pointer border-border/40 transition-colors hover:bg-muted/40"
                    onClick={() => openDetail(entry)}
                  >
                    <TableCell>
                      <span className="text-xs font-semibold text-muted-foreground">
                        {(page - 1) * pageSize + idx + 1}
                      </span>
                    </TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {new Date(entry.createdAt).toLocaleString([], {
                        month: 'short',
                        day: 'numeric',
                        hour: '2-digit',
                        minute: '2-digit',
                        second: '2-digit',
                      })}
                    </TableCell>
                    <TableCell className="text-sm font-medium">
                      {entry.userName ?? (
                        <span className="text-muted-foreground italic">System</span>
                      )}
                    </TableCell>
                    <TableCell>
                      <ActionBadge action={entry.action} />
                    </TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {entry.resourceType ? (
                        <span className="rounded-md bg-muted/60 px-1.5 py-0.5 font-mono text-xs">
                          {entry.resourceType}
                        </span>
                      ) : (
                        '—'
                      )}
                    </TableCell>
                    <TableCell className="font-mono text-xs text-muted-foreground">
                      {entry.ipAddress ?? '—'}
                    </TableCell>
                    <TableCell>
                      {entry.beforeSnapshot || entry.afterSnapshot ? (
                        <span className="inline-flex items-center gap-1 rounded-full bg-indigo-500/10 px-2 py-0.5 text-[10px] font-semibold text-indigo-400">
                          <ArrowRightLeft className="h-2.5 w-2.5" />
                          {t('backOfficeAuditLogs.hasChanges')}
                        </span>
                      ) : (
                        <span className="text-xs text-muted-foreground/50">—</span>
                      )}
                    </TableCell>
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-7 w-7 rounded-lg text-muted-foreground hover:text-foreground"
                        onClick={(e) => {
                          e.stopPropagation();
                          openDetail(entry);
                        }}
                      >
                        <Eye className="h-3.5 w-3.5" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))
              ) : (
                <TableRow>
                  <TableCell colSpan={8} className="h-32 text-center text-muted-foreground">
                    {t('backOfficeAuditLogs.noLogsFound')}
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </div>

        {/* ── Pagination ── */}
        <div className="flex items-center justify-between">
          <p className="text-sm text-muted-foreground">
            {t('backOfficeAuditLogs.pageOf', { currentPage: page, totalPages })}
          </p>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={page <= 1}
              onClick={() => setPage((p) => Math.max(1, p - 1))}
              className="h-8 gap-1.5 rounded-xl text-xs"
              id="audit-prev-page"
            >
              <ChevronLeft className="h-3.5 w-3.5" />
              {t('backOfficeAuditLogs.previous')}
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={page >= totalPages}
              onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
              className="h-8 gap-1.5 rounded-xl text-xs"
              id="audit-next-page"
            >
              {t('backOfficeAuditLogs.next')}
              <ChevronRight className="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>
      </div>
    </BackOfficeLayout>
  );
}
