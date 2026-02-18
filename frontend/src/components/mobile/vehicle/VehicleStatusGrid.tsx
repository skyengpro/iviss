import React from 'react';
import { useTranslation } from 'react-i18next';
import { StatusBadge } from '@/components/ui/status-badge';
import { AlertCircle, CheckCircle, AlertTriangle, Clock } from 'lucide-react';
import { StatusResults, Status } from '@/openapi-rq/requests/types.gen';

interface VehicleStatusGridProps {
  apiStatus: StatusResults;
}

export const VehicleStatusGrid: React.FC<VehicleStatusGridProps> = ({ apiStatus }) => {
  const { t } = useTranslation();
  return (
    <div className="space-y-3">
      <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
        {t('vehicleResult.legalStatus')}
      </h3>

      <StatusCard
        title={t('vehicleResult.insurance')}
        status={apiStatus.insurance.status}
        provider={apiStatus.insurance.provider || undefined}
        expiryDate={apiStatus.insurance.expiry_date || undefined}
        notes={apiStatus.insurance.notes || undefined}
      />
      <StatusCard
        title={t('vehicleResult.technicalInspection')}
        status={apiStatus.technical.status}
        expiryDate={apiStatus.technical.expiry_date || undefined}
        notes={apiStatus.technical.notes || undefined}
      />
      <StatusCard
        title={t('vehicleResult.wantedStatus')}
        status={apiStatus.police.status}
        notes={apiStatus.police.notes || undefined}
        isAlert={apiStatus.police.is_wanted || apiStatus.police.is_stolen}
      />
      <StatusCard
        title={t('vehicleResult.customsClearance')}
        status={apiStatus.customs.status}
        notes={apiStatus.customs.notes || undefined}
      />
    </div>
  );
};

function StatusCard({
  title,
  status,
  provider,
  expiryDate,
  notes,
  isAlert,
}: {
  title: string;
  status: Status;
  provider?: string;
  expiryDate?: string;
  notes?: string;
  isAlert?: boolean;
}) {
  const { t } = useTranslation();

  const renderNotes = () => {
    if (!notes) return null;
    return notes;
  };

  const getStatusIcon = () => {
    switch (status) {
      case 'valid':
        return <CheckCircle className="h-5 w-5 text-status-valid" />;
      case 'warning':
        return <AlertTriangle className="h-5 w-5 text-status-warning" />;
      case 'critical':
        return <AlertCircle className="h-5 w-5 text-status-critical" />;
      case 'pending':
      default:
        return <Clock className="h-5 w-5 text-muted-foreground" />;
    }
  };

  const getBorderColor = () => {
    switch (status) {
      case 'valid':
        return 'border-status-valid/30';
      case 'warning':
        return 'border-status-warning/30';
      case 'critical':
        return 'border-status-critical/30';
      case 'pending':
      default:
        return 'border-border';
    }
  };

  const getBgColor = () => {
    switch (status) {
      case 'critical':
        return 'bg-status-critical/5';
      case 'warning':
        return 'bg-status-warning/5';
      default:
        return 'bg-card';
    }
  };

  return (
    <div
      className={`rounded-xl border ${getBorderColor()} ${getBgColor()} p-4 ${isAlert ? 'animate-pulse-status' : ''}`}
    >
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-3">
          {getStatusIcon()}
          <div>
            <p className="font-semibold">{title}</p>
            {provider && <p className="text-sm text-muted-foreground">{provider}</p>}
            {expiryDate && (
              <p className="text-sm text-muted-foreground">
                {t('vehicleResult.expires', { date: expiryDate })}
              </p>
            )}
          </div>
        </div>
        <StatusBadge variant={status} size="sm">
          {t(`mobileHistory.${status}`)}
        </StatusBadge>
      </div>
      {notes && (
        <p
          className={`mt-2 text-sm ${status === 'critical' ? 'text-status-critical font-medium' : 'text-muted-foreground'}`}
        >
          {renderNotes()}
        </p>
      )}
    </div>
  );
}
