/**
 * Auth Interceptor
 *
 * Rules:
 * - Silently refresh the access token when it expires (401 on any protected endpoint)
 * - ONLY log the user out if:
 *   1. The refresh token itself is rejected (admin revoked the session)
 *   2. The backend explicitly returns "Shift ended"
 * - NEVER log out due to: page reload, HMR, network errors, backend restarts,
 *   or a raced/expired refresh nonce (that's a timing issue, not a session issue —
 *   the challenge is retried instead).
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

// Overall time budget for a full refresh, including any nonce-retry attempts.
// Must stay comfortably under the backend's nonce TTL (300s) for a single attempt.
export const REFRESH_TIMEOUT_MS = 30_000;
// A raced/expired nonce is a timing issue, not an auth failure — retry the
// challenge a bounded number of times before giving up.
const MAX_NONCE_ATTEMPTS = 2;

let refreshPromise: Promise<string | null> | null = null;

type AuthErrorCategory =
  | 'NONCE_RETRY'
  | 'SESSION_REVOKED'
  | 'SHIFT_ENDED'
  | 'DEVICE_REACTIVATION'
  | 'UNKNOWN';

function isDeviceReactivationMessage(message: string): boolean {
  return (
    message.includes('device is not registered') ||
    message.includes('device not found or revoked') ||
    message.includes('device suspended') ||
    message.includes('device status: suspended') ||
    message.includes('pending activation')
  );
}

function markDeviceReactivationRequired() {
  localStorage.removeItem('iviss_device_activated');
  localStorage.setItem('iviss_forced_logout_reason', 'DEVICE_REACTIVATION_REQUIRED');
}

/**
 * Classifies a backend error message into an action category. "nonce" is
 * checked first because messages like "Nonce expired or not found" also
 * contain the word "expired" — without this ordering they'd be
 * misclassified as SESSION_REVOKED and trigger a false-positive logout.
 */
function classifyAuthErrorMessage(message: string | undefined | null): AuthErrorCategory {
  const msg = (message || '').toLowerCase();
  if (!msg) return 'UNKNOWN';
  if (msg.includes('nonce')) return 'NONCE_RETRY';
  if (isDeviceReactivationMessage(msg)) return 'DEVICE_REACTIVATION';
  if (msg.includes('shift ended')) return 'SHIFT_ENDED';
  if (
    msg.includes('invalid') ||
    msg.includes('expired') ||
    msg.includes('revoked') ||
    msg.includes('session terminated')
  ) {
    return 'SESSION_REVOKED';
  }
  return 'UNKNOWN';
}

function extractErrorMessage(error: unknown): string | undefined {
  if (error && typeof error === 'object' && 'message' in error) {
    const message = (error as { message?: unknown }).message;
    return typeof message === 'string' ? message : undefined;
  }
  return undefined;
}

function extractErrorCode(error: unknown): string | undefined {
  if (error && typeof error === 'object' && 'code' in error) {
    const code = (error as { code?: unknown }).code;
    return typeof code === 'string' ? code : undefined;
  }
  return undefined;
}

// Structured codes the backend may return on the refresh flow (Phase 2
// hardening). Their string values are the backend's SCREAMING_SNAKE_CASE
// ErrorCode variants, chosen to match these category names exactly.
const STRUCTURED_ERROR_CODES: ReadonlySet<string> = new Set([
  'NONCE_RETRY',
  'SESSION_REVOKED',
  'SHIFT_ENDED',
  'DEVICE_REACTIVATION',
]);

/**
 * Prefers the backend's structured `code` when present (older deployed
 * backends won't send one yet), falling back to text classification.
 */
function classifyAuthError(code: string | undefined, message: string | undefined | null): AuthErrorCategory {
  if (code && STRUCTURED_ERROR_CODES.has(code)) {
    return code as AuthErrorCategory;
  }
  return classifyAuthErrorMessage(message);
}

function isAdminSession(): boolean {
  try {
    const raw = localStorage.getItem('iviss_session');
    if (!raw) return false;
    const session = JSON.parse(raw);
    const role = session?.user?.role;
    return role === 'admin' || role === 'manager' || role === 'org_admin';
  } catch {
    return false;
  }
}

