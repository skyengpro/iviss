/**
 * Token Manager
 *
 * Centralized access and refresh token storage.
 * Uses localStorage for persistence across page reloads.
 */

const ACCESS_TOKEN_KEY = 'iviss_access_token';
const REFRESH_TOKEN_KEY = 'iviss_refresh_token';
const SESSION_KEY = 'iviss_session';

// Event to notify AuthContext when tokens are refreshed
const TOKEN_REFRESHED_EVENT = 'iviss:token-refreshed';

export function getAccessToken(): string | null {
  return localStorage.getItem(ACCESS_TOKEN_KEY);
}

export function setAccessToken(token: string, notifyContext = true): void {
  localStorage.setItem(ACCESS_TOKEN_KEY, token);
  // Keep the session object in sync so initIdentity on next page load
  // sees a valid token and doesn't clear the session prematurely.
  try {
    const raw = localStorage.getItem(SESSION_KEY);
    if (raw) {
      const session = JSON.parse(raw);
      session.accessToken = token;
      localStorage.setItem(SESSION_KEY, JSON.stringify(session));

      // Notify AuthContext to update its state with the new token
      if (notifyContext) {
        window.dispatchEvent(
          new CustomEvent(TOKEN_REFRESHED_EVENT, {
            detail: { accessToken: token, session },
          })
        );
      }
    }
  } catch {
    // ignore
  }
}

export function getRefreshToken(): string | null {
  return localStorage.getItem(REFRESH_TOKEN_KEY);
}

export function setRefreshToken(token: string): void {
  localStorage.setItem(REFRESH_TOKEN_KEY, token);
  // Keep the session object in sync
  try {
    const raw = localStorage.getItem(SESSION_KEY);
    if (raw) {
      const session = JSON.parse(raw);
      session.refreshToken = token;
      localStorage.setItem(SESSION_KEY, JSON.stringify(session));
    }
  } catch {
    // ignore
  }
}

export function clearAccessToken(): void {
  localStorage.removeItem(ACCESS_TOKEN_KEY);
}

export function clearTokens(): void {
  localStorage.removeItem(ACCESS_TOKEN_KEY);
  localStorage.removeItem(REFRESH_TOKEN_KEY);
}

/**
 * Decodes the `exp` claim (seconds since epoch) from a JWT payload WITHOUT
 * verifying its signature. Suitable only for client-side freshness checks —
 * the backend is the source of truth for actual token validity.
 */
export function getTokenExpiry(token: string): number | null {
  const parts = token.split('.');
  if (parts.length !== 3) return null;
  try {
    const payload = JSON.parse(atob(parts[1].replace(/-/g, '+').replace(/_/g, '/')));
    return typeof payload.exp === 'number' ? payload.exp : null;
  } catch {
    return null;
  }
}

/**
 * True if the token is expired or will expire within `leewaySecs`.
 * Treats an undecodable token as expired.
 */
export function isTokenExpired(token: string, leewaySecs = 0): boolean {
  const exp = getTokenExpiry(token);
  if (exp === null) return true;
  const nowSecs = Math.floor(Date.now() / 1000);
  return exp <= nowSecs + leewaySecs;
}
