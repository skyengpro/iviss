import { useState, useEffect, useCallback, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { MobileLayout } from '@/components/layout/MobileLayout';
import { Clock, ArrowLeft } from 'lucide-react';
import { useAuth } from '@/hooks/auth/use-auth';
import { useVehicles } from '@/hooks/api/useVehicles';
import { VehicleSearchResult } from '@/openapi-rq/requests/types.gen';

import { VehicleHeader } from '@/components/mobile/vehicle/VehicleHeader';
import { VehicleStatusGrid } from '@/components/mobile/vehicle/VehicleStatusGrid';
import { VehicleImageCollapsible } from '@/components/mobile/vehicle/VehicleImageCollapsible';
import { VehicleLoadingState, VehicleErrorState } from '@/components/mobile/vehicle/VehicleStates';
import { VehicleNotFound } from '@/components/mobile/vehicle/VehicleNotFound';
import { Button } from '@/components/ui/button';

export default function MobileVehicleResult() {
  const { plateNumber } = useParams<{ plateNumber: string }>();
  const navigate = useNavigate();
  const { user } = useAuth();
  const { t } = useTranslation();
  const { search, isSearching } = useVehicles();

  interface SearchError {
    status: number;
    code: string;
    message: string;
    original: unknown;
  }

  const [result, setResult] = useState<VehicleSearchResult | null>(null);
  const [error, setError] = useState<SearchError | null>(null);
  const searchedPlateRef = useRef<string | null>(null);

  const performSearch = useCallback(
    async (plate?: string, force = false) => {
      const targetPlate = plate || plateNumber;
      if (!targetPlate) return;

      // Prevent duplicate searches for same plate unless forced
      if (!force && searchedPlateRef.current === targetPlate && (result || error)) {
        return;
      }

      try {
        setError(null);
        searchedPlateRef.current = targetPlate;

        const data = await search({ plate: targetPlate });
        setResult(data.data as VehicleSearchResult);
      } catch (err: unknown) {
        const errorObj = err as {
          status?: number;
          code?: string;
          message?: string;
          body?: { status?: number; code?: string; message?: string };
          response?: { status?: number };
          error?: { code?: string; message?: string };
        };
        console.error('Search error:', err);
        // Handles @hey-api/client-fetch errors
        const status =
          errorObj.status ||
          (errorObj.body && errorObj.body.status) ||
          (errorObj.response && errorObj.response.status) ||
          500;
        const code =
          errorObj.code ||
          (errorObj.body && errorObj.body.code) ||
          (errorObj.error && errorObj.error.code) ||
          'UNKNOWN';
        const message =
          errorObj.message ||
          (errorObj.body && errorObj.body.message) ||
          (errorObj.error && errorObj.error.message) ||
          'An error occurred';

        setError({
          status: Number(status),
          code: String(code),
          message: String(message),
          original: err,
        });
      } finally {
        // Search finished
      }
    },
    [plateNumber, search, result, error]
  ); // Kept result/error here for the "already searched" check, but we'll be careful with useEffect

  useEffect(() => {
    if (plateNumber && searchedPlateRef.current !== plateNumber) {
      performSearch(plateNumber);
    }
  }, [plateNumber, performSearch]);

  const handleRetry = () => {
    performSearch(plateNumber, true);
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
              {t(
                'vehicleResult.invalidPlateMessage',
                'The plate number format is incorrect. Please check and try again.'
              )}
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

        {result.status_results.vehicle_image_url && (
          <VehicleImageCollapsible imageUrl={result.status_results.vehicle_image_url} />
        )}

        <VehicleStatusGrid apiStatus={result.status_results} />

        {/* Query Time */}
        <div className="flex items-center justify-center gap-2 text-xs text-muted-foreground">
          <Clock className="h-3 w-3" />
          <span>{t('vehicleResult.queryTime', { time: 0.4 })}</span>
        </div>
      </div>

      {/* New Search Button Only */}
      <div className="fixed bottom-0 left-0 right-0 border-t border-border bg-background p-4 z-30">
        <Button
          variant="outline"
          className="w-full h-12"
          onClick={() => navigate('/mobile/search')}
        >
          {t('vehicleResult.newSearch')}
        </Button>
      </div>
    </MobileLayout>
  );
}