async function performAdminRefresh(refreshToken: string, signal: AbortSignal): Promise<string | null> {
  const res = await requestRefresh({ body: { refreshToken }, throwOnError: false, signal });
  if (res.error || !res.data) {
    console.error('[AuthInterceptor] Admin token refresh failed:', res.error);
    return null;
  }
  const data = res.data as { accessToken?: string };
  if (!data.accessToken) return null;
  setAccessToken(data.accessToken);
  return data.accessToken;
}

/** One attempt at the agent's 2-step challenge-response refresh. */
async function performAgentChallenge(
  refreshToken: string,
  signal: AbortSignal
): Promise<{ accessToken: string } | { category: AuthErrorCategory }> {
  const deviceId = await getDeviceId();

  const res1 = await requestRefresh({ body: { refreshToken, deviceId }, throwOnError: false, signal });
  if (res1.error || !res1.data) {
    return { category: classifyAuthError(extractErrorCode(res1.error), extractErrorMessage(res1.error)) };
  }

  const { nonce } = res1.data as { nonce?: string };
  if (!nonce) return { category: 'UNKNOWN' };

  const signedNonce = await signNonce(nonce);
  const res2 = await verifyRefresh({
    body: { refreshToken, deviceId, signedNonce },
    throwOnError: false,
    signal,
  });
  if (res2.error || !res2.data) {
    return { category: classifyAuthError(extractErrorCode(res2.error), extractErrorMessage(res2.error)) };
  }

  const { accessToken, refreshToken: nextRT } = res2.data as {
    accessToken?: string;
    refreshToken?: string;
  };
  if (!accessToken) return { category: 'UNKNOWN' };

  setAccessToken(accessToken);
  if (nextRT) setRefreshToken(nextRT);
  return { accessToken };
}

/**
 * Runs a full refresh attempt (admin: single call; agent: challenge-response
 * with bounded nonce retries), bounded by a single AbortController covering
 * the whole operation. Never rejects — every failure path resolves to null
 * so callers can rely on `if (!token)` without a try/catch.
 */
async function runTokenRefresh(): Promise<string | null> {
  const refreshToken = getRefreshToken();
  // Guard against null or the literal string "null" which could be stored
  // when a prior version of the code did localStorage.setItem(key, null).
  if (!refreshToken || refreshToken === 'null') {
    console.warn('[AuthInterceptor] No valid refresh token available');
    return null;
  }

  const controller = new AbortController();
  const timer = window.setTimeout(() => controller.abort(), REFRESH_TIMEOUT_MS);

  try {
    if (isAdminSession()) {
      return await performAdminRefresh(refreshToken, controller.signal);
    }

    let lastCategory: AuthErrorCategory = 'UNKNOWN';
    for (let attempt = 1; attempt <= MAX_NONCE_ATTEMPTS; attempt++) {
      const result = await performAgentChallenge(refreshToken, controller.signal);
      if ('accessToken' in result) return result.accessToken;
      lastCategory = result.category;
      if (lastCategory !== 'NONCE_RETRY') break;
      console.warn(
        `[AuthInterceptor] Refresh nonce race/expiry (attempt ${attempt}/${MAX_NONCE_ATTEMPTS}) — retrying challenge`
      );
    }
    if (lastCategory !== 'NONCE_RETRY') {
      console.error('[AuthInterceptor] Token refresh failed:', lastCategory);
    }
    return null;
  } catch (error) {
    // Timeout (AbortError), network error, backend down, etc. — never
    // throw, the caller treats a null token as "keep the session, try later".
    console.error('[AuthInterceptor] Token refresh exception:', error);
    return null;
  } finally {
    clearTimeout(timer);
  }
}

/**
 * Deduplicates concurrent refresh attempts (a wake-from-sleep typically
 * fires several 401s at once) and guarantees `refreshPromise` is always
 * cleared once the attempt settles — regardless of success, failure, or
 * timeout — so a stuck attempt can never poison subsequent refreshes.
 */
export async function performTokenRefresh(): Promise<string | null> {
  if (refreshPromise) return refreshPromise;

  const promise = runTokenRefresh();
  refreshPromise = promise;
  promise.finally(() => {
    if (refreshPromise === promise) {
      refreshPromise = null;
    }
  });

  return promise;
}

