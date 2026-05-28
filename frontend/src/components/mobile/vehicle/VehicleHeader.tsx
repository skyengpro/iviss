import React from 'react';
import { useTranslation } from 'react-i18next';
import { Car, FileText, User } from 'lucide-react';
import { StatusBadge } from '@/components/ui/status-badge';
import { VehicleInfo, Status } from '@/openapi-rq/requests/types.gen';

interface VehicleHeaderProps {
  plateNumber: string;
  vehicle: VehicleInfo;
  overallStatus: Status;
}

export const VehicleHeader: React.FC<VehicleHeaderProps> = ({
  plateNumber,
  vehicle,
  overallStatus,
}) => {
  const { t } = useTranslation();
  const display = (value?: string | number | null) => {
    if (value === undefined || value === null || value === '') return '-';
    return String(value);
  };

  return (
    <div className="rounded-xl border border-border bg-card overflow-hidden animate-slide-up">
      <div className="bg-primary p-4 text-primary-foreground">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xs opacity-70">{t('vehicleResult.plateNumber')}</p>
            <p className="text-2xl font-bold tracking-widest font-mono">{plateNumber}</p>
          </div>
          <StatusBadge variant={overallStatus === 'pending' ? 'warning' : overallStatus} size="lg">
            {t(`mobileHistory.${overallStatus}`)}
          </StatusBadge>
        </div>
      </div>

      <div className="p-4 space-y-4">
        {/* Vehicle Details */}
        <div className="grid grid-cols-2 gap-4">
          <DetailItem icon={Car} label={t('vehicleResult.brand')} value={display(vehicle.brand)} />
          <DetailItem icon={Car} label={t('vehicleResult.model')} value={display(vehicle.model)} />
          <DetailItem icon={FileText} label={t('vehicleResult.year')} value={display(vehicle.year)} />
          <DetailItem
            icon={FileText}
            label={t('vehicleResult.power')}
            value={display(vehicle.engine_power)}
          />
        </div>

        <div className="border-t border-border pt-4">
          <DetailItem
            icon={FileText}
            label={t('vehicleResult.chassisNumber')}
            value={display(vehicle.chassis_number)}
            fullWidth
          />
        </div>

        <div className="border-t border-border pt-4">
          <DetailItem
            icon={User}
            label={t('vehicleResult.owner')}
            value={display(vehicle.owner.name)}
            fullWidth
          />
          <p className="text-sm text-muted-foreground mt-1 ml-8">{vehicle.owner.address || ''}</p>
        </div>
      </div>
    </div>
  );
};

function DetailItem({
  icon: Icon,
  label,
  value,
  fullWidth,
}: {
  icon: React.ElementType;
  label: string;
  value: string;
  fullWidth?: boolean;
}) {
  return (
    <div className={`flex items-start gap-2 ${fullWidth ? '' : ''}`}>
      <Icon className="h-4 w-4 text-muted-foreground mt-0.5" />
      <div>
        <p className="text-xs text-muted-foreground">{label}</p>
        <p className="font-medium">{value}</p>
      </div>
    </div>
  );
}
