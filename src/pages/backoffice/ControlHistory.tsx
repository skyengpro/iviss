import { useState } from "react";
import { BackOfficeLayout } from "@/components/layout/BackOfficeLayout";
import { StatusBadge } from "@/components/ui/status-badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Search,
  Filter,
  Download,
  Eye,
  Calendar,
  RefreshCw,
  ChevronLeft,
  ChevronRight,
} from "lucide-react";

// Mock control data
const mockControls = [
  {
    id: "CTR-001",
    plateNumber: "AB-123-CD",
    vehicleBrand: "Renault",
    vehicleModel: "Clio",
    status: "valid" as const,
    agentName: "Agent Dupont",
    organization: "Brigade Alpha",
    location: "Highway A1, KM 42",
    timestamp: "2024-01-15 10:30:00",
    gpsCoords: "48.8566, 2.3522",
  },
  {
    id: "CTR-002",
    plateNumber: "XY-789-ZW",
    vehicleBrand: "Peugeot",
    vehicleModel: "308",
    status: "warning" as const,
    agentName: "Agent Martin",
    organization: "Brigade Beta",
    location: "Rue de Paris, Checkpoint 3",
    timestamp: "2024-01-15 09:45:00",
    gpsCoords: "48.8744, 2.3526",
  },
  {
    id: "CTR-003",
    plateNumber: "EF-456-GH",
    vehicleBrand: "BMW",
    vehicleModel: "X3",
    status: "critical" as const,
    agentName: "Agent Bernard",
    organization: "Brigade Alpha",
    location: "Border Checkpoint Alpha",
    timestamp: "2024-01-15 08:15:00",
    gpsCoords: "48.8534, 2.3488",
  },
  {
    id: "CTR-004",
    plateNumber: "JK-321-LM",
    vehicleBrand: "Volkswagen",
    vehicleModel: "Golf",
    status: "valid" as const,
    agentName: "Agent Leroy",
    organization: "Brigade Gamma",
    location: "Highway A6, KM 15",
    timestamp: "2024-01-14 16:20:00",
    gpsCoords: "48.8456, 2.3789",
  },
  {
    id: "CTR-005",
    plateNumber: "NO-654-PQ",
    vehicleBrand: "Toyota",
    vehicleModel: "Yaris",
    status: "valid" as const,
    agentName: "Agent Dupont",
    organization: "Brigade Alpha",
    location: "Highway A1, KM 42",
    timestamp: "2024-01-14 14:10:00",
    gpsCoords: "48.8566, 2.3522",
  },
];

export default function ControlHistory() {
  const [searchQuery, setSearchQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState("all");
  const [organizationFilter, setOrganizationFilter] = useState("all");

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

              <Select
                value={organizationFilter}
                onValueChange={setOrganizationFilter}
              >
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
              Showing <span className="font-semibold text-foreground">5</span> of{" "}
              <span className="font-semibold text-foreground">1,284</span> controls
            </p>
            <div className="flex gap-2">
              <StatusBadge variant="valid" size="sm">Valid: 982</StatusBadge>
              <StatusBadge variant="warning" size="sm">Warning: 256</StatusBadge>
              <StatusBadge variant="critical" size="sm">Critical: 46</StatusBadge>
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
                {mockControls.map((control) => (
                  <TableRow key={control.id} className="group">
                    <TableCell className="font-mono text-sm">
                      {control.id}
                    </TableCell>
                    <TableCell>
                      <span className="font-mono font-semibold tracking-wider">
                        {control.plateNumber}
                      </span>
                    </TableCell>
                    <TableCell>
                      {control.vehicleBrand} {control.vehicleModel}
                    </TableCell>
                    <TableCell>
                      <StatusBadge variant={control.status} size="sm">
                        {control.status}
                      </StatusBadge>
                    </TableCell>
                    <TableCell>{control.agentName}</TableCell>
                    <TableCell>{control.organization}</TableCell>
                    <TableCell className="max-w-[200px] truncate">
                      {control.location}
                    </TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {control.timestamp}
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
                ))}
              </TableBody>
            </Table>
          </div>

          {/* Pagination */}
          <div className="mt-4 flex items-center justify-between">
            <p className="text-sm text-muted-foreground">
              Page 1 of 129
            </p>
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
