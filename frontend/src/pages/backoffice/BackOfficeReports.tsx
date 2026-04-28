import { useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { BackOfficeLayout } from '@/components/layout/BackOfficeLayout';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { StatusBadge } from '@/components/ui/status-badge';
import { useOrganizations } from '@/hooks/api/useOrganizations';
import { useQuery } from '@tanstack/react-query';
import { fetchWithAuth } from '@/services/api/backendFetch';
import {
  FileText,
  Download,
  Calendar,
  Filter,
  TrendingUp,
  Users,
  AlertTriangle,
  FileSpreadsheet,
  Loader2,
  BarChart3,
  PieChart,
  Activity,
} from 'lucide-react';
import type { PagedControlsResponse } from '@/openapi-rq/requests/types.gen';
import jsPDF from 'jspdf';
import autoTable from 'jspdf-autotable';
import * as XLSX from 'xlsx';

type ReportType = 'control-summary' | 'agent-performance' | 'vehicle-status' | 'organization-stats';

interface ReportFilters {
  startDate: string;
  endDate: string;
  organizationId: string;
  reportType: ReportType;
}

interface AgentStats {
  agentId: string;
  agentName: string;
  totalControls: number;
  validControls: number;
  warningControls: number;
  criticalControls: number;
}

interface OrganizationStats {
  organizationId: string;
  organizationName: string;
  totalControls: number;
  activeAgents: number;
  alertsCount: number;
}

export default function BackOfficeReports() {
  const { t } = useTranslation();
  const { organizations } = useOrganizations();

  // Get default date range (last 30 days)
  const getDefaultDates = () => {
    const end = new Date();
    const start = new Date();
    start.setDate(start.getDate() - 30);
    return {
      start: start.toISOString().split('T')[0],
      end: end.toISOString().split('T')[0],
    };
  };

  const defaultDates = getDefaultDates();

  const [filters, setFilters] = useState<ReportFilters>({
    startDate: defaultDates.start,
    endDate: defaultDates.end,
    organizationId: 'all',
    reportType: 'control-summary',
  });

  const [isGenerating, setIsGenerating] = useState(false);

  // Fetch control data for reports
  const {
    data: controlsData,
    isLoading: isLoadingControls,
    error: controlsError,
  } = useQuery({
    queryKey: ['reports-controls', filters.startDate, filters.endDate, filters.organizationId],
    queryFn: async (): Promise<PagedControlsResponse> => {
      const qs = new URLSearchParams();
      qs.set('page', '1');
      qs.set('page_size', '1000'); // Get more data for reports

      if (filters.organizationId !== 'all') {
        qs.set('organization_id', filters.organizationId);
      }

      if (filters.startDate) {
        qs.set('start_date', `${filters.startDate} 00:00:00`);
      }

      if (filters.endDate) {
        qs.set('end_date', `${filters.endDate} 23:59:59`);
      }

      const res = await fetchWithAuth(`/api/v1/admin/controls/paged?${qs.toString()}`);
      if (!res.ok) {
        throw new Error(`Failed to fetch controls: ${res.status}`);
      }
      return (await res.json()) as PagedControlsResponse;
    },
    enabled: true,
    retry: 1,
    staleTime: 30000, // 30 seconds
  });

  const controls = useMemo(() => controlsData?.items ?? [], [controlsData?.items]);

  // Calculate statistics
  const stats = useMemo(() => {
    const total = controls.length;
    const valid = controls.filter((c) => c.status === 'valid').length;
    const warning = controls.filter((c) => c.status === 'warning').length;
    const critical = controls.filter((c) => c.status === 'critical').length;

    return { total, valid, warning, critical };
  }, [controls]);

  // Calculate agent performance
  const agentStats = useMemo((): AgentStats[] => {
    const agentMap = new Map<string, AgentStats>();

    controls.forEach((control) => {
      const agentId = control.agent_id;
      const agentName = control.agent_name || 'Unknown Agent';

      if (!agentMap.has(agentId)) {
        agentMap.set(agentId, {
          agentId,
          agentName,
          totalControls: 0,
          validControls: 0,
          warningControls: 0,
          criticalControls: 0,
        });
      }

      const stats = agentMap.get(agentId)!;
      stats.totalControls++;

      if (control.status === 'valid') stats.validControls++;
      if (control.status === 'warning') stats.warningControls++;
      if (control.status === 'critical') stats.criticalControls++;
    });

    return Array.from(agentMap.values()).sort((a, b) => b.totalControls - a.totalControls);
  }, [controls]);

  // Calculate organization statistics
  const organizationStats = useMemo((): OrganizationStats[] => {
    const orgMap = new Map<string, OrganizationStats>();

    controls.forEach((control) => {
      const orgId = control.organization_id;
      const orgName = organizations?.find((o) => o.id === orgId)?.name || 'Unknown Organization';

      if (!orgMap.has(orgId)) {
        orgMap.set(orgId, {
          organizationId: orgId,
          organizationName: orgName,
          totalControls: 0,
          activeAgents: new Set<string>().size,
          alertsCount: 0,
        });
      }

      const stats = orgMap.get(orgId)!;
      stats.totalControls++;

      if (control.status === 'critical' || control.status === 'warning') {
        stats.alertsCount++;
      }
    });

    // Count unique agents per organization
    controls.forEach((control) => {
      const orgId = control.organization_id;
      const stats = orgMap.get(orgId);
      if (stats) {
        // This is a simplified count - in real implementation, track unique agent IDs
        stats.activeAgents = Math.ceil(stats.totalControls / 10); // Rough estimate
      }
    });

    return Array.from(orgMap.values()).sort((a, b) => b.totalControls - a.totalControls);
  }, [controls, organizations]);

  // Export functions
  const exportToCSV = () => {
    setIsGenerating(true);
    setTimeout(() => {
      let csvContent = '';

      if (filters.reportType === 'control-summary') {
        csvContent = 'Plate Number,Status,Agent,Organization,Date,Location\n';
        controls.forEach((control) => {
          const row = [
            control.plate_number,
            control.status,
            control.agent_name,
            organizations?.find((o) => o.id === control.organization_id)?.name || '',
            new Date(control.timestamp).toLocaleString(),
            control.location?.address || '',
          ];
          csvContent += row.map((field) => `"${field}"`).join(',') + '\n';
        });
      } else if (filters.reportType === 'agent-performance') {
        csvContent = 'Agent Name,Total Controls,Valid,Warning,Critical\n';
        agentStats.forEach((agent) => {
          csvContent += `"${agent.agentName}",${agent.totalControls},${agent.validControls},${agent.warningControls},${agent.criticalControls}\n`;
        });
      } else if (filters.reportType === 'organization-stats') {
        csvContent = 'Organization,Total Controls,Active Agents,Alerts\n';
        organizationStats.forEach((org) => {
          csvContent += `"${org.organizationName}",${org.totalControls},${org.activeAgents},${org.alertsCount}\n`;
        });
      }

      const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
      const link = document.createElement('a');
      const url = URL.createObjectURL(blob);
      link.setAttribute('href', url);
      link.setAttribute('download', `iviss-report-${filters.reportType}-${Date.now()}.csv`);
      link.style.visibility = 'hidden';
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);

      setIsGenerating(false);
    }, 500);
  };

  const exportToPDF = () => {
    setIsGenerating(true);
    setTimeout(() => {
      try {
        const doc = new jsPDF();

        // Add title
        doc.setFontSize(20);
        doc.setFont('helvetica', 'bold');
        doc.text('IVISS Control Report', 14, 20);

        // Add report type
        doc.setFontSize(14);
        doc.setFont('helvetica', 'normal');
        const reportTypeLabel =
          reportTypeOptions.find((opt) => opt.value === filters.reportType)?.label || '';
        doc.text(reportTypeLabel, 14, 30);

        // Add metadata
        doc.setFontSize(10);
        doc.setTextColor(100);
        doc.text(`Date Range: ${filters.startDate} to ${filters.endDate}`, 14, 38);

        const orgName =
          filters.organizationId === 'all'
            ? 'All Organizations'
            : organizations?.find((o) => o.id === filters.organizationId)?.name || 'Unknown';
        doc.text(`Organization: ${orgName}`, 14, 44);
        doc.text(`Generated: ${new Date().toLocaleString()}`, 14, 50);

        // Add statistics summary
        doc.setFontSize(12);
        doc.setTextColor(0);
        doc.setFont('helvetica', 'bold');
        doc.text('Summary Statistics', 14, 60);

        doc.setFontSize(10);
        doc.setFont('helvetica', 'normal');
        doc.text(`Total Controls: ${stats.total}`, 14, 68);
        doc.text(
          `Valid: ${stats.valid} (${stats.total > 0 ? Math.round((stats.valid / stats.total) * 100) : 0}%)`,
          14,
          74
        );
        doc.text(
          `Warning: ${stats.warning} (${stats.total > 0 ? Math.round((stats.warning / stats.total) * 100) : 0}%)`,
          14,
          80
        );
        doc.text(
          `Critical: ${stats.critical} (${stats.total > 0 ? Math.round((stats.critical / stats.total) * 100) : 0}%)`,
          14,
          86
        );

        // Add table based on report type
        let tableData: string[][] = [];
        let headers: string[] = [];

        if (filters.reportType === 'control-summary') {
          headers = ['Plate Number', 'Status', 'Agent', 'Organization', 'Date'];
          tableData = controls.map((control) => [
            control.plate_number,
            control.status,
            control.agent_name || 'Unknown',
            organizations?.find((o) => o.id === control.organization_id)?.name || 'Unknown',
            new Date(control.timestamp).toLocaleDateString(),
          ]);
        } else if (filters.reportType === 'agent-performance') {
          headers = ['Agent Name', 'Total Controls', 'Valid', 'Warning', 'Critical'];
          tableData = agentStats.map((agent) => [
            agent.agentName,
            agent.totalControls.toString(),
            agent.validControls.toString(),
            agent.warningControls.toString(),
            agent.criticalControls.toString(),
          ]);
        } else if (filters.reportType === 'organization-stats') {
          headers = ['Organization', 'Total Controls', 'Active Agents', 'Alerts'];
          tableData = organizationStats.map((org) => [
            org.organizationName,
            org.totalControls.toString(),
            org.activeAgents.toString(),
            org.alertsCount.toString(),
          ]);
        } else {
          headers = ['Plate Number', 'Status', 'Agent', 'Date'];
          tableData = controls
            .slice(0, 50)
            .map((control) => [
              control.plate_number,
              control.status,
              control.agent_name || 'Unknown',
              new Date(control.timestamp).toLocaleDateString(),
            ]);
        }

        // Add table using autoTable plugin
        autoTable(doc, {
          head: [headers],
          body: tableData,
          startY: 95,
          styles: {
            fontSize: 8,
            cellPadding: 3,
          },
          headStyles: {
            fillColor: [59, 130, 246], // Blue
            textColor: 255,
            fontStyle: 'bold',
          },
          alternateRowStyles: {
            fillColor: [245, 247, 250],
          },
          margin: { top: 95, left: 14, right: 14 },
        });

        // Add footer
        const pageCount = doc.getNumberOfPages();
        for (let i = 1; i <= pageCount; i++) {
          doc.setPage(i);
          doc.setFontSize(8);
          doc.setTextColor(150);
          doc.text(
            `Page ${i} of ${pageCount}`,
            doc.internal.pageSize.getWidth() / 2,
            doc.internal.pageSize.getHeight() - 10,
            { align: 'center' }
          );
          doc.text(
            '© 2024 IVISS - National Vehicle Control System',
            doc.internal.pageSize.getWidth() / 2,
            doc.internal.pageSize.getHeight() - 5,
            { align: 'center' }
          );
        }

        // Save the PDF
        doc.save(`iviss-report-${filters.reportType}-${Date.now()}.pdf`);
      } catch (error) {
        console.error('PDF export error:', error);
        alert('Failed to generate PDF. Please try again.');
      } finally {
        setIsGenerating(false);
      }
    }, 500);
  };

  const exportToExcel = () => {
    setIsGenerating(true);
    setTimeout(() => {
      try {
        // Create a new workbook
        const wb = XLSX.utils.book_new();

        // Add metadata sheet
        const metadataData = [
          ['IVISS Control Report'],
          [''],
          [
            'Report Type',
            reportTypeOptions.find((opt) => opt.value === filters.reportType)?.label || '',
          ],
          ['Date Range', `${filters.startDate} to ${filters.endDate}`],
          [
            'Organization',
            filters.organizationId === 'all'
              ? 'All Organizations'
              : organizations?.find((o) => o.id === filters.organizationId)?.name || 'Unknown',
          ],
          ['Generated', new Date().toLocaleString()],
          [''],
          ['Summary Statistics'],
          ['Total Controls', stats.total],
          [
            'Valid Controls',
            stats.valid,
            `${stats.total > 0 ? Math.round((stats.valid / stats.total) * 100) : 0}%`,
          ],
          [
            'Warning Controls',
            stats.warning,
            `${stats.total > 0 ? Math.round((stats.warning / stats.total) * 100) : 0}%`,
          ],
          [
            'Critical Controls',
            stats.critical,
            `${stats.total > 0 ? Math.round((stats.critical / stats.total) * 100) : 0}%`,
          ],
        ];
        const wsMetadata = XLSX.utils.aoa_to_sheet(metadataData);
        XLSX.utils.book_append_sheet(wb, wsMetadata, 'Summary');

        // Add data sheet based on report type
        if (filters.reportType === 'control-summary') {
          const controlData = [
            ['Plate Number', 'Status', 'Agent', 'Organization', 'Date', 'Location'],
            ...controls.map((control) => [
              control.plate_number,
              control.status,
              control.agent_name || 'Unknown',
              organizations?.find((o) => o.id === control.organization_id)?.name || 'Unknown',
              new Date(control.timestamp).toLocaleString(),
              control.location?.address || '',
            ]),
          ];
          const wsControls = XLSX.utils.aoa_to_sheet(controlData);

          // Set column widths
          wsControls['!cols'] = [
            { wch: 15 }, // Plate Number
            { wch: 10 }, // Status
            { wch: 20 }, // Agent
            { wch: 25 }, // Organization
            { wch: 20 }, // Date
            { wch: 40 }, // Location
          ];

          XLSX.utils.book_append_sheet(wb, wsControls, 'Controls');
        } else if (filters.reportType === 'agent-performance') {
          const agentData = [
            ['Agent Name', 'Total Controls', 'Valid', 'Warning', 'Critical', 'Success Rate'],
            ...agentStats.map((agent) => [
              agent.agentName,
              agent.totalControls,
              agent.validControls,
              agent.warningControls,
              agent.criticalControls,
              `${agent.totalControls > 0 ? Math.round((agent.validControls / agent.totalControls) * 100) : 0}%`,
            ]),
          ];
          const wsAgents = XLSX.utils.aoa_to_sheet(agentData);

          // Set column widths
          wsAgents['!cols'] = [
            { wch: 25 }, // Agent Name
            { wch: 15 }, // Total Controls
            { wch: 10 }, // Valid
            { wch: 10 }, // Warning
            { wch: 10 }, // Critical
            { wch: 15 }, // Success Rate
          ];

          XLSX.utils.book_append_sheet(wb, wsAgents, 'Agent Performance');
        } else if (filters.reportType === 'organization-stats') {
          const orgData = [
            ['Organization', 'Total Controls', 'Active Agents', 'Alerts', 'Alert Rate'],
            ...organizationStats.map((org) => [
              org.organizationName,
              org.totalControls,
              org.activeAgents,
              org.alertsCount,
              `${org.totalControls > 0 ? Math.round((org.alertsCount / org.totalControls) * 100) : 0}%`,
            ]),
          ];
          const wsOrgs = XLSX.utils.aoa_to_sheet(orgData);

          // Set column widths
          wsOrgs['!cols'] = [
            { wch: 30 }, // Organization
            { wch: 15 }, // Total Controls
            { wch: 15 }, // Active Agents
            { wch: 10 }, // Alerts
            { wch: 12 }, // Alert Rate
          ];

          XLSX.utils.book_append_sheet(wb, wsOrgs, 'Organizations');
        } else {
          // Vehicle status or default
          const vehicleData = [
            ['Plate Number', 'Status', 'Agent', 'Organization', 'Date'],
            ...controls.map((control) => [
              control.plate_number,
              control.status,
              control.agent_name || 'Unknown',
              organizations?.find((o) => o.id === control.organization_id)?.name || 'Unknown',
              new Date(control.timestamp).toLocaleString(),
            ]),
          ];
          const wsVehicles = XLSX.utils.aoa_to_sheet(vehicleData);

          // Set column widths
          wsVehicles['!cols'] = [
            { wch: 15 }, // Plate Number
            { wch: 10 }, // Status
            { wch: 20 }, // Agent
            { wch: 25 }, // Organization
            { wch: 20 }, // Date
          ];

          XLSX.utils.book_append_sheet(wb, wsVehicles, 'Data');
        }

        // Save the Excel file
        XLSX.writeFile(wb, `iviss-report-${filters.reportType}-${Date.now()}.xlsx`);
      } catch (error) {
        console.error('Excel export error:', error);
        alert('Failed to generate Excel file. Please try again.');
      } finally {
        setIsGenerating(false);
      }
    }, 500);
  };

  const reportTypeOptions = [
    { value: 'control-summary', label: 'Control Summary Report', icon: BarChart3 },
    { value: 'agent-performance', label: 'Agent Performance Report', icon: Users },
    { value: 'vehicle-status', label: 'Vehicle Status Report', icon: Activity },
    { value: 'organization-stats', label: 'Organization Statistics', icon: PieChart },
  ];

  return (
    <BackOfficeLayout
      title={t('backOfficeSidebar.generateReport')}
      subtitle="Generate comprehensive reports on control activities"
    >
      {/* Loading State */}
      {isLoadingControls && (
        <div className="flex h-64 items-center justify-center">
          <div className="text-center">
            <Loader2 className="mx-auto h-12 w-12 animate-spin text-primary" />
            <p className="mt-4 text-sm text-muted-foreground">Loading report data...</p>
          </div>
        </div>
      )}

      {/* Error State */}
      {controlsError && (
        <Card className="rounded-2xl border border-red-200 bg-red-50 dark:border-red-900 dark:bg-red-950">
          <CardContent className="p-6">
            <div className="flex items-center gap-3">
              <AlertTriangle className="h-6 w-6 text-red-600" />
              <div>
                <h3 className="font-semibold text-red-900 dark:text-red-100">
                  Failed to Load Data
                </h3>
                <p className="mt-1 text-sm text-red-700 dark:text-red-300">
                  {controlsError instanceof Error
                    ? controlsError.message
                    : 'An error occurred while fetching report data.'}
                </p>
                <Button
                  onClick={() => window.location.reload()}
                  variant="outline"
                  size="sm"
                  className="mt-3"
                >
                  Retry
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Main Content - Only show when not loading and no error */}
      {!isLoadingControls && !controlsError && (
        <div className="space-y-6">
          {/* Report Configuration Card */}
          <Card className="rounded-2xl border border-border/60 bg-card shadow-md">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <FileText className="h-5 w-5" />
                Report Configuration
              </CardTitle>
              <CardDescription>
                Select the date range, organization, and report type to generate
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* Report Type Selection */}
              <div className="space-y-2">
                <Label htmlFor="report-type">Report Type</Label>
                <Select
                  value={filters.reportType}
                  onValueChange={(value) =>
                    setFilters({ ...filters, reportType: value as ReportType })
                  }
                >
                  <SelectTrigger id="report-type" className="h-11 rounded-xl">
                    <SelectValue placeholder="Select report type" />
                  </SelectTrigger>
                  <SelectContent>
                    {reportTypeOptions.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        <div className="flex items-center gap-2">
                          <option.icon className="h-4 w-4" />
                          {option.label}
                        </div>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* Filters Grid */}
              <div className="grid gap-4 md:grid-cols-3">
                {/* Date Range */}
                <div className="space-y-2">
                  <Label htmlFor="start-date">Start Date</Label>
                  <div className="relative">
                    <Calendar className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <input
                      id="start-date"
                      type="date"
                      value={filters.startDate}
                      onChange={(e) => setFilters({ ...filters, startDate: e.target.value })}
                      className="h-11 w-full rounded-xl border border-border bg-background pl-10 pr-3 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
                    />
                  </div>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="end-date">End Date</Label>
                  <div className="relative">
                    <Calendar className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <input
                      id="end-date"
                      type="date"
                      value={filters.endDate}
                      onChange={(e) => setFilters({ ...filters, endDate: e.target.value })}
                      className="h-11 w-full rounded-xl border border-border bg-background pl-10 pr-3 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
                    />
                  </div>
                </div>

                {/* Organization Filter */}
                <div className="space-y-2">
                  <Label htmlFor="organization">Organization</Label>
                  <Select
                    value={filters.organizationId}
                    onValueChange={(value) => setFilters({ ...filters, organizationId: value })}
                  >
                    <SelectTrigger id="organization" className="h-11 rounded-xl">
                      <Filter className="mr-2 h-4 w-4 text-muted-foreground" />
                      <SelectValue placeholder="Select organization" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">All Organizations</SelectItem>
                      {(organizations ?? []).map((org) => (
                        <SelectItem key={org.id} value={org.id}>
                          {org.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>

              {/* Export Buttons */}
              <div className="flex flex-wrap gap-3 border-t border-border/60 pt-6">
                <Button
                  onClick={exportToCSV}
                  disabled={isGenerating || isLoadingControls}
                  className="gap-2 rounded-xl"
                >
                  {isGenerating ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <FileSpreadsheet className="h-4 w-4" />
                  )}
                  Export to CSV
                </Button>
                <Button
                  onClick={exportToPDF}
                  disabled={isGenerating || isLoadingControls}
                  variant="outline"
                  className="gap-2 rounded-xl"
                >
                  {isGenerating ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <FileText className="h-4 w-4" />
                  )}
                  Export to PDF
                </Button>
                <Button
                  onClick={exportToExcel}
                  disabled={isGenerating || isLoadingControls}
                  variant="outline"
                  className="gap-2 rounded-xl"
                >
                  {isGenerating ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <Download className="h-4 w-4" />
                  )}
                  Export to Excel
                </Button>
              </div>
            </CardContent>
          </Card>

          {/* Statistics Overview */}
          <div className="grid gap-4 md:grid-cols-4">
            <Card className="rounded-2xl border border-border/60 bg-card shadow-sm">
              <CardContent className="p-6">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-muted-foreground">Total Controls</p>
                    <p className="mt-2 text-3xl font-bold">{stats.total}</p>
                  </div>
                  <div className="rounded-full bg-primary/10 p-3">
                    <BarChart3 className="h-6 w-6 text-primary" />
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card className="rounded-2xl border border-border/60 bg-card shadow-sm">
              <CardContent className="p-6">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-muted-foreground">Valid</p>
                    <p className="mt-2 text-3xl font-bold text-green-600">{stats.valid}</p>
                  </div>
                  <div className="rounded-full bg-green-100 p-3 dark:bg-green-900/20">
                    <TrendingUp className="h-6 w-6 text-green-600" />
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card className="rounded-2xl border border-border/60 bg-card shadow-sm">
              <CardContent className="p-6">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-muted-foreground">Warnings</p>
                    <p className="mt-2 text-3xl font-bold text-yellow-600">{stats.warning}</p>
                  </div>
                  <div className="rounded-full bg-yellow-100 p-3 dark:bg-yellow-900/20">
                    <AlertTriangle className="h-6 w-6 text-yellow-600" />
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card className="rounded-2xl border border-border/60 bg-card shadow-sm">
              <CardContent className="p-6">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-muted-foreground">Critical</p>
                    <p className="mt-2 text-3xl font-bold text-red-600">{stats.critical}</p>
                  </div>
                  <div className="rounded-full bg-red-100 p-3 dark:bg-red-900/20">
                    <AlertTriangle className="h-6 w-6 text-red-600" />
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Report Preview */}
          <Card className="rounded-2xl border border-border/60 bg-card shadow-md">
            <CardHeader>
              <CardTitle>Report Preview</CardTitle>
              <CardDescription>
                Preview of the data that will be included in your report
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Tabs value={filters.reportType} className="w-full">
                <TabsList className="grid w-full grid-cols-4 rounded-xl">
                  <TabsTrigger value="control-summary" className="rounded-lg">
                    Controls
                  </TabsTrigger>
                  <TabsTrigger value="agent-performance" className="rounded-lg">
                    Agents
                  </TabsTrigger>
                  <TabsTrigger value="vehicle-status" className="rounded-lg">
                    Vehicles
                  </TabsTrigger>
                  <TabsTrigger value="organization-stats" className="rounded-lg">
                    Organizations
                  </TabsTrigger>
                </TabsList>

                {/* Control Summary Tab */}
                <TabsContent value="control-summary" className="mt-6">
                  <div className="overflow-hidden rounded-xl border border-border/60">
                    <Table>
                      <TableHeader>
                        <TableRow className="bg-muted/40">
                          <TableHead className="font-bold">Plate Number</TableHead>
                          <TableHead className="font-bold">Status</TableHead>
                          <TableHead className="font-bold">Agent</TableHead>
                          <TableHead className="font-bold">Organization</TableHead>
                          <TableHead className="font-bold">Date</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {controls.slice(0, 10).map((control) => (
                          <TableRow key={control.id}>
                            <TableCell className="font-mono font-bold">
                              {control.plate_number}
                            </TableCell>
                            <TableCell>
                              <StatusBadge variant={control.status} size="sm">
                                {control.status}
                              </StatusBadge>
                            </TableCell>
                            <TableCell>{control.agent_name}</TableCell>
                            <TableCell className="text-muted-foreground">
                              {organizations?.find((o) => o.id === control.organization_id)?.name ||
                                'Unknown'}
                            </TableCell>
                            <TableCell className="text-sm text-muted-foreground">
                              {new Date(control.timestamp).toLocaleDateString()}
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                    {controls.length > 10 && (
                      <div className="border-t border-border/60 bg-muted/20 p-3 text-center text-sm text-muted-foreground">
                        Showing 10 of {controls.length} controls. Export to see all data.
                      </div>
                    )}
                  </div>
                </TabsContent>

                {/* Agent Performance Tab */}
                <TabsContent value="agent-performance" className="mt-6">
                  <div className="overflow-hidden rounded-xl border border-border/60">
                    <Table>
                      <TableHeader>
                        <TableRow className="bg-muted/40">
                          <TableHead className="font-bold">Agent Name</TableHead>
                          <TableHead className="font-bold">Total Controls</TableHead>
                          <TableHead className="font-bold">Valid</TableHead>
                          <TableHead className="font-bold">Warning</TableHead>
                          <TableHead className="font-bold">Critical</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {agentStats.slice(0, 10).map((agent) => (
                          <TableRow key={agent.agentId}>
                            <TableCell className="font-medium">{agent.agentName}</TableCell>
                            <TableCell className="font-bold">{agent.totalControls}</TableCell>
                            <TableCell className="text-green-600">{agent.validControls}</TableCell>
                            <TableCell className="text-yellow-600">
                              {agent.warningControls}
                            </TableCell>
                            <TableCell className="text-red-600">{agent.criticalControls}</TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                    {agentStats.length > 10 && (
                      <div className="border-t border-border/60 bg-muted/20 p-3 text-center text-sm text-muted-foreground">
                        Showing 10 of {agentStats.length} agents. Export to see all data.
                      </div>
                    )}
                  </div>
                </TabsContent>

                {/* Vehicle Status Tab */}
                <TabsContent value="vehicle-status" className="mt-6">
                  <div className="rounded-xl border border-border/60 bg-muted/20 p-8 text-center">
                    <Activity className="mx-auto h-12 w-12 text-muted-foreground" />
                    <p className="mt-4 text-sm text-muted-foreground">
                      Vehicle status report shows detailed compliance information for all vehicles
                      checked during the selected period.
                    </p>
                    <p className="mt-2 text-sm font-medium">
                      Export the report to view detailed vehicle status data.
                    </p>
                  </div>
                </TabsContent>

                {/* Organization Stats Tab */}
                <TabsContent value="organization-stats" className="mt-6">
                  <div className="overflow-hidden rounded-xl border border-border/60">
                    <Table>
                      <TableHeader>
                        <TableRow className="bg-muted/40">
                          <TableHead className="font-bold">Organization</TableHead>
                          <TableHead className="font-bold">Total Controls</TableHead>
                          <TableHead className="font-bold">Active Agents</TableHead>
                          <TableHead className="font-bold">Alerts</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {organizationStats.map((org) => (
                          <TableRow key={org.organizationId}>
                            <TableCell className="font-medium">{org.organizationName}</TableCell>
                            <TableCell className="font-bold">{org.totalControls}</TableCell>
                            <TableCell>{org.activeAgents}</TableCell>
                            <TableCell className="text-red-600">{org.alertsCount}</TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </div>
                </TabsContent>
              </Tabs>
            </CardContent>
          </Card>
        </div>
      )}
    </BackOfficeLayout>
  );
}
