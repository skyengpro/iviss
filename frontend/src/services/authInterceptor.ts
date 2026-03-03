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

import { getAccessToken, getRefreshToken, setAccessToken, clearTokens } from './tokenManager';
import { getDeviceId } from './deviceId';
import { signNonce } from './signatureService';

// Custom header used to mark a request as a retry to prevent infinite loops
const RETRY_HEADER = 'X-Auth-Retry';

// Module-level promise to track an ongoing refresh operation
let refreshPromise: Promise<string | null> | null = null;

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
  refreshPromise = (async () => {
    try {
      const deviceId = await getDeviceId();

      // Step 1: Send refresh token to get nonce challenge
      const refreshResponse = await fetch(`${baseUrl}/auth/refresh`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          refresh_token: refreshToken,
          device_id: deviceId,
        }),
      });

      if (!refreshResponse.ok) {
        return null;
      }

      const { nonce } = await refreshResponse.json();
      if (!nonce) {
        return null;
      }

      // Step 2: Sign the nonce with the device private key
      const signedNonce = await signNonce(nonce);

      // Step 3: Send signed nonce to complete the challenge
      const verifyResponse = await fetch(`${baseUrl}/auth/refresh/verify`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          refresh_token: refreshToken,
          device_id: deviceId,
          signed_nonce: signedNonce,
        }),
      });

      if (!verifyResponse.ok) {
        return null;
      }

      const { access_token } = await verifyResponse.json();
      if (!access_token) {
        return null;
      }

      // Store the new access token
      setAccessToken(access_token);
      return access_token;
    } catch {
      return null;
    } finally {
      // Clear the promise when done so future 401s can trigger a new refresh if needed
      refreshPromise = null;
    }
  })();

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
// eslint-disable-next-line @typescript-eslint/no-explicit-any
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

    // Prevent infinite retry: if this is already a retry, give up
    if (request.headers.get(RETRY_HEADER)) {
      clearTokens();
      options.onSessionExpired?.();
      return response;
    }

    // Attempt token refresh with device signature
    const newToken = await performTokenRefresh(options.baseUrl);

    if (!newToken) {
      // Refresh failed — session is expired
      clearTokens();
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