type HeyApiClient = {
  interceptors: {
    request: { use: (fn: (req: Request) => Promise<Request> | Request) => void };
    response: { use: (fn: (res: Response, req: Request) => Promise<Response> | Response) => void };
  };
};

// Holds an unconsumed clone of each outgoing request, taken before fetch
// disturbs its body, so a request with a body (POST/PUT/PATCH) can still be
// retried after a token refresh. WeakMap keys are GC'd once the request
// object is no longer referenced elsewhere.
const pendingRequestBodies = new WeakMap<Request, Request>();

export function setupAuthInterceptors(
  client: HeyApiClient,
  options: { baseUrl: string; onSessionExpired?: () => void }
): void {
  // Attach token on every request, and stash a replayable clone before the
  // body gets consumed by fetch.
  client.interceptors.request.use(async (request: Request) => {
    let finalRequest = request;
    if (!request.headers.get('Authorization')) {
      const token = getAccessToken();
      if (token) {
        const headers = new Headers(request.headers);
        headers.set('Authorization', `Bearer ${token}`);
        finalRequest = new Request(request, { headers });
      }
    }

    try {
      pendingRequestBodies.set(finalRequest, finalRequest.clone());
    } catch {
      // Body already unusable for cloning — retry after refresh will be
      // skipped for this request rather than risk a "body already used" error.
    }

    return finalRequest;
  });

  // Handle 401 responses
  client.interceptors.response.use(async (response: Response, request: Request) => {
    if (response.status !== 401) return response;

    // Unauthenticated request (e.g. login) — not a session issue
    if (!getAccessToken()) {
      return response;
    }

    // Check if this is a refresh/daily-login endpoint returning 401. These
    // calls are made directly by performTokenRefresh itself, so we must not
    // try to refresh-and-retry here (infinite recursion) — just classify the
    // failure as a side effect and let the response flow back to the caller.
    try {
      const url = new URL(request.url);
      if (REFRESH_PATHS.some((p) => url.pathname.includes(p))) {
        try {
          const body = await response.clone().json();
          const category = classifyAuthError(body?.code, body?.message);
          if (category === 'DEVICE_REACTIVATION') {
            console.warn('[AuthInterceptor] Device reactivation required:', body.message);
            markDeviceReactivationRequired();
            options.onSessionExpired?.();
          } else if (category === 'SHIFT_ENDED') {
            console.warn('[AuthInterceptor] Shift ended — redirecting to daily-login');
            options.onSessionExpired?.();
          } else if (category === 'SESSION_REVOKED') {
            console.warn('[AuthInterceptor] Refresh token rejected — session revoked by admin');
            options.onSessionExpired?.();
          }
          // NONCE_RETRY / UNKNOWN: not a session problem — performTokenRefresh
          // (if this call originated there) handles the retry/failure itself.
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
      // Refresh failed — could be a network error (backend restarting), a
      // raced nonce, or a genuine auth error. Check the original 401 body to
      // decide whether to log out.
      try {
        const body = await response.clone().json();
        const category = classifyAuthError(body?.code, body?.message);
        if (category === 'DEVICE_REACTIVATION') {
          console.warn('[AuthInterceptor] Device reactivation required:', body.message);
          markDeviceReactivationRequired();
          options.onSessionExpired?.();
          return response;
        }
        if (category === 'SHIFT_ENDED' || category === 'SESSION_REVOKED') {
          console.warn('[AuthInterceptor] Session terminated:', body.message);
          options.onSessionExpired?.();
        }
        // NONCE_RETRY / UNKNOWN / network errors: keep the session, a later
        // request will attempt the refresh again.
      } catch {
        /* not JSON — network error, keep session */
      }
      return response;
    }

    // Retry with new token, using the unconsumed clone taken at request time.
    const cloned = pendingRequestBodies.get(request);
    if (!cloned) {
      // No stashed clone available — surface the original 401 rather than
      // risk a "body already used" error from rebuilding a consumed request.
      return response;
    }
    const headers = new Headers(cloned.headers);
    headers.set('Authorization', `Bearer ${newToken}`);
    headers.set(RETRY_HEADER, '1');
    return fetch(new Request(cloned, { headers }));
  });
}
