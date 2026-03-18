import { useCallback } from 'react';
import { useAuth } from '@/hooks/auth/use-auth';
import { useSearchVehicle, useSubmitVehicle } from '../../openapi-rq/queries/queries';
import {
  VehicleSearchRequest,
  CreatePendingSubmissionRequest,
} from '../../openapi-rq/requests/types.gen';

export function useVehicles() {
  const { user } = useAuth();
  const {
    mutateAsync: searchMutate,
    isPending: isSearching,
    error: searchError,
  } = useSearchVehicle();

  const {
    mutateAsync: submitMutate,
    isPending: isSubmitting,
    error: submitError,
    isSuccess: submitSuccess,
  } = useSubmitVehicle();

  const search = useCallback(
    async (request: VehicleSearchRequest) => {
      // Auto-inject agent info for control logging
      const enrichedRequest = {
        ...request,
        agent_id: user?.id,
        organization_id: user?.organizationId,
      };
      return searchMutate({
        body: enrichedRequest,
        throwOnError: true,
      });
    },
    [searchMutate, user]
  );

  const submit = useCallback(
    async (request: CreatePendingSubmissionRequest) => {
      return submitMutate({
        body: request,
        throwOnError: true,
      });
    },
    [submitMutate]
  );

  return {
    search,
    isSearching,
    searchError,

    submit,
    isSubmitting,
    submitError,
    submitSuccess,
  };
}
