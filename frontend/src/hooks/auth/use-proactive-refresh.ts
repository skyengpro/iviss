import { useEffect } from 'react';
import { focusManager } from '@tanstack/react-query';
import { getAccessToken, getRefreshToken, isTokenExpired } from '@/services/auth/tokenManager';
import { performTokenRefresh } from '@/services/auth/authInterceptor';

// Refresh a bit before actual expiry so the very first request after
// returning to the foreground doesn't itself have to wait on a 401 round-trip.
const EXPIRY_LEEWAY_SECS = 120;

/** Mirrors TanStack Query's own default focus listener (visibilitychange-only). */
function defaultFocusSetup(setFocused: (focused?: boolean) => void) {
  if (typeof window === 'undefined' || !window.addEventListener) return undefined;
  const listener = () => setFocused();
  window.addEventListener('visibilitychange', listener, false);
  return () => window.removeEventListener('visibilitychange', listener);
}

/**
 * Refreshes the access token proactively when the app returns to the
 * foreground (tab/app switch, phone wake from sleep), BEFORE React Query's
 * focus-triggered refetches fire. Without this, a stale token causes a burst
 * of simultaneous 401s exactly when the network is least reliable (radio
 * reconnecting after sleep) — the scenario behind the "screens go blank
 * after the phone wakes up" bug.
 *
 * Replaces TanStack's default focus listener with one that awaits the
 * refresh first, then notifies query observers — same visibilitychange
 * trigger, just reordered so refetches always see a fresh token.
 */
export function useProactiveRefresh(isAuthenticated: boolean): void {
  useEffect(() => {
    if (!isAuthenticated) return undefined;

    focusManager.setEventListener((setFocused) => {
      if (typeof window === 'undefined' || !window.addEventListener) return undefined;

      const onVisibilityChange = () => {
        void (async () => {
          if (document.visibilityState !== 'visible') {
            setFocused();
            return;
          }
          const accessToken = getAccessToken();
          const refreshToken = getRefreshToken();
          const hasRefreshableSession =
            !!accessToken && !!refreshToken && refreshToken !== 'null';
          if (hasRefreshableSession && isTokenExpired(accessToken, EXPIRY_LEEWAY_SECS)) {
            await performTokenRefresh();
          }
          setFocused();
        })();
      };

      window.addEventListener('visibilitychange', onVisibilityChange, false);
      return () => window.removeEventListener('visibilitychange', onVisibilityChange);
    });

    return () => {
      focusManager.setEventListener(defaultFocusSetup);
    };
  }, [isAuthenticated]);
}
