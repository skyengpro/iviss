import { useCallback } from 'react';
import { useAuth } from '@/hooks/auth/use-auth';
import { useSearchVehicle, useSubmitVehicle } from '../../openapi-rq/queries/queries';
import {
  VehicleSearchRequest,
  CreatePendingSubmissionRequest,
} from '../../openapi-rq/requests/types.gen';

type GeoLocation = {
  latitude: number;
  longitude: number;
};

async function getBrowserLocation(timeoutMs: number): Promise<GeoLocation | null> {
  if (typeof navigator === 'undefined' || !navigator.geolocation) return null;

  return new Promise((resolve) => {
    let settled = false;
    const timeoutId = window.setTimeout(() => {
      if (settled) return;
      settled = true;
      resolve(null);
    }, timeoutMs);

    navigator.geolocation.getCurrentPosition(
      (pos) => {
        if (settled) return;
        settled = true;
        window.clearTimeout(timeoutId);
        resolve({
          latitude: pos.coords.latitude,
          longitude: pos.coords.longitude,
        });
      },
      () => {
        if (settled) return;
        settled = true;
        window.clearTimeout(timeoutId);
        resolve(null);
      },
      {
        enableHighAccuracy: true,
        maximumAge: 30_000,
        timeout: timeoutMs,
      }
    );
  });
}

async function reverseGeocode(
  latitude: number,
  longitude: number,
  timeoutMs: number
): Promise<string | null> {
  if (typeof fetch === 'undefined') return null;

  const controller = new AbortController();
  const timeoutId = window.setTimeout(() => controller.abort(), timeoutMs);

  try {
    const url = new URL('https://nominatim.openstreetmap.org/reverse');
    url.searchParams.set('format', 'jsonv2');
    url.searchParams.set('lat', String(latitude));
    url.searchParams.set('lon', String(longitude));
    url.searchParams.set('zoom', '18');
    url.searchParams.set('addressdetails', '1');

    const res = await fetch(url.toString(), {
      signal: controller.signal,
      headers: {
        Accept: 'application/json',
      },
    });

    if (!res.ok) return null;
    const data = (await res.json()) as { display_name?: string };
    return data.display_name ?? null;
  } catch {
    return null;
  } finally {
    window.clearTimeout(timeoutId);
  }
}

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
      const location =
        request.latitude == null || request.longitude == null
          ? await getBrowserLocation(1500)
          : null;

      const latitude = request.latitude ?? location?.latitude;
      const longitude = request.longitude ?? location?.longitude;
      const address =
        request.address ??
        (latitude != null && longitude != null
          ? await reverseGeocode(latitude, longitude, 1200)
          : undefined);

      // Auto-inject agent info for control logging
      const enrichedRequest = {
        ...request,
        agent_id: user?.id,
        organization_id: user?.organizationId,
        latitude,
        longitude,
        address,
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
