/**
 * Token Manager
 *
 * Centralized access and refresh token storage.
 * Uses localStorage for persistence across page reloads.
 * Consumed by the auth interceptor and AuthContext.
 */

const ACCESS_TOKEN_KEY = 'iviss_access_token';
const REFRESH_TOKEN_KEY = 'iviss_refresh_token';

/**
 * Retrieve the stored access token.
 */
export function getAccessToken(): string | null {
  return localStorage.getItem(ACCESS_TOKEN_KEY);
}

/**
 * Store a new access token.
 */
export function setAccessToken(token: string): void {
  localStorage.setItem(ACCESS_TOKEN_KEY, token);
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
