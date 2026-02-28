import React from 'react';
import { Button } from '@/components/ui/button';
import { AlertCircle, CheckCircle, AlertTriangle } from 'lucide-react';
import { cn } from '@/lib/utils';
import { DetectedPlate } from '@/hooks/feature/usePlateScanner';
import { useTranslation } from 'react-i18next';

interface ScanResultCardProps {
  detectedPlate: DetectedPlate;
  isEditing: boolean;
  editedPlate: string;
  onEditToggle: () => void;
  onEditChange: (value: string) => void;
  onRetry: () => void;
  onConfirm: () => void;
}

export const ScanResultCard: React.FC<ScanResultCardProps> = ({
  detectedPlate,
  isEditing,
  editedPlate,
  onEditToggle,
  onEditChange,
  onRetry,
  onConfirm,
}) => {
  const { t } = useTranslation();
  return (
    <div className="absolute inset-x-4 bottom-6 animate-slide-up rounded-2xl bg-card p-5 shadow-2xl z-20 border border-border/50">
      <div className="mb-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div
            className={cn(
              'flex h-10 w-10 items-center justify-center rounded-full',
              detectedPlate.status === 'critical'
                ? 'bg-status-critical/10 text-status-critical'
                : detectedPlate.status === 'warning' || detectedPlate.confidence < 80
                  ? 'bg-status-warning/10 text-status-warning'
                  : 'bg-status-valid/10 text-status-valid'
            )}
          >
            {detectedPlate.status === 'critical' ? (
              <AlertCircle className="h-5 w-5" />
            ) : detectedPlate.confidence >= 80 ? (
              <CheckCircle className="h-5 w-5" />
            ) : (
              <AlertTriangle className="h-5 w-5" />
            )}
          </div>
          <div>
            <p className="text-sm text-muted-foreground">{t('mobileScan.detectedPlate')}</p>
            {isEditing ? (
              <input
                type="text"
                value={editedPlate}
                onChange={(e) => onEditChange(e.target.value.toUpperCase())}
                className="text-2xl font-bold tracking-widest bg-transparent border-b-2 border-accent focus:outline-none"
                autoFocus
              />
            ) : (
              <p className="text-2xl font-bold tracking-widest">{detectedPlate.plateNumber}</p>
            )}
          </div>
        </div>
        <div className="text-right">
          <p className="text-sm text-muted-foreground">{t('mobileScan.confidence')}</p>
          <p
            className={cn(
              'text-lg font-semibold',
              detectedPlate.confidence >= 80 ? 'text-status-valid' : 'text-status-warning'
            )}
          >
            {Math.round(detectedPlate.confidence)}%
          </p>
        </div>
      </div>

      {detectedPlate.status === 'critical' && (
        <div className="mb-4 rounded-lg bg-status-critical/10 p-2.5 text-status-critical">
          <div className="flex items-center gap-2">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span className="text-xs font-semibold">{t('mobileScan.alertFlaggedVehicle')}</span>
          </div>
        </div>
      )}

      <div className="flex gap-3 mb-3">
        <Button variant="outline" onClick={onEditToggle} className="flex-1">
          {isEditing ? t('mobileScan.cancelEdit') : t('mobileScan.editPlate')}
        </Button>
        <Button variant="outline" onClick={onRetry} className="flex-1">
          {t('mobileScan.retry')}
        </Button>
      </div>

      <Button
        onClick={onConfirm}
        className="w-full bg-accent text-accent-foreground hover:bg-accent/90"
      >
        {t('mobileScan.confirmAndSearch')}
      </Button>
    </div>
  );
};
