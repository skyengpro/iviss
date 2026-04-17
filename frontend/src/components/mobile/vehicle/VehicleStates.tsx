import React from 'react';
import { useTranslation } from 'react-i18next';
import { Shield, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';

export const VehicleLoadingState: React.FC<{ queryTime?: number }> = ({ queryTime }) => {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col items-center justify-center min-h-[60vh] p-4">
      <div className="relative flex items-center justify-center">
        <div className="h-16 w-16 animate-spin rounded-full border-4 border-muted border-t-accent" />
        <img src="/pwa-64x64.png" alt="IVISS" className="absolute h-8 w-8 rounded-full" />
      </div>
      <p className="mt-6 text-lg font-medium">{t('vehicleResult.searchingDatabases')}</p>
      <p className="mt-2 text-sm text-muted-foreground text-center">
        {t('vehicleResult.queryingSystems')}
      </p>
      {!!queryTime && (
        <p className="mt-4 text-xs text-muted-foreground">
          {t('vehicleResult.queryTime', { time: queryTime })}
        </p>
      )}
    </div>
  );
};

export const VehicleErrorState: React.FC<{ onRetry: () => void }> = ({ onRetry }) => {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col items-center justify-center min-h-[60vh] p-4">
      <AlertCircle className="h-16 w-16 text-destructive" />
      <h2 className="mt-4 text-xl font-bold">{t('vehicleResult.searchError')}</h2>
      <p className="mt-2 text-muted-foreground text-center">
        {t('vehicleResult.searchErrorMessage')}
      </p>
      <Button className="mt-6" onClick={onRetry}>
        {t('vehicleResult.tryAgain')}
      </Button>
    </div>
  );
};
