/**
 * Auth Interceptor
 *
 * Rules:
 * - Silently refresh the access token when it expires (401 on any protected endpoint)
 * - ONLY log the user out if:
 *   1. The refresh token itself is rejected (admin revoked the session)
 *   2. The backend explicitly returns "Shift ended"
 * - NEVER log out due to: page reload, HMR, network errors, backend restarts
 */

import { getAccessToken, getRefreshToken, setAccessToken, setRefreshToken } from './tokenManager';
import { getDeviceId } from '../device/deviceId';
import { signNonce } from './signatureService';
import { requestRefresh, verifyRefresh } from '@/openapi-rq/requests/services.gen';

const RETRY_HEADER = 'X-Auth-Retry';

const REFRESH_PATHS = [
  '/api/v1/auth/refresh',
  '/api/v1/auth/refresh/verify',
  '/api/v1/auth/request-daily-login',
  '/api/v1/auth/verify-daily-login',
];

let refreshPromise: Promise<string | null> | null = null;
const REFRESH_TIMEOUT_MS = 15_000;

function isDeviceReactivationMessage(message: string): boolean {
  return (
    message.includes('device is not active') ||
    message.includes('device is not registered') ||
    message.includes('device not found or revoked') ||
    message.includes('device suspended') ||
    message.includes('device status: suspended') ||
    message.includes('device status: revoked')
  );
}

function markDeviceReactivationRequired() {
  localStorage.removeItem('iviss_device_activated');
  localStorage.setItem('iviss_forced_logout_reason', 'DEVICE_REACTIVATION_REQUIRED');
}

function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const t = window.setTimeout(() => reject(new Error('Refresh timed out')), ms);
    promise
      .then(resolve)
      .catch(reject)
      .finally(() => clearTimeout(t));
  });
}

async function performTokenRefresh(): Promise<string | null> {
  if (refreshPromise) return refreshPromise;

  const refreshToken = getRefreshToken();
  // Guard against null or the literal string "null" which could be stored
  // when a prior version of the code did localStorage.setItem(key, null).
  if (!refreshToken || refreshToken === 'null') {
    console.warn('[AuthInterceptor] No valid refresh token available');
    return null;
  }

  refreshPromise = withTimeout(
    (async () => {
      try {
        const isAdminRole = (() => {
          try {
            const raw = localStorage.getItem('iviss_session');
            if (!raw) return false;
            const session = JSON.parse(raw);
            const role = session?.user?.role;
            return role === 'admin' || role === 'manager' || role === 'org_admin';
          } catch {
            return false;
          }
        })();

        if (isAdminRole) {
          const res = await requestRefresh({ body: { refreshToken }, throwOnError: false });
          if (res.error || !res.data) {
            console.error('[AuthInterceptor] Token refresh failed:', res.error);
            return null;
          }
          const data = res.data as { accessToken?: string };
          if (data.accessToken) {
            setAccessToken(data.accessToken);
            return data.accessToken;
          }
          return null;
        }

        // Agent: 2-step challenge-response
        const deviceId = await getDeviceId();
        const res1 = await requestRefresh({
          body: { refreshToken, deviceId },
          throwOnError: false,
        });
        if (res1.error || !res1.data) {
          console.error('[AuthInterceptor] Token refresh failed:', res1.error);
          return null;
        }

        const { nonce } = res1.data as { nonce?: string };
        if (!nonce) {
          console.error('[AuthInterceptor] No nonce received');
          return null;
        }

        const signedNonce = await signNonce(nonce);
        const res2 = await verifyRefresh({
          body: { refreshToken, deviceId, signedNonce },
          throwOnError: false,
        });
        if (res2.error || !res2.data) {
          console.error('[AuthInterceptor] Token refresh verification failed:', res2.error);
          return null;
        }

        const { accessToken, refreshToken: nextRT } = res2.data as {
          accessToken?: string;
          refreshToken?: string;
        };
        if (!accessToken) {
          console.error('[AuthInterceptor] No access token received');
          return null;
        }

        setAccessToken(accessToken);
        if (nextRT) setRefreshToken(nextRT);
        return accessToken;
      } catch (error) {
        // Network error, backend down, etc. — return null but don't logout
        console.error('[AuthInterceptor] Token refresh exception:', error);
        return null;
      } finally {
        refreshPromise = null;
      }
    })(),
    REFRESH_TIMEOUT_MS
  );

  return refreshPromise;
}

type HeyApiClient = {
  interceptors: {
    request: { use: (fn: (req: Request) => Promise<Request> | Request) => void };
    response: { use: (fn: (res: Response, req: Request) => Promise<Response> | Response) => void };
  };
};

export function setupAuthInterceptors(
  client: HeyApiClient,
  options: { baseUrl: string; onSessionExpired?: () => void }
): void {
  // Attach token on every request
  client.interceptors.request.use(async (request: Request) => {
    if (request.headers.get('Authorization')) return request;
    const token = getAccessToken();
    if (!token) return request;
    const headers = new Headers(request.headers);
    headers.set('Authorization', `Bearer ${token}`);
    return new Request(request, { headers });
  });

  // Handle 401 responses
  client.interceptors.response.use(async (response: Response, request: Request) => {
    if (response.status !== 401) return response;

    // Unauthenticated request (e.g. login) — not a session issue
    if (!getAccessToken()) {
      return response;
    }

    // Check if this is a refresh endpoint returning 401
    // That means the refresh token itself was rejected (session revoked by admin)
    try {
      const url = new URL(request.url);
      if (REFRESH_PATHS.some((p) => url.pathname.includes(p))) {
        // Check if it's specifically "invalid/expired refresh token" — that means revoked
        try {
          const body = await response.clone().json();
          const msg = body?.message?.toLowerCase() || '';
          if (isDeviceReactivationMessage(msg)) {
            console.warn('[AuthInterceptor] Device reactivation required:', body.message);
            markDeviceReactivationRequired();
            options.onSessionExpired?.();
            return response;
          }
          if (msg.includes('invalid') || msg.includes('expired') || msg.includes('revoked')) {
            console.warn('[AuthInterceptor] Refresh token rejected — session revoked by admin');
            options.onSessionExpired?.();
          }
        } catch {
          /* not JSON */
        }
        return response;
      }
    } catch {
      /* unparseable URL */
    }

    // Prevent infinite retry loop
    if (request.headers.get(RETRY_HEADER)) {
      return response;
    }

    // Try to refresh the access token silently
    const newToken = await performTokenRefresh();

    if (!newToken) {
      // Refresh failed — could be network error (backend restarting) or auth error.
      // Check the original 401 response body to decide whether to logout.
      try {
        const body = await response.clone().json();
        const msg = body?.message?.toLowerCase() || '';
        if (isDeviceReactivationMessage(msg)) {
          console.warn('[AuthInterceptor] Device reactivation required:', body.message);
          markDeviceReactivationRequired();
          options.onSessionExpired?.();
          return response;
        }
        // Only logout on explicit session termination signals
        if (
          msg.includes('shift ended') ||
          msg.includes('session terminated')
        ) {
          console.warn('[AuthInterceptor] Session terminated:', body.message);
          options.onSessionExpired?.();
        }
        // For all other failures (network errors, backend down), keep the session
      } catch {
        /* not JSON — network error, keep session */
      }
      return response;
    }

    // Retry with new token
    const headers = new Headers(request.headers);
    headers.set('Authorization', `Bearer ${newToken}`);
    headers.set(RETRY_HEADER, '1');
    return fetch(new Request(request, { headers }));
  });
}
