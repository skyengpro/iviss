import { useLocation, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { MobileLayout } from '@/components/layout/MobileLayout';
import { StatusBadge } from '@/components/ui/status-badge';
import { Button } from '@/components/ui/button';
import {
  ArrowLeft,
  Calendar,
  MapPin,
  Clock,
  User,
  Smartphone,
  CheckCircle,
  AlertTriangle,
  AlertCircle,
  Info,
  Car,
  FileText,
} from 'lucide-react';
import { ListControlResponse } from '@/openapi-rq/requests/types.gen';
import { cn } from '@/lib/utils';

export default function MobileHistoryDetail() {
  const location = useLocation();
  const navigate = useNavigate();
  const { t, i18n } = useTranslation();
  const control = location.state?.control as ListControlResponse;

  if (!control) {
    return (
      <MobileLayout title={t('common.error')} hideNavigation>
        <div className="flex flex-col items-center justify-center h-[80vh] p-6 text-center space-y-4">
          <AlertCircle className="h-12 w-12 text-destructive" />
          <h2 className="text-xl font-bold">
            {t('mobileHistory.notFoundTitle', 'History Not Found')}
          </h2>
          <p className="text-muted-foreground">
            {t(
              'mobileHistory.notFoundMessage',
              'The requested history details could not be found.'
            )}
          </p>
          <Button onClick={() => navigate('/mobile/history')}>{t('common.back', 'Back')}</Button>
        </div>
      </MobileLayout>
    );
  }

  const formatDateTime = (isoString: string) => {
    const date = new Date(isoString);
    return date.toLocaleString(i18n.language, {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'valid':
        return <CheckCircle className="h-5 w-5 text-status-valid" />;
      case 'warning':
        return <AlertTriangle className="h-5 w-5 text-status-warning" />;
      case 'critical':
        return <AlertCircle className="h-5 w-5 text-status-critical" />;
      default:
        return <Info className="h-5 w-5 text-muted-foreground" />;
    }
  };

  const display = (value?: string | number | null) => {
    if (value === undefined || value === null || value === '') return '-';
    return String(value);
  };

  const formatBrandModel = (brand?: string | null, model?: string | null) => {
    const value = [brand, model].filter(Boolean).join(' ');
    return value || '-';
  };

  return (
    <MobileLayout title={t('mobileHistory.detailTitle', 'Control Detail')} hideNavigation>
      <div className="p-4 space-y-6 pb-20">
        {/* Back Button */}
        <button
          onClick={() => navigate(-1)}
          className="flex items-center gap-2 text-muted-foreground hover:text-foreground mb-2"
        >
          <ArrowLeft className="h-4 w-4" />
          {t('common.back', 'Back')}
        </button>

        {/* Header Card */}
        <div className="rounded-2xl border border-border bg-card p-6 shadow-sm">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1">
                {t('mobileHistory.plateNumber', 'Plate Number')}
              </p>
              <h1 className="font-mono text-3xl font-bold tracking-widest uppercase">
                {control.plate_number}
              </h1>
            </div>
            <StatusBadge variant={control.status} size="lg">
              {t(`mobileHistory.${control.status}`)}
            </StatusBadge>
          </div>

          <div className="mt-6 grid grid-cols-1 gap-4 border-t pt-6">
            <InfoRow
              icon={Clock}
              label={t('mobileHistory.timestamp', 'Date & Time')}
              value={formatDateTime(control.timestamp)}
            />
            <InfoRow
              icon={MapPin}
              label={t('mobileHistory.location', 'Location')}
              value={control.location.address || 'Unknown Address'}
            />
          </div>
        </div>

        {/* Vehicle Information */}
        {control.vehicle && (
          <div className="space-y-3">
            <h3 className="text-lg font-bold px-1">
              {t('mobileHistory.vehicleInformation', 'Vehicle Information')}
            </h3>
            <div className="rounded-2xl border border-border bg-card p-4 space-y-4 shadow-sm">
              <div className="grid grid-cols-2 gap-4">
                <InfoRow
                  icon={Car}
                  label={t('mobileHistory.brandModel', 'Brand & Model')}
                  value={formatBrandModel(control.vehicle.brand, control.vehicle.model)}
                />
                <InfoRow
                  icon={Calendar}
                  label={t('mobileHistory.year', 'Year')}
                  value={display(control.vehicle.year)}
                />
                <InfoRow
                  icon={Info}
                  label={t('mobileHistory.color', 'Color')}
                  value={display(control.vehicle.color)}
                />
                <InfoRow
                  icon={FileText}
                  label={t('mobileHistory.chassisNumber', 'Chassis')}
                  value={display(control.vehicle.chassis_number)}
                />
              </div>

              <div className="border-t pt-4">
                <div className="flex items-center gap-2 mb-2">
                  <User className="h-4 w-4 text-muted-foreground" />
                  <span className="text-sm font-bold">
                    {t('mobileHistory.ownerInformation', 'Owner Information')}
                  </span>
                </div>
                <div className="bg-muted/30 rounded-xl p-3">
                  <p className="font-semibold text-sm">{display(control.vehicle.owner.name)}</p>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Verification Results */}
        <div className="space-y-3">
          <h3 className="text-lg font-bold px-1">
            {t('mobileHistory.verificationResults', 'Verification Results')}
          </h3>
          <div className="grid gap-3">
            <ResultCard
              label={t('mobileHistory.registration', 'Registration')}
              status={control.results.registration}
              icon={getStatusIcon(control.results.registration)}
            />
            <ResultCard
              label={t('mobileHistory.insurance', 'Insurance')}
              status={control.results.insurance}
              icon={getStatusIcon(control.results.insurance)}
            />
            <ResultCard
              label={t('mobileHistory.technicalInspection', 'Technical Inspection')}
              status={control.results.technical_inspection}
              icon={getStatusIcon(control.results.technical_inspection)}
            />
            <ResultCard
              label={t('mobileHistory.wantedStatus', 'Wanted Status')}
              status={control.results.wanted_status}
              icon={getStatusIcon(control.results.wanted_status)}
            />
            <ResultCard
              label={t('mobileHistory.customs', 'Customs')}
              status={control.results.customs_status}
              icon={getStatusIcon(control.results.customs_status)}
            />
          </div>
        </div>

        {/* Actions Timeline */}
        {control.actions && control.actions.length > 0 && (
          <div className="space-y-3">
            <h3 className="text-lg font-bold px-1">
              {t('mobileHistory.actionsTaken', 'Actions Taken')}
            </h3>
            <div className="rounded-2xl border border-border bg-card p-4 space-y-4">
              {control.actions.map((action, index) => (
                <div key={index} className="flex gap-4">
                  <div className="flex flex-col items-center">
                    <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10 text-primary font-bold text-sm">
                      {index + 1}
                    </div>
                    {index < control.actions.length - 1 && (
                      <div className="w-px flex-1 bg-border mt-2" />
                    )}
                  </div>
                  <div className="flex-1 pb-4">
                    <p className="font-bold capitalize">{action.action_type}</p>
                    {action.description && (
                      <p className="text-sm text-muted-foreground mt-1">{action.description}</p>
                    )}
                    <p className="text-xs text-muted-foreground mt-2">
                      {formatDateTime(action.timestamp)}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Notes */}
        {control.notes && (
          <div className="space-y-3">
            <h3 className="text-lg font-bold px-1">{t('mobileHistory.notes', 'Agent Notes')}</h3>
            <div className="rounded-2xl border border-border bg-card p-4 italic text-muted-foreground">
              {control.notes}
            </div>
          </div>
        )}

        {/* OCR Confidence */}
        {control.confidence !== undefined && control.confidence !== null && (
          <div className="flex items-center justify-center gap-2 text-xs text-muted-foreground pt-4">
            <Info className="h-3 w-3" />
            <span>
              {t('mobileHistory.scanningConfidence', 'Scanning confidence: {{percent}}%', {
                percent: control.confidence,
              })}
            </span>
          </div>
        )}
      </div>
    </MobileLayout>
  );
}

function InfoRow({
  icon: Icon,
  label,
  value,
}: {
  icon: React.ElementType;
  label: string;
  value: string;
}) {
  return (
    <div className="flex items-start gap-3">
      <div className="mt-0.5 rounded-lg bg-muted p-2">
        <Icon className="h-4 w-4 text-muted-foreground" />
      </div>
      <div>
        <p className="text-xs font-medium text-muted-foreground">{label}</p>
        <p className="text-sm font-semibold">{value}</p>
      </div>
    </div>
  );
}

function ResultCard({
  label,
  status,
  icon,
}: {
  label: string;
  status: string;
  icon: React.ReactNode;
}) {
  const { t } = useTranslation();
  return (
    <div className="flex items-center justify-between rounded-xl border border-border bg-card p-4 transition-all shadow-sm">
      <div className="flex items-center gap-3">
        {icon}
        <span className="font-bold">{label}</span>
      </div>
      <StatusBadge
        variant={status as 'valid' | 'warning' | 'critical' | 'pending'}
        size="sm"
        showIcon={false}
      >
        {t(`mobileHistory.${status}`)}
      </StatusBadge>
    </div>
  );
}
