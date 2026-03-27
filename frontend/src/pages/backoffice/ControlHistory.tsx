import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { BackOfficeLayout } from '@/components/layout/BackOfficeLayout';
import { StatusBadge } from '@/components/ui/status-badge';
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
  Eye,
  Calendar,
  ChevronLeft,
  ChevronRight,
  SlidersHorizontal,
} from 'lucide-react';
import { useSearchParams } from 'react-router-dom';

import { useQuery } from '@tanstack/react-query';
import { fetchWithAuth } from '@/services/api/backendFetch';
import { useOrganizations } from '@/hooks/api/useOrganizations';
import type {
  ListControlResponse,
  PagedControlsResponse,
  Status,
} from '@/openapi-rq/requests/types.gen';

export default function ControlHistory() {
  const { t } = useTranslation();
  const [searchParams] = useSearchParams();
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState('all');
  const [organizationFilter, setOrganizationFilter] = useState('all');
  const [startDate, setStartDate] = useState<string>('');
  const [endDate, setEndDate] = useState<string>('');
  const [page, setPage] = useState(1);
  const [selectedControl, setSelectedControl] = useState<ListControlResponse | null>(null);
  const [isViewOpen, setIsViewOpen] = useState(false);
  const pageSize = 10;

  const { organizations } = useOrganizations();

  const orgNameById = useMemo(() => {
    const map = new Map<string, string>();
    (organizations ?? []).forEach((o) => map.set(o.id, o.name));
    return map;
  }, [organizations]);

  useEffect(() => {
    const status = searchParams.get('status');
    if (status === 'alerts') {
      setStatusFilter('all');
    }
  }, [searchParams]);

  useEffect(() => {
    setPage(1);
  }, [searchQuery, statusFilter, organizationFilter, startDate, endDate]);

  const statusParam: Status | undefined = useMemo(() => {
    if (statusFilter === 'all') return;
    if (statusFilter === 'valid') return 'valid' as Status;
    if (statusFilter === 'warning') return 'warning' as Status;
    if (statusFilter === 'critical') return 'critical' as Status;
    return;
  }, [statusFilter]);

  const { data, isLoading, isFetching, error, refetch } = useQuery({
    queryKey: [
      'controls-paged',
      page,
      pageSize,
      statusFilter,
      organizationFilter,
      searchQuery,
      startDate,
      endDate,
    ],
    queryFn: async (): Promise<PagedControlsResponse> => {
      const qs = new URLSearchParams();
      qs.set('page', String(page));
      qs.set('page_size', String(pageSize));

      if (organizationFilter !== 'all') {
        qs.set('organization_id', organizationFilter);
      }

      if (statusParam) {
        qs.set('status', String(statusParam));
      }

      if (searchQuery.trim()) {
        qs.set('q', searchQuery.trim());
      }

      if (startDate) {
        qs.set('start_date', `${startDate} 00:00:00`);
      }

      if (endDate) {
        qs.set('end_date', `${endDate} 23:59:59`);
      }

      const res = await fetchWithAuth(`/api/v1/admin/controls/paged?${qs.toString()}`);
      if (!res.ok) {
        const body = await res.text();
        throw new Error(body || `Failed to fetch controls: ${res.status}`);
      }
      return (await res.json()) as PagedControlsResponse;
    },
    refetchInterval: 5000,
    refetchOnWindowFocus: true,
  });

  const controls: ListControlResponse[] = data?.items ?? [];
  const total = data?.total ?? 0;
  const totalPages = Math.max(1, Math.ceil(total / pageSize));

  useEffect(() => {
    if (page > totalPages) {
      setPage(totalPages);
    }
  }, [page, totalPages]);

  const showingFrom = total === 0 ? 0 : (page - 1) * pageSize + 1;
  const showingTo = Math.min(total, page * pageSize);

  const isAnyFilterActive =
    searchQuery.trim() ||
    statusFilter !== 'all' ||
    organizationFilter !== 'all' ||
    startDate ||
    endDate;

  const clearFilters = () => {
    setSearchQuery('');
    setStatusFilter('all');
    setOrganizationFilter('all');
    setStartDate('');
    setEndDate('');
    setPage(1);
  };

  const validCount = controls.filter((c) => c.status === 'valid').length;
  const warningCount = controls.filter((c) => c.status === 'warning').length;
  const criticalCount = controls.filter((c) => c.status === 'critical').length;

  const openView = (control: ListControlResponse) => {
    setSelectedControl(control);
    setIsViewOpen(true);
  };

  return (
    <BackOfficeLayout
      title={t('backOfficeControlHistory.title')}
      subtitle={t('backOfficeControlHistory.subtitle')}
    >
      <Dialog
        open={isViewOpen}
        onOpenChange={(open) => {
          setIsViewOpen(open);
          if (!open) setSelectedControl(null);
        }}
      >
        <DialogContent className="max-w-3xl rounded-2xl p-0">
          <div className="rounded-2xl border border-border/60 bg-gradient-to-br from-card to-muted/20">
            <div className="border-b border-border/60 px-6 py-5">
              <DialogHeader className="space-y-2">
                <DialogTitle className="flex flex-wrap items-center gap-3">
                  <span className="font-mono text-xl font-bold tracking-widest">
                    {selectedControl?.plate_number ?? ''}
                  </span>
                  {selectedControl ? (
                    <StatusBadge variant={selectedControl.status} size="sm">
                      {t(`backOfficeControlHistory.${selectedControl.status}`)}
                    </StatusBadge>
                  ) : null}
                </DialogTitle>
                <DialogDescription className="flex flex-wrap items-center gap-x-4 gap-y-1">
                  <span>
                    {selectedControl?.timestamp
                      ? new Date(selectedControl.timestamp).toLocaleString()
                      : ''}
                  </span>
                  {selectedControl?.agent_name ? (
                    <span className="text-muted-foreground">• {selectedControl.agent_name}</span>
                  ) : null}
                  {selectedControl ? (
                    <span className="text-muted-foreground">
                      •{' '}
                      {orgNameById.get(selectedControl.organization_id) ??
                        selectedControl.organization_id}
                    </span>
                  ) : null}
                </DialogDescription>
              </DialogHeader>
            </div>

            {selectedControl ? (
              <div className="max-h-[70vh] overflow-auto px-6 py-5">
                <div className="grid gap-4 md:grid-cols-2">
                  <div className="rounded-2xl border border-border/60 bg-background/70 p-4">
                    <h3 className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                      Control
                    </h3>
                    <div className="mt-3 space-y-2 text-sm">
                      <div className="flex items-center justify-between gap-3">
                        <span className="text-muted-foreground">Mode</span>
                        <span className="font-medium">{selectedControl.identification_mode}</span>
                      </div>
                      <div className="flex items-center justify-between gap-3">
                        <span className="text-muted-foreground">Confidence</span>
                        <span className="font-medium">
                          {typeof selectedControl.confidence === 'number'
                            ? `${Math.round(selectedControl.confidence * 100)}%`
                            : '—'}
                        </span>
                      </div>
                      <div className="flex items-center justify-between gap-3">
                        <span className="text-muted-foreground">Notes</span>
                        <span className="max-w-[220px] truncate font-medium">
                          {selectedControl.notes ?? '—'}
                        </span>
                      </div>
                    </div>
                  </div>

                  <div className="rounded-2xl border border-border/60 bg-background/70 p-4">
                    <h3 className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                      Location
                    </h3>
                    <div className="mt-3 space-y-2 text-sm">
                      <div className="flex items-start justify-between gap-3">
                        <span className="text-muted-foreground">Address</span>
                        <span className="max-w-[260px] text-right font-medium">
                          {selectedControl.location?.address ?? '—'}
                        </span>
                      </div>
                      <div className="flex items-center justify-between gap-3">
                        <span className="text-muted-foreground">Lat / Lng</span>
                        <span className="font-medium">
                          {selectedControl.location?.latitude ?? '—'}
                          {selectedControl.location?.longitude
                            ? `, ${selectedControl.location.longitude}`
                            : ''}
                        </span>
                      </div>
                    </div>
                  </div>

                  <div className="rounded-2xl border border-border/60 bg-background/70 p-4 md:col-span-2">
                    <div className="flex items-center justify-between">
                      <h3 className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                        Results
                      </h3>
                      <StatusBadge variant={selectedControl.status} size="sm" showIcon={false}>
                        {selectedControl.status}
                      </StatusBadge>
                    </div>

                    <div className="mt-4 grid gap-2 sm:grid-cols-2 lg:grid-cols-5">
                      <div className="rounded-xl border border-border/60 bg-card p-3">
                        <p className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
                          Registration
                        </p>
                        <div className="mt-2">
                          <StatusBadge variant={selectedControl.results.registration} size="sm">
                            {selectedControl.results.registration}
                          </StatusBadge>
                        </div>
                      </div>
                      <div className="rounded-xl border border-border/60 bg-card p-3">
                        <p className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
                          Insurance
                        </p>
                        <div className="mt-2">
                          <StatusBadge variant={selectedControl.results.insurance} size="sm">
                            {selectedControl.results.insurance}
                          </StatusBadge>
                        </div>
                      </div>
                      <div className="rounded-xl border border-border/60 bg-card p-3">
                        <p className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
                          Technical
                        </p>
                        <div className="mt-2">
                          <StatusBadge
                            variant={selectedControl.results.technical_inspection}
                            size="sm"
                          >
                            {selectedControl.results.technical_inspection}
                          </StatusBadge>
                        </div>
                      </div>
                      <div className="rounded-xl border border-border/60 bg-card p-3">
                        <p className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
                          Police
                        </p>
                        <div className="mt-2">
                          <StatusBadge variant={selectedControl.results.wanted_status} size="sm">
                            {selectedControl.results.wanted_status}
                          </StatusBadge>
                        </div>
                      </div>
                      <div className="rounded-xl border border-border/60 bg-card p-3">
                        <p className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
                          Customs
                        </p>
                        <div className="mt-2">
                          <StatusBadge variant={selectedControl.results.customs_status} size="sm">
                            {selectedControl.results.customs_status}
                          </StatusBadge>
                        </div>
                      </div>
                    </div>

                    {selectedControl.notes ? (
                      <div className="mt-4 rounded-xl border border-border/60 bg-card p-4">
                        <p className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
                          Notes
                        </p>
                        <p className="mt-2 text-sm leading-relaxed">{selectedControl.notes}</p>
                      </div>
                    ) : null}
                  </div>
                </div>
              </div>
            ) : null}
          </div>
        </DialogContent>
      </Dialog>

      <div className="space-y-6">
        {/* ── Toolbar ── */}
        <div className="rounded-2xl border border-border/60 bg-card p-4 shadow-md">
          <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
            {/* Search */}
            <div className="relative w-full lg:w-[420px]">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <input
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder={t('backOfficeControlHistory.searchPlaceholder')}
                className="h-10 w-full rounded-xl border border-border bg-background pl-9 pr-3 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
              />
            </div>

            {/* Filters */}
            <div className="flex flex-wrap items-center gap-2">
              <Select value={statusFilter} onValueChange={setStatusFilter}>
                <SelectTrigger className="h-10 w-[140px] rounded-xl text-sm">
                  <Filter className="mr-1.5 h-3.5 w-3.5 text-muted-foreground" />
                  <SelectValue placeholder="Status" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">{t('backOfficeControlHistory.allStatus')}</SelectItem>
                  <SelectItem value="valid">{t('backOfficeControlHistory.valid')}</SelectItem>
                  <SelectItem value="warning">{t('backOfficeControlHistory.warning')}</SelectItem>
                  <SelectItem value="critical">{t('backOfficeControlHistory.critical')}</SelectItem>
                </SelectContent>
              </Select>

              <Select value={organizationFilter} onValueChange={setOrganizationFilter}>
                <SelectTrigger className="h-10 w-[190px] rounded-xl text-sm">
                  <SelectValue placeholder="Organization" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">
                    {t('backOfficeControlHistory.allOrganizations')}
                  </SelectItem>
                  {(organizations ?? []).map((o) => (
                    <SelectItem key={o.id} value={o.id}>
                      {o.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>

              <div className="flex items-center gap-2 rounded-xl border border-border bg-background px-3 py-2">
                <Calendar className="h-3.5 w-3.5 text-muted-foreground" />
                <span className="text-xs text-muted-foreground">From</span>
                <input
                  value={startDate}
                  onChange={(e) => setStartDate(e.target.value)}
                  type="date"
                  className="h-6 bg-transparent text-sm text-foreground outline-none"
                />
                <span className="text-sm text-muted-foreground">-</span>
                <span className="text-xs text-muted-foreground">To</span>
                <input
                  value={endDate}
                  onChange={(e) => setEndDate(e.target.value)}
                  type="date"
                  className="h-6 bg-transparent text-sm text-foreground outline-none"
                />
              </div>

              <Button variant="outline" size="sm" className="h-10 gap-1.5 rounded-xl text-sm">
                <SlidersHorizontal className="h-3.5 w-3.5" />
                {t('backOfficeControlHistory.moreFilters')}
              </Button>

              {isAnyFilterActive ? (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={clearFilters}
                  className="h-10 rounded-xl text-sm"
                >
                  Clear
                </Button>
              ) : null}
            </div>
          </div>
        </div>

        {/* ── Summary pills ── */}
        <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <p className="text-sm text-muted-foreground">
            {t('backOfficeControlHistory.showingControls', {
              count: total,
            })}
            {total > 0 ? <span className="ml-2">{`${showingFrom}-${showingTo}`}</span> : null}
          </p>
          <div className="flex flex-wrap gap-2">
            <Button
              variant={statusFilter === 'valid' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setStatusFilter('valid')}
              className="h-8 rounded-full px-3 text-xs"
            >
              <StatusBadge variant="valid" size="sm">
                {t('backOfficeControlHistory.validCount', { count: validCount })}
              </StatusBadge>
            </Button>
            <Button
              variant={statusFilter === 'warning' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setStatusFilter('warning')}
              className="h-8 rounded-full px-3 text-xs"
            >
              <StatusBadge variant="warning" size="sm">
                {t('backOfficeControlHistory.warningCount', { count: warningCount })}
              </StatusBadge>
            </Button>
            <Button
              variant={statusFilter === 'critical' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setStatusFilter('critical')}
              className="h-8 rounded-full px-3 text-xs"
            >
              <StatusBadge variant="critical" size="sm">
                {t('backOfficeControlHistory.criticalCount', { count: criticalCount })}
              </StatusBadge>
            </Button>
            {statusFilter !== 'all' ? (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setStatusFilter('all')}
                className="h-8 rounded-full px-3 text-xs text-muted-foreground"
              >
                Reset status
              </Button>
            ) : null}
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
                  {t('backOfficeControlHistory.plateNumber')}
                </TableHead>
                <TableHead className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  {t('backOfficeControlHistory.status')}
                </TableHead>
                <TableHead className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  {t('backOfficeControlHistory.agent')}
                </TableHead>
                <TableHead className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  {t('backOfficeControlHistory.organization')}
                </TableHead>
                <TableHead className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  {t('backOfficeControlHistory.location')}
                </TableHead>
                <TableHead className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  {t('backOfficeControlHistory.dateTime')}
                </TableHead>
                <TableHead className="w-[60px]" />
              </TableRow>
            </TableHeader>
            <TableBody>
              {error ? (
                <TableRow>
                  <TableCell colSpan={8} className="h-32 text-center text-muted-foreground">
                    Failed to load controls
                  </TableCell>
                </TableRow>
              ) : isLoading ? (
                Array.from({ length: 5 }).map((_, i) => (
                  <TableRow key={i} className="border-border/40">
                    {Array.from({ length: 8 }).map((_, j) => (
                      <TableCell key={j}>
                        <div className="h-4 animate-pulse rounded-lg bg-muted" />
                      </TableCell>
                    ))}
                  </TableRow>
                ))
              ) : controls.length > 0 ? (
                controls.map((control, idx) => (
                  <TableRow
                    key={control.id}
                    className="group cursor-pointer border-border/40 transition-colors hover:bg-muted/40"
                  >
                    <TableCell>
                      <span className="text-xs font-semibold text-muted-foreground">
                        {(page - 1) * pageSize + idx + 1}
                      </span>
                    </TableCell>
                    <TableCell>
                      <span className="font-mono text-sm font-bold tracking-widest">
                        {control.plate_number}
                      </span>
                    </TableCell>
                    <TableCell>
                      <StatusBadge variant={control.status} size="sm">
                        {t(`backOfficeControlHistory.${control.status}`)}
                      </StatusBadge>
                    </TableCell>
                    <TableCell className="text-sm font-medium">{control.agent_name}</TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {orgNameById.get(control.organization_id) ?? control.organization_id}
                    </TableCell>
                    <TableCell className="max-w-[180px] truncate text-sm text-muted-foreground">
                      {control.location?.address
                        ? control.location.address
                        : control.location?.latitude != null && control.location?.longitude != null
                          ? `${control.location.latitude.toFixed(5)}, ${control.location.longitude.toFixed(5)}`
                          : '-'}
                    </TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {new Date(control.timestamp).toLocaleString([], {
                        month: 'short',
                        day: 'numeric',
                        hour: '2-digit',
                        minute: '2-digit',
                      })}
                    </TableCell>
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-7 w-7 rounded-lg text-muted-foreground hover:text-foreground"
                        onClick={() => openView(control)}
                      >
                        <Eye className="h-3.5 w-3.5" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))
              ) : (
                <TableRow>
                  <TableCell colSpan={8} className="h-32 text-center text-muted-foreground">
                    {t('backOfficeControlHistory.noControlsFound')}
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </div>

        {/* ── Pagination ── */}
        <div className="flex items-center justify-between">
          <p className="text-sm text-muted-foreground">
            {t('backOfficeControlHistory.pageOf', { currentPage: page, totalPages })}
          </p>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={page <= 1}
              onClick={() => setPage((p) => Math.max(1, p - 1))}
              className="h-8 gap-1.5 rounded-xl text-xs"
            >
              <ChevronLeft className="h-3.5 w-3.5" />
              {t('backOfficeControlHistory.previous')}
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={page >= totalPages}
              onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
              className="h-8 gap-1.5 rounded-xl text-xs"
            >
              {t('backOfficeControlHistory.next')}
              <ChevronRight className="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>
      </div>
    </BackOfficeLayout>
  );
}
