import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { BackOfficeLayout } from '@/components/layout/BackOfficeLayout';
import { StatusBadge } from '@/components/ui/status-badge';
import { Button } from '@/components/ui/button';
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
  Download,
  Eye,
  Calendar,
  RefreshCw,
  ChevronLeft,
  ChevronRight,
  SlidersHorizontal,
} from 'lucide-react';

import { useQuery } from '@tanstack/react-query';
import { mockControlService, ControlStatus } from '@/services/mockControls';

export default function ControlHistory() {
  const { t } = useTranslation();
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState('all');
  const [organizationFilter, setOrganizationFilter] = useState('all');

  const { data: controls = [], isLoading, refetch } = useQuery({
    queryKey: ['controls', 'all', statusFilter, organizationFilter],
    queryFn: () =>
      mockControlService.getAllControls({
        status: statusFilter !== 'all' ? (statusFilter as ControlStatus) : undefined,
        organizationId: organizationFilter !== 'all' ? organizationFilter : undefined,
      }),
  });

  const filteredControls = controls.filter((control) =>
    control.plateNumber.toLowerCase().includes(searchQuery.toLowerCase()) ||
    control.agentName.toLowerCase().includes(searchQuery.toLowerCase()) ||
    control.location.address.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const validCount = filteredControls.filter((c) => c.status === 'valid').length;
  const warningCount = filteredControls.filter((c) => c.status === 'warning').length;
  const criticalCount = filteredControls.filter((c) => c.status === 'critical').length;

  return (
    <BackOfficeLayout
      title={t('backOfficeControlHistory.title')}
      subtitle={t('backOfficeControlHistory.subtitle')}
      actions={
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => refetch()}
            className="h-8 gap-1.5 rounded-lg text-xs"
          >
            <RefreshCw className="h-3.5 w-3.5" />
            {t('backOfficeControlHistory.refresh')}
          </Button>
          <Button
            size="sm"
            className="h-8 gap-1.5 rounded-lg bg-gradient-to-r from-primary to-[hsl(222,47%,32%)] text-xs text-white shadow"
          >
            <Download className="h-3.5 w-3.5" />
            {t('backOfficeControlHistory.export')}
          </Button>
        </div>
      }
    >
      <div className="space-y-4">
        {/* ── Toolbar ── */}
        <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          {/* Search */}
          <div className="relative w-full sm:w-80">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <input
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder={t('backOfficeControlHistory.searchPlaceholder')}
              className="h-9 w-full rounded-xl border border-border bg-background pl-9 pr-3 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
            />
          </div>

          {/* Filters */}
          <div className="flex flex-wrap items-center gap-2">
            <Select value={statusFilter} onValueChange={setStatusFilter}>
              <SelectTrigger className="h-9 w-[130px] rounded-xl text-sm">
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
              <SelectTrigger className="h-9 w-[150px] rounded-xl text-sm">
                <SelectValue placeholder="Organization" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">{t('backOfficeControlHistory.allOrganizations')}</SelectItem>
                <SelectItem value="alpha">{t('backOfficeControlHistory.brigadeAlpha')}</SelectItem>
                <SelectItem value="beta">{t('backOfficeControlHistory.brigadeBeta')}</SelectItem>
                <SelectItem value="gamma">{t('backOfficeControlHistory.brigadeGamma')}</SelectItem>
              </SelectContent>
            </Select>

            <Button variant="outline" size="sm" className="h-9 gap-1.5 rounded-xl text-sm">
              <Calendar className="h-3.5 w-3.5" />
              {t('backOfficeControlHistory.dateRange')}
            </Button>

            <Button variant="outline" size="sm" className="h-9 gap-1.5 rounded-xl text-sm">
              <SlidersHorizontal className="h-3.5 w-3.5" />
              {t('backOfficeControlHistory.moreFilters')}
            </Button>
          </div>
        </div>

        {/* ── Summary pills ── */}
        <div className="flex items-center justify-between">
          <p className="text-sm text-muted-foreground">
            {t('backOfficeControlHistory.showingControls', { count: filteredControls.length })}
          </p>
          <div className="flex gap-2">
            <StatusBadge variant="valid" size="sm">
              {t('backOfficeControlHistory.validCount', { count: validCount })}
            </StatusBadge>
            <StatusBadge variant="warning" size="sm">
              {t('backOfficeControlHistory.warningCount', { count: warningCount })}
            </StatusBadge>
            <StatusBadge variant="critical" size="sm">
              {t('backOfficeControlHistory.criticalCount', { count: criticalCount })}
            </StatusBadge>
          </div>
        </div>

        {/* ── Table ── */}
        <div className="overflow-hidden rounded-2xl border border-border/60 bg-card shadow-sm">
          <Table>
            <TableHeader>
              <TableRow className="border-border/60 bg-muted/40 hover:bg-muted/40">
                <TableHead className="w-[100px] text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  {t('backOfficeControlHistory.id')}
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
              {isLoading ? (
                Array.from({ length: 5 }).map((_, i) => (
                  <TableRow key={i} className="border-border/40">
                    {Array.from({ length: 8 }).map((_, j) => (
                      <TableCell key={j}>
                        <div className="h-4 animate-pulse rounded-lg bg-muted" />
                      </TableCell>
                    ))}
                  </TableRow>
                ))
              ) : filteredControls.length > 0 ? (
                filteredControls.map((control) => (
                  <TableRow
                    key={control.id}
                    className="group cursor-pointer border-border/40 transition-colors hover:bg-muted/40"
                  >
                    <TableCell>
                      <span className="font-mono text-xs text-muted-foreground">{control.id}</span>
                    </TableCell>
                    <TableCell>
                      <span className="font-mono text-sm font-bold tracking-widest">
                        {control.plateNumber}
                      </span>
                    </TableCell>
                    <TableCell>
                      <StatusBadge variant={control.status} size="sm">
                        {t(`backOfficeControlHistory.${control.status}`)}
                      </StatusBadge>
                    </TableCell>
                    <TableCell className="text-sm font-medium">{control.agentName}</TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {control.organizationName}
                    </TableCell>
                    <TableCell className="max-w-[180px] truncate text-sm text-muted-foreground">
                      {control.location.address}
                    </TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {control.timestamp.toLocaleString([], {
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
                        className="h-7 w-7 rounded-lg opacity-0 transition-opacity group-hover:opacity-100"
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
            {t('backOfficeControlHistory.pageOf', { currentPage: 1, totalPages: 129 })}
          </p>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" disabled className="h-8 gap-1.5 rounded-xl text-xs">
              <ChevronLeft className="h-3.5 w-3.5" />
              {t('backOfficeControlHistory.previous')}
            </Button>
            <Button variant="outline" size="sm" className="h-8 gap-1.5 rounded-xl text-xs">
              {t('backOfficeControlHistory.next')}
              <ChevronRight className="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>
      </div>
    </BackOfficeLayout>
  );
}
