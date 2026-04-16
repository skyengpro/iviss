import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from '@/hooks/ui/use-toast';
import {
  UserProfile,
  VehicleInfo,
  StatusResults,
  CreateControlData,
} from '@/openapi-rq/requests/types.gen';
import { useCreateControl } from '@/openapi-rq/queries/queries';

export interface LocationContext {
  latitude: number | null;
  longitude: number | null;
  address: string;
}

export function useLogControl() {
  const { t } = useTranslation();
  const createControlMutation = useCreateControl();
  const [isLoggingControl, setIsLoggingControl] = useState(false);
  const [controlLogged, setControlLogged] = useState(false);

  const logControl = async (
    user: UserProfile,
    plateNumber: string,
    statusResults: StatusResults,
    vehicle: VehicleInfo | null,
    location?: LocationContext,
    identificationMode: 'manual' | 'photo' | 'live' = 'manual'
  ) => {
    if (!user || !plateNumber || !statusResults) return;

    setIsLoggingControl(true);

    try {
      const payload: CreateControlData['body'] = {
        plate_number: plateNumber,
        agent_id: user.id || '',
        organization_id: user.organizationId || '',
        latitude: location?.latitude ?? null,
        longitude: location?.longitude ?? null,
        address: location?.address || null,
        identification_mode: identificationMode,
        ocr_confidence: 1.0,
        results: {
          registration: statusResults.overall_status,
          insurance: statusResults.insurance.status,
          technical_inspection: statusResults.technical.status,
          wanted_status: statusResults.police.status,
          customs_status: statusResults.customs.status,
        },
        notes: null,
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
