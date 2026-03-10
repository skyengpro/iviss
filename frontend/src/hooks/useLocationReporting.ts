import { useEffect, useRef } from 'react';
import { useGeolocation } from './useGeolocation';
import { useUpdateLocation } from '../openapi-rq/queries/queries';
import { useAuth } from './auth/use-auth';

export function useLocationReporting(intervalMs = 30000) {
  const { user } = useAuth();
  const { lat, lng, error } = useGeolocation({
    enableHighAccuracy: true,
    timeout: 10000,
    maximumAge: 0,
  });

  const { mutate: updateLocation } = useUpdateLocation();
  const lastReportedRef = useRef<{ lat: number; lng: number } | null>(null);
  const lastTimestampRef = useRef<number>(0);

  useEffect(() => {
    // Only report if we have a user (is an agent) and valid coordinates
    if (!user || user.role !== 'agent' || lat === null || lng === null) {
      return;
    }

    const now = Date.now();
    const shouldReport = () => {
      if (!lastReportedRef.current) return true;

      // Report if moved more than ~10 meters (roughly 0.0001 degrees)
      const dist = Math.sqrt(
        Math.pow(lat - lastReportedRef.current.lat, 2) +
          Math.pow(lng - lastReportedRef.current.lng, 2)
      );

      if (dist > 0.0001) return true;

      // Or if interval has passed
      if (now - lastTimestampRef.current > intervalMs) return true;

      return false;
    };

    if (shouldReport()) {
      updateLocation(
        {
          body: {
            agentId: user.id,
            latitude: lat,
            longitude: lng,
          },
        },
        {
          onSuccess: () => {
            lastReportedRef.current = { lat, lng };
            lastTimestampRef.current = now;
            console.log(`[Location] Reported: ${lat}, ${lng}`);
          },
          onError: (err) => {
            console.error('[Location] Failed to report:', err);
          },
        }
      );
    }
  }, [lat, lng, user, updateLocation, intervalMs]);

  return { lat, lng, error };
}
