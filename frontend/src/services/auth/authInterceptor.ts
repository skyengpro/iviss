/**
 * Auth Interceptor
 *
 * Centralizes authentication logic for the @hey-api/client-fetch API client:
 *
 * 1. **Request interceptor**: Attaches `Authorization: Bearer <token>` header.
 * 2. **Response interceptor**: Detects 401 responses and performs the
 *    challenge-response token refresh flow described in User_registration.md §5:
 *      - POST /auth/refresh with refresh_token
 *      - Receive nonce challenge
 *      - Sign nonce with device private key (ES256)
 *      - Receive new access token
 *      - Retry the original request
 *
 * Prevents infinite retry loops via a single-attempt guard.
 */

import { getAccessToken, getRefreshToken, setAccessToken, setRefreshToken } from './tokenManager';
import { getDeviceId } from '../device/deviceId';
import { signNonce } from './signatureService';
import { requestRefresh, verifyRefresh } from '@/openapi-rq/requests/services.gen';

// Custom header used to mark a request as a retry to prevent infinite loops
const RETRY_HEADER = 'X-Auth-Retry';

const REFRESH_PATHS = ['/auth/refresh', '/auth/refresh/verify', '/auth/request-daily-login', '/auth/verify-daily-login'];

// Module-level promise to track an ongoing refresh operation
let refreshPromise: Promise<string | null> | null = null;

const REFRESH_TIMEOUT_MS = 10_000;

function withTimeout<T>(promise: Promise<T>, timeoutMs: number): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timeout = window.setTimeout(() => {
      reject(new Error('Token refresh timed out'));
    }, timeoutMs);

    promise
      .then((value) => resolve(value))
      .catch((err) => reject(err))
      .finally(() => window.clearTimeout(timeout));
  });
}

/**
 * Perform the full token refresh with device signature challenge-response.
 * Uses a shared promise to ensure only one refresh happens at a time even
 * if multiple requests fail with 401 simultaneously.
 *
 * @param baseUrl - The API base URL
 * @returns The new access token, or null if refresh failed
 */
async function performTokenRefresh(baseUrl: string): Promise<string | null> {
  // If a refresh is already in progress, wait for it instead of starting a new one
  if (refreshPromise) {
    return refreshPromise;
  }

  const refreshToken = getRefreshToken();
  if (!refreshToken) {
    return null;
  }

  // Create the refresh promise
  refreshPromise = withTimeout(
    (async () => {
      try {
        const deviceId = await getDeviceId();
        console.log('--- AuthInterceptor: Starting Token Refresh ---');
        console.log('Base URL:', baseUrl);
        console.log('Device ID:', deviceId);
        console.log('Refresh Token length:', refreshToken.length);

        // Step 1: Send refresh token to get nonce challenge
        const refreshResponse = await requestRefresh({
          body: {
            refreshToken: refreshToken,
            deviceId: deviceId,
          },
          throwOnError: false,
        });

        if (refreshResponse.error || !refreshResponse.data) {
          console.warn('AuthInterceptor: POST /auth/refresh failed');
          return null;
        }

        const { nonce } = refreshResponse.data as { nonce?: string };
        console.log('Received Nonce:', nonce);

        if (!nonce) {
          console.warn('AuthInterceptor: No nonce in refresh response');
          return null;
        }

        // Step 2: Sign the nonce with the device private key
        console.log('Signing nonce...');
        const signedNonce = await signNonce(nonce);
        console.log('Nonce signed successfully');

        // Step 3: Send signed nonce to complete the challenge
        console.log('Sending signed nonce for verification...');
        const verifyResponse = await verifyRefresh({
          body: {
            refreshToken: refreshToken,
            deviceId: deviceId,
            signedNonce: signedNonce,
          },
          throwOnError: false,
        });

        if (verifyResponse.error || !verifyResponse.data) {
          console.warn('AuthInterceptor: POST /auth/refresh/verify failed');
          return null;
        }

        const { accessToken, refreshToken: nextRefreshToken } = verifyResponse.data as {
          accessToken?: string;
          refreshToken?: string;
        };
        if (!accessToken) {
          console.warn('AuthInterceptor: No accessToken in verify response');
          return null;
        }

        // Store the new access token
        setAccessToken(accessToken);

        // Persist rotated refresh token if backend returns one
        if (typeof nextRefreshToken === 'string' && nextRefreshToken.length > 0) {
          setRefreshToken(nextRefreshToken);
        }
        return accessToken;
      } catch (err) {
        console.error('AuthInterceptor: Unexpected error during refresh:', err);
        return null;
      } finally {
        // Clear the promise when done so future 401s can trigger a new refresh if needed
        refreshPromise = null;
      }
    })(),
    REFRESH_TIMEOUT_MS
  );

  return refreshPromise;
}

/**
 * Register auth interceptors on the provided hey-api client.
 *
 * @param client - The hey-api client instance from services.gen.ts
 * @param options - Configuration options
 * @param options.baseUrl - API base URL for refresh calls
 * @param options.onSessionExpired - Callback when refresh fails (e.g. redirect to login)
 */
type HeyApiClient = {
  interceptors: {
    request: { use: (fn: (request: Request) => Promise<Request> | Request) => void };
    response: {
      use: (fn: (response: Response, request: Request) => Promise<Response> | Response) => void;
    };
  };
};

export function setupAuthInterceptors(
  client: HeyApiClient,
  options: {
    baseUrl: string;
    onSessionExpired?: () => void;
  }
): void {
  // --- Request Interceptor: Attach Bearer token ---
  client.interceptors.request.use(async (request: Request) => {
    // Preserve an explicit Authorization header set by the caller.
    // This is important for flows that already have a freshly issued token
    // before the shared token store is updated.
    const existingAuth = request.headers.get('Authorization');
    if (existingAuth) {
      return request;
    }

    const token = getAccessToken();
    if (token) {
      // Clone the request to add the Authorization header
      const headers = new Headers(request.headers);
      headers.set('Authorization', `Bearer ${token}`);
      return new Request(request, { headers });
    }
    return request;
  });

  // --- Response Interceptor: Handle 401 + refresh ---
  client.interceptors.response.use(async (response: Response, request: Request) => {
    // Only handle 401 Unauthorized
    if (response.status !== 401) {
      return response;
    }

    // Never attempt refresh while calling refresh endpoints; avoids recursion.
    try {
      const url = new URL(request.url);
      if (REFRESH_PATHS.includes(url.pathname)) {
        return response;
      }
    } catch {
      // If request.url isn't parseable, fall through to the normal logic.
    }

    // Prevent infinite retry: if this is already a retry, give up
    if (request.headers.get(RETRY_HEADER)) {
      console.warn('AuthInterceptor: 401 loop detected, giving up without clearing tokens');
      return response;
    }

    // Attempt token refresh with device signature
    const newToken = await performTokenRefresh(options.baseUrl);

    if (!newToken) {
      // Refresh failed — session is expired
      console.error('AuthInterceptor: Token refresh failed; session expired');
      options.onSessionExpired?.();
      return response;
    }

    // Retry the original request with the new token
    const retryHeaders = new Headers(request.headers);
    retryHeaders.set('Authorization', `Bearer ${newToken}`);
    retryHeaders.set(RETRY_HEADER, '1');

    const retryRequest = new Request(request, { headers: retryHeaders });
    return fetch(retryRequest);
  });
}
