import { useState } from 'react';
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
      title="Control History"
      subtitle="View and manage all control records"
      actions={
        <div className="flex gap-2">
          <Button variant="outline" className="gap-2">
            <RefreshCw className="h-4 w-4" />
            Refresh
          </Button>
          <Button className="gap-2 bg-accent text-accent-foreground hover:bg-accent/90">
            <Download className="h-4 w-4" />
            Export
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
                placeholder="Search by plate, agent, location..."
                className="pl-9"
              />
            </div>

            {/* Filters */}
            <div className="flex flex-wrap gap-2">
              <Select value={statusFilter} onValueChange={setStatusFilter}>
                <SelectTrigger className="w-[140px]">
                  <SelectValue placeholder="Status" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Status</SelectItem>
                  <SelectItem value="valid">Valid</SelectItem>
                  <SelectItem value="warning">Warning</SelectItem>
                  <SelectItem value="critical">Critical</SelectItem>
                </SelectContent>
              </Select>

              <Select value={organizationFilter} onValueChange={setOrganizationFilter}>
                <SelectTrigger className="w-[160px]">
                  <SelectValue placeholder="Organization" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Organizations</SelectItem>
                  <SelectItem value="alpha">Brigade Alpha</SelectItem>
                  <SelectItem value="beta">Brigade Beta</SelectItem>
                  <SelectItem value="gamma">Brigade Gamma</SelectItem>
                </SelectContent>
              </Select>

              <Button variant="outline" className="gap-2">
                <Calendar className="h-4 w-4" />
                Date Range
              </Button>

              <Button variant="outline" className="gap-2">
                <Filter className="h-4 w-4" />
                More Filters
              </Button>
            </div>
          </div>
        </CardHeader>

        <CardContent>
          {/* Results summary */}
          <div className="mb-4 flex items-center justify-between">
            <p className="text-sm text-muted-foreground">
              Showing{' '}
              <span className="font-semibold text-foreground">{filteredControls.length}</span>{' '}
              controls
            </p>
            <div className="flex gap-2">
              <StatusBadge variant="valid" size="sm">
                Valid: {filteredControls.filter((c) => c.status === 'valid').length}
              </StatusBadge>
              <StatusBadge variant="warning" size="sm">
                Warning: {filteredControls.filter((c) => c.status === 'warning').length}
              </StatusBadge>
              <StatusBadge variant="critical" size="sm">
                Critical: {filteredControls.filter((c) => c.status === 'critical').length}
              </StatusBadge>
            </div>
          </div>

          {/* Table */}
          <div className="rounded-lg border">
            <Table>
              <TableHeader>
                <TableRow className="bg-muted/50">
                  <TableHead className="w-[100px]">ID</TableHead>
                  <TableHead>Plate Number</TableHead>
                  <TableHead>Vehicle</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Agent</TableHead>
                  <TableHead>Organization</TableHead>
                  <TableHead>Location</TableHead>
                  <TableHead>Date/Time</TableHead>
                  <TableHead className="w-[80px]">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {isLoading ? (
                  <TableRow>
                    <TableCell colSpan={9} className="h-24 text-center">
                      <div className="flex items-center justify-center gap-2">
                        <RefreshCw className="h-4 w-4 animate-spin" />
                        Loading controls...
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
                      <TableCell>Vehicle Info</TableCell>
                      <TableCell>
                        <StatusBadge variant={control.status} size="sm">
                          {control.status}
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
                      No controls found matching your filters.
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </div>

          {/* Pagination */}
          <div className="mt-4 flex items-center justify-between">
            <p className="text-sm text-muted-foreground">Page 1 of 129</p>
            <div className="flex gap-2">
              <Button variant="outline" size="sm" disabled>
                <ChevronLeft className="h-4 w-4" />
                Previous
              </Button>
              <Button variant="outline" size="sm">
                Next
                <ChevronRight className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </BackOfficeLayout>
  );
}
