import { useState, useEffect, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { MobileLayout } from '@/components/layout/MobileLayout';
import { Clock, ArrowLeft } from 'lucide-react';
import { useAuth } from '@/hooks/auth/use-auth';
import { useVehicles } from '@/hooks/api/useVehicles';
import { VehicleSearchResult } from '@/openapi-rq/requests/types.gen';

import { useLogControl } from '@/hooks/api/useLogControl';
import { VehicleHeader } from '@/components/mobile/vehicle/VehicleHeader';
import { VehicleStatusGrid } from '@/components/mobile/vehicle/VehicleStatusGrid';
import { VehicleActionFooter } from '@/components/mobile/vehicle/VehicleActionFooter';
import { VehicleLoadingState, VehicleErrorState } from '@/components/mobile/vehicle/VehicleStates';
import { VehicleNotFound } from '@/components/mobile/vehicle/VehicleNotFound';

export default function MobileVehicleResult() {
  const { plateNumber } = useParams<{ plateNumber: string }>();
  const navigate = useNavigate();
  const { user } = useAuth();
  const { t } = useTranslation();
  const { search, isSearching } = useVehicles();
  const [result, setResult] = useState<VehicleSearchResult | null>(null);
  const [error, setError] = useState<any>(null);

  const performSearch = useCallback(async () => {
    if (!plateNumber) return;
    try {
      setError(null);
      const data = await search({ plate: plateNumber });
      setResult(data.data as VehicleSearchResult);
    } catch (err: any) {
      console.error("Search error:", err);
      console.error("Search error:", err);
      // Handles @hey-api/client-fetch errors
      const status = err.status || (err.body && err.body.status) || (err.response && err.response.status) || 500;
      // Check for code in body (hey-api standard) or top level
      const code = err.code || (err.body && err.body.code) || (err.error && err.error.code) || 'UNKNOWN';
      const message = err.message || (err.body && err.body.message) || (err.error && err.error.message) || 'An error occurred';

      setError({ status, code, message, original: err });
    }
  }, [plateNumber, search]);

  useEffect(() => {
    performSearch();
  }, [performSearch]);

  const { isLoggingControl, controlLogged, logControl } = useLogControl();

  const handleRetry = () => {
    performSearch();
  };

  if (isSearching && !result) {
    return (
      <MobileLayout title={t('vehicleResult.searchingTitle')} hideNavigation>
        <VehicleLoadingState queryTime={0.5} />
      </MobileLayout>
    );
  }

  if (error) {
    // Check if it's 404
    if (error.status === 404 || error.code === 'NOT_FOUND') {
      return (
        <MobileLayout title={t('vehicleResult.notFoundTitle')} hideNavigation>
          <VehicleNotFound plateNumber={plateNumber || ''} />
        </MobileLayout>
      );
    }

    // Check if it's 400 (Bad Request - Invalid Plate)
    if (error.status === 400 || error.code === 'INVALID_PLATE') {
      return (
        <MobileLayout title={t('vehicleResult.errorTitle')} hideNavigation>
          <div className="flex flex-col items-center justify-center h-[60vh] p-6 text-center space-y-4">
            <div className="text-destructive font-bold text-lg">
              {t('vehicleResult.invalidPlateTitle', 'Invalid Plate Format')}
            </div>
            <p className="text-muted-foreground">
              {t('vehicleResult.invalidPlateMessage', 'The plate number format is incorrect. Please check and try again.')}
            </p>
            <button
              onClick={() => navigate('/mobile/search')}
              className="px-4 py-2 bg-primary text-primary-foreground rounded-md"
            >
              {t('common.tryAgain', 'Try Again')}
            </button>
          </div>
        </MobileLayout>
      );
    }

    return (
      <MobileLayout title={t('vehicleResult.errorTitle')} hideNavigation>
        <VehicleErrorState onRetry={handleRetry} />
      </MobileLayout>
    );
  }

  if (!result || !plateNumber) {
    if (!plateNumber) return null; // Should redirect or show error clearly
    // Fallback loading state if we have a plate but no result/error yet (initial mount before effect)
    return (
      <MobileLayout title={t('vehicleResult.searchingTitle')} hideNavigation>
        <VehicleLoadingState queryTime={0.0} />
      </MobileLayout>
    );
  }

  return (
    <MobileLayout title={t('vehicleResult.detailsTitle')} hideNavigation>
      <div className="p-4 space-y-4 pb-32">
        <button
          onClick={() => navigate(-1)}
          className="flex items-center gap-2 text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft className="h-4 w-4" />
          {t('vehicleResult.backButton')}
        </button>

        <VehicleHeader
          plateNumber={plateNumber}
          vehicle={result.vehicle}
          overallStatus={result.status_results.overall_status}
        />

        <VehicleStatusGrid apiStatus={result.status_results} />

        {/* Query Time */}
        <div className="flex items-center justify-center gap-2 text-xs text-muted-foreground">
          <Clock className="h-3 w-3" />
          <span>{t('vehicleResult.queryTime', { time: 0.4 })}</span>
        </div>
      </div>

      <VehicleActionFooter
        controlLogged={controlLogged}
        isLoggingControl={isLoggingControl}
        onLogControl={() => user && logControl(user, plateNumber, result.status_results, result.vehicle)}
        onNewSearch={() => navigate('/mobile/search')}
      />
    </MobileLayout>
  );
}


