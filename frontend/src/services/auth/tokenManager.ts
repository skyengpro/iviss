/**
 * Token Manager
 *
 * Centralized access and refresh token storage.
 * Uses localStorage for persistence across page reloads.
 * Consumed by the auth interceptor and AuthContext.
 */

const ACCESS_TOKEN_KEY = 'iviss_access_token';
const REFRESH_TOKEN_KEY = 'iviss_refresh_token';
const SESSION_KEY = 'iviss_session';

/**
 * Retrieve the stored access token.
 */
export function getAccessToken(): string | null {
  return localStorage.getItem(ACCESS_TOKEN_KEY);
}

/**
 * Store a new access token and sync it into the session object so
 * page reloads pick up the refreshed token instead of the expired one.
 */
export function setAccessToken(token: string): void {
  localStorage.setItem(ACCESS_TOKEN_KEY, token);
  // Keep the session object in sync so initIdentity on next page load
  // sees a valid token and doesn't clear the session prematurely.
  try {
    const raw = localStorage.getItem(SESSION_KEY);
    if (raw) {
      const session = JSON.parse(raw);
      session.accessToken = token;
      localStorage.setItem(SESSION_KEY, JSON.stringify(session));
    }
  } catch {
    // ignore
  }
}

/**
 * Retrieve the stored refresh token.
 */
export function getRefreshToken(): string | null {
  return localStorage.getItem(REFRESH_TOKEN_KEY);
}

/**
 * Store a new refresh token.
 */
export function setRefreshToken(token: string): void {
  localStorage.setItem(REFRESH_TOKEN_KEY, token);
}

export function clearAccessToken(): void {
  localStorage.removeItem(ACCESS_TOKEN_KEY);
}

/**
 * Clear both tokens. Used on logout or when refresh fails.
 */
export function clearTokens(): void {
  localStorage.removeItem(ACCESS_TOKEN_KEY);
  localStorage.removeItem(REFRESH_TOKEN_KEY);
}
