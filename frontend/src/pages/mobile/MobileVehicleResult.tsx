import { useParams, useNavigate } from 'react-router-dom';
import { MobileLayout } from '@/components/layout/MobileLayout';
import { Clock, ArrowLeft } from 'lucide-react';
import { useAuth } from '@/contexts/AuthContext';
import { useQuery } from '@tanstack/react-query';
import { mockVehicleService, Vehicle } from '@/services/mockVehicles';
import { mockExternalAPIService } from '@/services/mockExternalAPIs';

import { useLogControl } from '@/hooks/useLogControl';
import { VehicleHeader } from '@/components/mobile/vehicle/VehicleHeader';
import { VehicleStatusGrid } from '@/components/mobile/vehicle/VehicleStatusGrid';
import { VehicleActionFooter } from '@/components/mobile/vehicle/VehicleActionFooter';
import { VehicleLoadingState, VehicleErrorState } from '@/components/mobile/vehicle/VehicleStates';
import { VehicleNotFound } from '@/components/mobile/vehicle/VehicleNotFound';

export default function MobileVehicleResult() {
  const { plateNumber } = useParams<{ plateNumber: string }>();
  const navigate = useNavigate();
  const { user } = useAuth();

  const {
    data: vehicleData,
    isLoading: vehicleLoading,
    error: vehicleError,
    refetch: refetchVehicle,
  } = useQuery({
    queryKey: ['vehicle', plateNumber],
    queryFn: () =>
      plateNumber
        ? mockVehicleService.searchByPlate(plateNumber)
        : Promise.resolve({ found: false } as { found: boolean; vehicle?: Vehicle | null }),
    enabled: !!plateNumber,
  });

  const {
    data: apiStatus,
    isLoading: apiLoading,
    refetch: refetchApi,
  } = useQuery({
    queryKey: ['api-status', plateNumber],
    queryFn: () =>
      plateNumber ? mockExternalAPIService.checkAllSystems(plateNumber) : Promise.resolve(null),
    enabled: !!plateNumber,
  });

  const { isLoggingControl, controlLogged, logControl } = useLogControl();

  const handleRetry = () => {
    refetchVehicle();
    refetchApi();
  };

  const isLoading = vehicleLoading || apiLoading;
  const found = vehicleData?.found || false;
  const vehicle = vehicleData?.vehicle || null;

  if (isLoading) {
    return (
      <MobileLayout title="Searching..." hideNavigation>
        <VehicleLoadingState queryTime={apiStatus?.queryTime} />
      </MobileLayout>
    );
  }

  if (vehicleError || (!isLoading && !vehicleData)) {
    return (
      <MobileLayout title="Error" hideNavigation>
        <VehicleErrorState onRetry={handleRetry} />
      </MobileLayout>
    );
  }

  if (!found || !plateNumber) {
    return (
      <MobileLayout title="Vehicle Not Found" hideNavigation>
        <VehicleNotFound plateNumber={plateNumber} />
      </MobileLayout>
    );
  }

  if (!vehicle || !apiStatus) return null;

  return (
    <MobileLayout title="Vehicle Details" hideNavigation>
      <div className="p-4 space-y-4 pb-32">
        <button
          onClick={() => navigate(-1)}
          className="flex items-center gap-2 text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft className="h-4 w-4" />
          Back
        </button>

        <VehicleHeader
          vehicle={vehicle}
          overallStatus={apiStatus.overallStatus as 'valid' | 'warning' | 'critical'}
        />

        <VehicleStatusGrid apiStatus={apiStatus} />

        {/* Query Time */}
        <div className="flex items-center justify-center gap-2 text-xs text-muted-foreground">
          <Clock className="h-3 w-3" />
          <span>Query completed in {apiStatus.queryTime}ms</span>
        </div>
      </div>

      <VehicleActionFooter
        controlLogged={controlLogged}
        isLoggingControl={isLoggingControl}
        onLogControl={() => user && logControl(user, plateNumber, apiStatus, vehicle)}
        onNewSearch={() => navigate('/mobile/search')}
      />
    </MobileLayout>
  );
}
