import { StatusBadge } from '@/components/ui/status-badge';
import { cn } from '@/lib/utils';
import { Car, User, Calendar, FileCheck, Shield, AlertTriangle, ChevronRight } from 'lucide-react';

export type VehicleStatus = 'valid' | 'warning' | 'critical' | 'pending';

interface VehicleInfo {
  plateNumber: string;
  chassisNumber: string;
  brand: string;
  model: string;
  year: number;
  enginePower: string;
  owner: {
    name: string;
    address?: string;
  };
  registration: {
    status: VehicleStatus;
    expiryDate: string;
  };
  insurance: {
    status: VehicleStatus;
    provider?: string;
    expiryDate?: string;
  };
  technicalInspection: {
    status: VehicleStatus;
    expiryDate?: string;
  };
  wantedStatus: {
    status: VehicleStatus;
    reason?: string;
  };
  customsStatus: {
    status: VehicleStatus;
    notes?: string;
  };
}

interface VehicleStatusCardProps {
  readonly vehicle: VehicleInfo;
  readonly className?: string;
  readonly onClick?: () => void;
}

const statusLabels: Record<VehicleStatus, string> = {
  valid: 'Valid',
  warning: 'Warning',
  critical: 'Alert',
  pending: 'Pending',
};

export function VehicleStatusCard({ vehicle, className, onClick }: VehicleStatusCardProps) {
  // Determine overall status
  const getOverallStatus = (): VehicleStatus => {
    if (vehicle.wantedStatus.status === 'critical') return 'critical';
    if (vehicle.customsStatus.status === 'critical') return 'critical';
    if (
      vehicle.registration.status === 'warning' ||
      vehicle.insurance.status === 'warning' ||
      vehicle.technicalInspection.status === 'warning'
    ) {
      return 'warning';
    }
    if (
      vehicle.registration.status === 'pending' ||
      vehicle.insurance.status === 'pending' ||
      vehicle.technicalInspection.status === 'pending'
    ) {
      return 'pending';
    }
    return 'valid';
  };

  const overallStatus = getOverallStatus();

  return (
    <div
      className={cn(
        'rounded-xl border bg-card p-4 transition-all duration-200 card-elevated',
        overallStatus === 'critical' && 'border-status-critical/50 ring-2 ring-status-critical/20',
        overallStatus === 'warning' && 'border-status-warning/50',
        onClick && 'cursor-pointer active:scale-[0.98]',
        className
      )}
      onClick={onClick}
    >
      {/* Header with plate number */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div
            className={cn(
              'flex h-12 w-12 items-center justify-center rounded-lg',
              overallStatus === 'critical' && 'bg-status-critical/10 text-status-critical',
              overallStatus === 'warning' && 'bg-status-warning/10 text-status-warning',
              overallStatus === 'valid' && 'bg-status-valid/10 text-status-valid',
              overallStatus === 'pending' && 'bg-muted text-muted-foreground'
            )}
          >
            <Car className="h-6 w-6" />
          </div>
          <div>
            <p className="text-xl font-bold tracking-wider">{vehicle.plateNumber}</p>
            <p className="text-sm text-muted-foreground">
              {vehicle.brand} {vehicle.model} • {vehicle.year}
            </p>
          </div>
        </div>
        <StatusBadge variant={overallStatus} size="lg">
          {statusLabels[overallStatus]}
        </StatusBadge>
      </div>

      {/* Critical alert */}
      {overallStatus === 'critical' && (
        <div className="mt-4 flex items-center gap-2 rounded-lg bg-status-critical/10 p-3 text-status-critical">
          <AlertTriangle className="h-5 w-5 shrink-0" />
          <p className="text-sm font-medium">
            {vehicle.wantedStatus.status === 'critical'
              ? vehicle.wantedStatus.reason || 'Vehicle is flagged - Contact dispatch immediately'
              : vehicle.customsStatus.notes || 'Customs clearance issue detected'}
          </p>
        </div>
      )}

      {/* Owner info */}
      <div className="mt-4 flex items-center gap-2 text-sm">
        <User className="h-4 w-4 text-muted-foreground" />
        <span className="text-muted-foreground">Owner:</span>
        <span className="font-medium">{vehicle.owner.name}</span>
      </div>

      {/* Status grid */}
      <div className="mt-4 grid grid-cols-2 gap-2">
        <StatusItem
          icon={FileCheck}
          label="Registration"
          status={vehicle.registration.status}
          detail={vehicle.registration.expiryDate}
        />
        <StatusItem
          icon={Shield}
          label="Insurance"
          status={vehicle.insurance.status}
          detail={vehicle.insurance.expiryDate}
        />
        <StatusItem
          icon={Calendar}
          label="Tech. Inspection"
          status={vehicle.technicalInspection.status}
          detail={vehicle.technicalInspection.expiryDate}
        />
        <StatusItem
          icon={AlertTriangle}
          label="Wanted Status"
          status={vehicle.wantedStatus.status}
        />
      </div>

      {/* View details */}
      {onClick && (
        <div className="mt-4 flex items-center justify-center gap-1 text-sm text-accent">
          <span>View full details</span>
          <ChevronRight className="h-4 w-4" />
        </div>
      )}
    </div>
  );
}

function StatusItem({
  icon: Icon,
  label,
  status,
  detail,
}: {
  icon: React.ElementType;
  label: string;
  status: VehicleStatus;
  detail?: string;
}) {
  return (
    <div
      className={cn(
        'flex items-center gap-2 rounded-lg p-2',
        status === 'valid' && 'bg-status-valid/10',
        status === 'warning' && 'bg-status-warning/10',
        status === 'critical' && 'bg-status-critical/10',
        status === 'pending' && 'bg-muted'
      )}
    >
      <Icon
        className={cn(
          'h-4 w-4',
          status === 'valid' && 'text-status-valid',
          status === 'warning' && 'text-status-warning',
          status === 'critical' && 'text-status-critical',
          status === 'pending' && 'text-muted-foreground'
        )}
      />
      <div className="min-w-0 flex-1">
        <p className="text-xs text-muted-foreground">{label}</p>
        {detail && <p className="truncate text-xs font-medium">{detail}</p>}
      </div>
    </div>
  );
}
