import { useCallback } from 'react';
import { useSearchVehicle, useSubmitVehicle } from '../../openapi-rq/queries/queries';
import {
  VehicleSearchRequest,
  CreatePendingSubmissionRequest,
} from '../../openapi-rq/requests/types.gen';

export function useVehicles() {
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
      return searchMutate({
        body: request,
        throwOnError: true,
      });
    },
    [searchMutate]
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
