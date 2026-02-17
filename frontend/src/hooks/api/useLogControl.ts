import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from '@/hooks/ui/use-toast';
import { UserProfile, VehicleInfo, StatusResults, CreateControlData } from '@/openapi-rq/requests/types.gen';
import { useCreateControl } from '@/openapi-rq/queries/queries';

export function useLogControl() {
  const { t } = useTranslation();
  const createControlMutation = useCreateControl();
  const [isLoggingControl, setIsLoggingControl] = useState(false);
  const [controlLogged, setControlLogged] = useState(false);

  const logControl = async (
    user: UserProfile,
    plateNumber: string,
    statusResults: StatusResults,
    vehicle: VehicleInfo | null
  ) => {
    if (!user || !plateNumber || !statusResults) return;

    setIsLoggingControl(true);

    try {
      const payload: CreateControlData['body'] = {
        plate_number: plateNumber,
        agent_id: user.id || '', // Ensure UUID or handle empty
        organization_id: user.organizationId || '',
        // Hardcoded for now as per previous mock, or get from device location if available
        latitude: 48.8566,
        longitude: 2.3522,
        address: 'Highway A1, KM 42',
        identification_mode: 'manual', // or pass from caller
        ocr_confidence: 1.0,
        results: {
          registration: statusResults.overall_status,
          insurance: statusResults.insurance.status,
          technical_inspection: statusResults.technical.status,
          wanted_status: statusResults.police.status,
          customs_status: statusResults.customs.status,
        },
        notes: "Logged via mobile app",
      };

      await createControlMutation.mutateAsync({ body: payload });

      setControlLogged(true);
      toast({
        title: t('logControl.successTitle'),
        description: t('logControl.successDescription', { plateNumber }),
      });
      return true;
    } catch (error) {
      toast({
        title: t('logControl.errorTitle'),
        description: t('logControl.errorDescription'),
        variant: 'destructive',
      });
      return false;
    } finally {
      setIsLoggingControl(false);
    }
  };

  return {
    isLoggingControl,
    controlLogged,
    logControl,
    setControlLogged,
  };
}

