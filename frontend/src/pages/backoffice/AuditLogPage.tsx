import { useTranslation } from 'react-i18next';
import { BackOfficeLayout } from '@/components/layout/BackOfficeLayout';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Search, Download, Shield, User, Clock, Activity, ArrowUpDown } from 'lucide-react';
import { useState, useMemo } from 'react';
import { useListAuditLogs, useExportAuditLogs } from '@/openapi-rq/queries/queries';
import { format } from 'date-fns';
import { StatusBadge } from '@/components/ui/status-badge';

export default function AuditLogPage() {
  const { t } = useTranslation();
  const [searchTerm, setSearchTerm] = useState('');
  const [page, setPage] = useState(1);
  const pageSize = 20;

  const { data, isLoading } = useListAuditLogs(
    {
      startDate: null,
      endDate: null,
      userId: null,
      action: null,
      resourceType: null,
      page,
      pageSize,
    },
    [],
    {}
  );

  const exportMutation = useExportAuditLogs();

  const handleExport = async () => {
    try {
      const response = await exportMutation.refetch();
      if (response.data) {
        // Blob handling for CSV export
        const blob = new Blob([response.data as string], { type: 'text/csv' });
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `audit-logs-${format(new Date(), 'yyyy-MM-dd')}.csv`;
        document.body.appendChild(a);
        a.click();
        window.URL.revokeObjectURL(url);
      }
    } catch (error) {
      console.error('Export failed', error);
    }
  };

  const auditLogs = useMemo(() => data?.items ?? [], [data]);

  const getActionColor = (action: string): 'valid' | 'critical' | 'neutral' => {
    if (action.includes('Approved') || action.includes('Login')) return 'valid';
    if (action.includes('Rejected') || action.includes('Delete')) return 'critical';
    return 'neutral';
  };

  const getActionIcon = (action: string) => {
    if (action.includes('Login') || action.includes('Logout')) return User;
    if (action.includes('Approved') || action.includes('Rejected')) return Shield;
    return Activity;
  };

  return (
    <BackOfficeLayout
      title={t('backOfficeAudit.title')}
      subtitle={t('backOfficeAudit.subtitle')}
      actions={
        <Button
          variant="outline"
          size="sm"
          className="rounded-xl border-primary/20 bg-primary/5 text-primary hover:bg-primary/10"
          onClick={handleExport}
          disabled={exportMutation.isFetching}
        >
          <Download className="mr-2 h-4 w-4" />
          {t('backOfficeAudit.export')}
        </Button>
      }
    >
      <div className="space-y-6">
        {/* Filters */}
        <div className="flex flex-col gap-4 sm:flex-row sm:items-center">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder={t('backOfficeAudit.searchPlaceholder')}
              className="pl-10 rounded-xl"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
            />
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" className="rounded-xl h-10 px-4">
              <ArrowUpDown className="mr-2 h-4 w-4" />
              {t('backOfficeAudit.allActions')}
            </Button>
          </div>
        </div>

        {/* Table/List */}
        <Card className="rounded-2xl border-none shadow-premium overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-muted/50 border-b border-border/50">
                <tr>
                  <th className="px-6 py-4 text-left font-bold text-muted-foreground uppercase tracking-wider">
                    {t('backOfficeAudit.timestamp')}
                  </th>
                  <th className="px-6 py-4 text-left font-bold text-muted-foreground uppercase tracking-wider">
                    {t('backOfficeAudit.user')}
                  </th>
                  <th className="px-6 py-4 text-left font-bold text-muted-foreground uppercase tracking-wider">
                    {t('backOfficeAudit.action')}
                  </th>
                  <th className="px-6 py-4 text-left font-bold text-muted-foreground uppercase tracking-wider">
                    {t('backOfficeAudit.details')}
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border/50">
                {isLoading ? (
                  Array.from({ length: 5 }).map((_, i) => (
                    <tr key={i} className="animate-pulse">
                      <td colSpan={4} className="px-6 py-4">
                        <div className="h-4 bg-muted rounded w-full"></div>
                      </td>
                    </tr>
                  ))
                ) : auditLogs.length === 0 ? (
                  <tr>
                    <td colSpan={4} className="px-6 py-12 text-center text-muted-foreground">
                      <div className="flex flex-col items-center gap-2">
                        <Shield className="h-8 w-8 opacity-20" />
                        <p>{t('backOfficeAudit.noLogsFound')}</p>
                      </div>
                    </td>
                  </tr>
                ) : (
                  auditLogs.map((log) => {
                    const ActionIcon = getActionIcon(log.action);
                    return (
                      <tr key={log.id} className="hover:bg-muted/30 transition-colors group">
                        <td className="px-6 py-4 whitespace-nowrap text-muted-foreground flex items-center gap-2">
                          <Clock className="h-3.5 w-3.5" />
                          {format(new Date(log.createdAt), 'MMM d, HH:mm:ss')}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <div className="flex items-center gap-2">
                            <div className="h-7 w-7 rounded-full bg-primary/10 flex items-center justify-center text-primary text-[10px] font-bold">
                              {log.userName?.charAt(0).toUpperCase() ?? '?'}
                            </div>
                            <span className="font-semibold">{log.userName ?? 'System'}</span>
                          </div>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <StatusBadge variant={getActionColor(log.action)} className="capitalize">
                            <ActionIcon className="mr-1.5 h-3 w-3" />
                            {t(`backOfficeAudit.${log.action.toLowerCase()}`, {
                              defaultValue: log.action,
                            })}
                          </StatusBadge>
                        </td>
                        <td className="px-6 py-4 text-muted-foreground">
                          <p className="max-w-xs truncate group-hover:block" title={log.details}>
                            {log.details}
                          </p>
                        </td>
                      </tr>
                    );
                  })
                )}
              </tbody>
            </table>
          </div>
        </Card>

        {/* Pagination placeholder */}
        <div className="flex items-center justify-between px-2">
          <p className="text-xs text-muted-foreground">
            {t('backOfficeControlHistory.pageOf', {
              currentPage: page,
              totalPages: data?.totalPages ?? 1,
            })}
          </p>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              className="rounded-xl"
              onClick={() => setPage((p) => Math.max(1, p - 1))}
              disabled={page === 1}
            >
              {t('backOfficeControlHistory.previous')}
            </Button>
            <Button
              variant="outline"
              size="sm"
              className="rounded-xl"
              onClick={() => setPage((p) => p + 1)}
              disabled={page >= (data?.totalPages ?? 1)}
            >
              {t('backOfficeControlHistory.next')}
            </Button>
          </div>
        </div>
      </div>
    </BackOfficeLayout>
  );
}
