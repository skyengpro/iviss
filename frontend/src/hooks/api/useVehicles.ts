import { useCallback } from 'react';
import { useSearchVehicle, useSubmitVehicle } from '../../openapi-rq/queries/queries';
import {
  VehicleSearchRequest,
  CreatePendingSubmissionRequest,
} from '../../openapi-rq/requests/types.gen';

export function useVehicles() {
  const searchMutation = useSearchVehicle();
  const submitMutation = useSubmitVehicle();

  const search = useCallback(
    async (request: VehicleSearchRequest) => {
      return searchMutation.mutateAsync({
        body: request,
        throwOnError: true,
      });
    },
    [searchMutation]
  );

  const submit = useCallback(
    async (request: CreatePendingSubmissionRequest) => {
      return submitMutation.mutateAsync({
        body: request,
        throwOnError: true,
      });
    },
    [submitMutation]
  );

  return {
    search,
    isSearching: searchMutation.isPending,
    searchError: searchMutation.error,

    submit,
    isSubmitting: submitMutation.isPending,
    submitError: submitMutation.error,
    submitSuccess: submitMutation.isSuccess,
  };
}
