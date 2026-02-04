import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { BackOfficeLayout } from '@/components/layout/BackOfficeLayout';
import { StatusBadge } from '@/components/ui/status-badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
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
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Search,
  Filter,
  Download,
  Eye,
  Calendar,
  RefreshCw,
  ChevronLeft,
  ChevronRight,
} from 'lucide-react';

import { useQuery } from '@tanstack/react-query';
import { mockControlService, ControlStatus } from '@/services/mockControls';

export default function ControlHistory() {
  const { t } = useTranslation();
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState('all');
  const [organizationFilter, setOrganizationFilter] = useState('all');

  const { data: controls = [], isLoading } = useQuery({
    queryKey: ['controls', 'all', statusFilter, organizationFilter],
    queryFn: () =>
      mockControlService.getAllControls({
        status: statusFilter !== 'all' ? (statusFilter as ControlStatus) : undefined,
        organizationId: organizationFilter !== 'all' ? organizationFilter : undefined,
      }),
  });

  const filteredControls = controls.filter((control) => {
    return (
      control.plateNumber.toLowerCase().includes(searchQuery.toLowerCase()) ||
      control.agentName.toLowerCase().includes(searchQuery.toLowerCase()) ||
      control.location.address.toLowerCase().includes(searchQuery.toLowerCase())
    );
  });

  return (
    <BackOfficeLayout
      title={t('backOfficeControlHistory.title')}
      subtitle={t('backOfficeControlHistory.subtitle')}
      actions={
        <div className="flex gap-2">
          <Button variant="outline" className="gap-2">
            <RefreshCw className="h-4 w-4" />
            {t('backOfficeControlHistory.refresh')}
          </Button>
          <Button className="gap-2 bg-accent text-accent-foreground hover:bg-accent/90">
            <Download className="h-4 w-4" />
            {t('backOfficeControlHistory.export')}
          </Button>
        </div>
      }
    >
      <Card>
        <CardHeader>
          <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
            {/* Search */}
            <div className="relative w-full lg:w-96">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder={t('backOfficeControlHistory.searchPlaceholder')}
                className="pl-9"
              />
            </div>

            {/* Filters */}
            <div className="flex flex-wrap gap-2">
              <Select value={statusFilter} onValueChange={setStatusFilter}>
                <SelectTrigger className="w-[140px]">
                  <SelectValue placeholder={t('backOfficeControlHistory.status')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">{t('backOfficeControlHistory.allStatus')}</SelectItem>
                  <SelectItem value="valid">{t('backOfficeControlHistory.valid')}</SelectItem>
                  <SelectItem value="warning">{t('backOfficeControlHistory.warning')}</SelectItem>
                  <SelectItem value="critical">{t('backOfficeControlHistory.critical')}</SelectItem>
                </SelectContent>
              </Select>

              <Select value={organizationFilter} onValueChange={setOrganizationFilter}>
                <SelectTrigger className="w-[160px]">
                  <SelectValue placeholder={t('backOfficeControlHistory.organization')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">
                    {t('backOfficeControlHistory.allOrganizations')}
                  </SelectItem>
                  <SelectItem value="alpha">
                    {t('backOfficeControlHistory.brigadeAlpha')}
                  </SelectItem>
                  <SelectItem value="beta">{t('backOfficeControlHistory.brigadeBeta')}</SelectItem>
                  <SelectItem value="gamma">
                    {t('backOfficeControlHistory.brigadeGamma')}
                  </SelectItem>
                </SelectContent>
              </Select>

              <Button variant="outline" className="gap-2">
                <Calendar className="h-4 w-4" />
                {t('backOfficeControlHistory.dateRange')}
              </Button>

              <Button variant="outline" className="gap-2">
                <Filter className="h-4 w-4" />
                {t('backOfficeControlHistory.moreFilters')}
              </Button>
            </div>
          </div>
        </CardHeader>

        <CardContent>
          {/* Results summary */}
          <div className="mb-4 flex items-center justify-between">
            <p className="text-sm text-muted-foreground">
              {t('backOfficeControlHistory.showingControls', { count: filteredControls.length })}
            </p>
            <div className="flex gap-2">
              <StatusBadge variant="valid" size="sm">
                {t('backOfficeControlHistory.validCount', {
                  count: filteredControls.filter((c) => c.status === 'valid').length,
                })}
              </StatusBadge>
              <StatusBadge variant="warning" size="sm">
                {t('backOfficeControlHistory.warningCount', {
                  count: filteredControls.filter((c) => c.status === 'warning').length,
                })}
              </StatusBadge>
              <StatusBadge variant="critical" size="sm">
                {t('backOfficeControlHistory.criticalCount', {
                  count: filteredControls.filter((c) => c.status === 'critical').length,
                })}
              </StatusBadge>
            </div>
          </div>

          {/* Table */}
          <div className="rounded-lg border">
            <Table>
              <TableHeader>
                <TableRow className="bg-muted/50">
                  <TableHead className="w-[100px]">{t('backOfficeControlHistory.id')}</TableHead>
                  <TableHead>{t('backOfficeControlHistory.plateNumber')}</TableHead>
                  <TableHead>{t('backOfficeControlHistory.vehicle')}</TableHead>
                  <TableHead>{t('backOfficeControlHistory.status')}</TableHead>
                  <TableHead>{t('backOfficeControlHistory.agent')}</TableHead>
                  <TableHead>{t('backOfficeControlHistory.organization')}</TableHead>
                  <TableHead>{t('backOfficeControlHistory.location')}</TableHead>
                  <TableHead>{t('backOfficeControlHistory.dateTime')}</TableHead>
                  <TableHead className="w-[80px]">
                    {t('backOfficeControlHistory.actions')}
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {isLoading ? (
                  <TableRow>
                    <TableCell colSpan={9} className="h-24 text-center">
                      <div className="flex items-center justify-center gap-2">
                        <RefreshCw className="h-4 w-4 animate-spin" />
                        {t('backOfficeControlHistory.loadingControls')}
                      </div>
                    </TableCell>
                  </TableRow>
                ) : filteredControls.length > 0 ? (
                  filteredControls.map((control) => (
                    <TableRow key={control.id} className="group">
                      <TableCell className="font-mono text-sm">{control.id}</TableCell>
                      <TableCell>
                        <span className="font-mono font-semibold tracking-wider">
                          {control.plateNumber}
                        </span>
                      </TableCell>
                      <TableCell>{t('backOfficeControlHistory.vehicle')}</TableCell>
                      <TableCell>
                        <StatusBadge variant={control.status} size="sm">
                          {t(`backOfficeControlHistory.${control.status}`)}
                        </StatusBadge>
                      </TableCell>
                      <TableCell>{control.agentName}</TableCell>
                      <TableCell>{control.organizationName}</TableCell>
                      <TableCell className="max-w-[200px] truncate">
                        {control.location.address}
                      </TableCell>
                      <TableCell className="text-sm text-muted-foreground">
                        {control.timestamp.toLocaleString()}
                      </TableCell>
                      <TableCell>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="opacity-0 group-hover:opacity-100"
                        >
                          <Eye className="h-4 w-4" />
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))
                ) : (
                  <TableRow>
                    <TableCell colSpan={9} className="h-24 text-center text-muted-foreground">
                      {t('backOfficeControlHistory.noControlsFound')}
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </div>

          {/* Pagination */}
          <div className="mt-4 flex items-center justify-between">
            <p className="text-sm text-muted-foreground">
              {t('backOfficeControlHistory.pageOf', { currentPage: 1, totalPages: 129 })}
            </p>
            <div className="flex gap-2">
              <Button variant="outline" size="sm" disabled>
                <ChevronLeft className="h-4 w-4" />
                {t('backOfficeControlHistory.previous')}
              </Button>
              <Button variant="outline" size="sm">
                {t('backOfficeControlHistory.next')}
                <ChevronRight className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </BackOfficeLayout>
  );
}
