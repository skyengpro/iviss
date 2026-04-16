import { useState, useEffect, ReactNode } from 'react';
import { AuthContext, AuthContextType } from '@/hooks/auth/use-auth';
import { UserProfile, AuthResponse } from '@/openapi-rq/requests/types.gen';
import { getDeviceId } from '@/services/device/deviceId';
import {
  setAccessToken,
  setRefreshToken,
  getAccessToken,
  clearAccessToken,
} from '@/services/auth/tokenManager';
import {
  client,
  activateDevice,
  getUserProfile,
  requestDailyLogin,
  verifyDailyLogin,
  loginUser,
  logoutUser,
} from '@/openapi-rq/requests/services.gen';

const SESSION_KEY = 'iviss_session';
const REFRESH_TOKEN_KEY = 'iviss_refresh_token';

// We manage a global logout reference to allow the interceptor to trigger a logout
let globalLogout: (() => Promise<void>) | null = null;

function applyAuthTokenToApiClient(token?: string) {
  const baseUrl = import.meta.env.VITE_API_URL || '';
  // Only set baseUrl — Authorization is handled dynamically by the request interceptor
  // in authInterceptor.ts which reads the latest token from tokenManager on every request.
  // Setting it here as a static header causes stale tokens to be preserved after refresh.
  client.setConfig({ baseUrl });
  // Keep token manager in sync so the interceptor picks up the right token
  if (token) {
    setAccessToken(token);
  }
}

function humanizeActivationError(payload: unknown): string | undefined {
  if (!payload || typeof payload !== 'object') return;
  const maybe = payload as { code?: unknown; message?: unknown };
  const code = typeof maybe.code === 'string' ? maybe.code : undefined;
  const message = typeof maybe.message === 'string' ? maybe.message : undefined;

  if (!code && !message) return;

  if (code === 'NOT_FOUND') {
    if (message?.toLowerCase().includes('device is not registered')) {
      return message;
    }
    return 'Badge number not found. Please check it or contact an administrator.';
  }

  if (code === 'BAD_REQUEST') {
    if (message?.toLowerCase().includes('expired')) {
      return 'Your OTP code has expired. Please request a new one.';
    }
    if (message?.toLowerCase().includes('invalid activation code')) {
      return 'Invalid OTP code. Please try again.';
    }
    return message;
  }

  return message;
}

function hasErrorCode(value: unknown): value is { code: unknown } {
  return typeof value === 'object' && value !== null && 'code' in value;
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserProfile | null>(null);
  const [session, setSession] = useState<AuthResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const logout = async (forced = false) => {
    const accessToken = getAccessToken();

    if (!forced && accessToken) {
      try {
        await logoutUser({
          headers: { Authorization: `Bearer ${accessToken}` },
          throwOnError: false,
        });
      } catch {
        // We still clear local auth state even if the remote logout request fails.
      }
    }

    setSession(null);
    setUser(null);

    // Clear tokens from API client
    applyAuthTokenToApiClient(undefined);

    // Clear ALL auth tokens from local storage
    clearAccessToken();
    localStorage.removeItem(SESSION_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);

    // Preserve IndexedDB state here:
    // - device_id must remain stable so a terminated browser can request a new daily login
    // - key material must remain stable so refresh flows keep working after re-login
    // Session termination only revokes auth state, not device identity.

    if (forced) {
      // Set a flag so the login page can show the toast after the full-page redirect
      localStorage.setItem('iviss_forced_logout_reason', 'TERMINATED');
      // Force redirect to the daily login flow.
      window.location.href = '/daily-login';
    }
  };

  // Assign the global logout function to the instance logout so the interceptor can call it
  useEffect(() => {
    globalLogout = () => logout(true);
  }, []);

  // Set up the API response interceptor
  useEffect(() => {
    const responseInterceptor = async (response: Response) => {
      if (!response.ok) {
        // Only fire forced logout if the user is currently authenticated.
        // Avoids kicking the user during a failed login attempt.
        const hasSession = !!localStorage.getItem(SESSION_KEY);
        if (!hasSession) return response;

        // Only logout on explicit SESSION_REVOKED — plain 401s are handled
        // by authInterceptor which refreshes the token transparently.
        let isSessionRevoked = false;
        try {
          const clonedResponse = response.clone();
          const body = await clonedResponse.json();
          if (body && typeof body === 'object' && 'code' in body) {
            if (body.code === 'SESSION_REVOKED') {
              isSessionRevoked = true;
            }
          }
        } catch {
          // Not JSON or unreadable — ignore
        }

        if (isSessionRevoked && globalLogout) {
          await globalLogout();
        }
      }
      return response;
    };

    client.interceptors.response.use(responseInterceptor);

    const handleSessionRevoked = async () => {
      if (globalLogout) {
        await globalLogout();
      }
    };

    window.addEventListener('iviss:session-revoked', handleSessionRevoked);

    return () => {
      client.interceptors.response.eject(responseInterceptor);
      window.removeEventListener('iviss:session-revoked', handleSessionRevoked);
    };
  }, []);

  // Initialize identity and check for existing session on mount
  useEffect(() => {
    const initIdentity = async () => {
      // Ensure device_id is generated and stored in IndexedDB
      await getDeviceId();

      const existingSession = localStorage.getItem(SESSION_KEY);
      if (existingSession) {
        try {
          const sessionData = JSON.parse(existingSession) as AuthResponse;
          // Decode the JWT payload (middle segment) to check expiry WITHOUT
          // signature verification. This lets us reject stale tokens from old
          // key pairs or expired sessions before a 401 can trigger a forced logout.
          const parts = sessionData.accessToken?.split('.');
          let isValid = false;
          if (parts && parts.length === 3) {
            try {
              const payload = JSON.parse(atob(parts[1].replace(/-/g, '+').replace(/_/g, '/')));
              const nowSecs = Math.floor(Date.now() / 1000);
              isValid = typeof payload.exp === 'number' && payload.exp > nowSecs;
            } catch {
              // unparseable payload
            }
          }

          if (isValid) {
            setSession(sessionData);
            setUser(sessionData.user);
            applyAuthTokenToApiClient(sessionData.accessToken);

            // Sync token manager with existing session token
            if (sessionData.accessToken && !getAccessToken()) {
              setAccessToken(sessionData.accessToken);
            }
            // Always restore refresh token so the interceptor can refresh expired access tokens
            if (sessionData.refreshToken) {
              setRefreshToken(sessionData.refreshToken);
            }
          } else {
            // Access token is expired but we may still have a valid refresh token.
            // Restore the session state so the interceptor can attempt a refresh
            // on the next API call rather than clearing everything immediately.
            if (sessionData.refreshToken) {
              setRefreshToken(sessionData.refreshToken);
              // Keep the stale access token in the store so the interceptor
              // knows we were authenticated and should attempt a refresh.
              if (sessionData.accessToken) {
                setAccessToken(sessionData.accessToken);
              }
              setSession(sessionData);
              setUser(sessionData.user);
              applyAuthTokenToApiClient(sessionData.accessToken);
            } else {
              // No refresh token either — truly expired, clear everything
              localStorage.removeItem(SESSION_KEY);
              localStorage.removeItem(REFRESH_TOKEN_KEY);
            }
          }
        } catch {
          localStorage.removeItem(SESSION_KEY);
          localStorage.removeItem(REFRESH_TOKEN_KEY);
        }
      }
      setIsLoading(false);
    };

    initIdentity();

    if (
      !(window as { __iviss_shift_interceptor_registered?: boolean })
        .__iviss_shift_interceptor_registered
    ) {
      (
        window as { __iviss_shift_interceptor_registered?: boolean }
      ).__iviss_shift_interceptor_registered = true;
      const interceptor = async (response: Response) => {
        if (!response.ok && response.status === 401) {
          try {
            const resClone = response.clone();
            const json = await resClone.json();
            if (
              json?.message === 'Shift ended' ||
              json?.reason === 'Shift ended' ||
              json?.message?.includes('Shift ended')
            ) {
              setSession(null);
              setUser(null);
              applyAuthTokenToApiClient();
              localStorage.removeItem(SESSION_KEY);
              clearAccessToken();
              globalThis.location.href = '/daily-login';
            }
          } catch {
            // ignore parse error
          }
        }
        return response;
      };
      client.interceptors.response.use(interceptor);
    }
  }, []);

  const activate: AuthContextType['activate'] = async ({
    badgeId,
    activationCode,
    deviceId,
    publicKeyBase64,
  }) => {
    try {
      const res = await activateDevice({
        body: {
          badgeId,
          activationCode,
          deviceId,
          publicKeyBase64,
        },
        throwOnError: false,
      });

      if (res.error) {
        const friendly = humanizeActivationError(res.error);
        return { success: false, error: friendly || 'Activation failed' };
      }

      const data = res.data;
      if (!data) {
        return { success: false, error: 'Activation failed' };
      }

      let resolvedUser: UserProfile = data.user;
      try {
        const meRes = await getUserProfile({
          headers: { Authorization: `Bearer ${data.accessToken}` },
          throwOnError: false,
        });
        if (meRes.data) resolvedUser = meRes.data;
      } catch {
        // Ignore profile refresh errors
      }

      const newSession = {
        token: data.accessToken,
        user: resolvedUser,
      } as unknown as AuthResponse;

      localStorage.setItem(SESSION_KEY, JSON.stringify(newSession));
      localStorage.setItem(REFRESH_TOKEN_KEY, data.refreshToken);
      localStorage.setItem('iviss_device_activated', 'true');

      // Sync with token manager
      setAccessToken(data.accessToken);
      setRefreshToken(data.refreshToken);

      applyAuthTokenToApiClient(data.accessToken);

      setSession(newSession);
      setUser(resolvedUser);

      return { success: true };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : 'Activation failed' };
    }
  };

  const dailyLoginRequest: AuthContextType['dailyLoginRequest'] = async ({ badgeId }) => {
    try {
      const deviceId = await getDeviceId();

      const res = await requestDailyLogin({
        body: { badgeId, deviceId },
        throwOnError: false,
      });

      if (res.error) {
        // Handle case where device is deleted from backend but flag exists on frontend
        if (hasErrorCode(res.error) && res.error.code === 'NOT_FOUND') {
          localStorage.removeItem('iviss_device_activated');
        }
        const friendly = humanizeActivationError(res.error);
        return { success: false, error: friendly || 'Failed to request OTP' };
      }
      return { success: true };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : 'Request failed' };
    }
  };

  const dailyLoginVerify: AuthContextType['dailyLoginVerify'] = async ({
    badgeId,
    activationCode,
    deviceId,
  }) => {
    try {
      const res = await verifyDailyLogin({
        body: { badgeId, activationCode, deviceId },
        throwOnError: false,
      });

      if (res.error) {
        // Handle case where device is deleted from backend but flag exists on frontend
        if (hasErrorCode(res.error) && res.error.code === 'NOT_FOUND') {
          localStorage.removeItem('iviss_device_activated');
        }
        const friendly = humanizeActivationError(res.error);
        return { success: false, error: friendly || 'Verification failed' };
      }

      const data = res.data;
      if (!data) {
        return { success: false, error: 'Verification failed' };
      }

      // Daily login returns tokens only; user profile is fetched separately.
      // Keep behavior consistent with activation: best-effort refresh profile.
      let resolvedUser: UserProfile | null = null;
      try {
        const meRes = await getUserProfile({
          headers: { Authorization: `Bearer ${data.accessToken}` },
          throwOnError: false,
        });
        if (meRes.data) resolvedUser = meRes.data;
      } catch {
        // Ignore profile refresh errors
      }

      const newSession = {
        token: data.accessToken,
        user: resolvedUser,
      } as unknown as AuthResponse;

      localStorage.setItem(SESSION_KEY, JSON.stringify(newSession));
      localStorage.setItem(REFRESH_TOKEN_KEY, data.refreshToken);
      localStorage.setItem('iviss_device_activated', 'true');

      setAccessToken(data.accessToken);
      setRefreshToken(data.refreshToken);

      applyAuthTokenToApiClient(data.accessToken);

      setSession(newSession);
      setUser(resolvedUser);

      return { success: true };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : 'Verification failed' };
    }
  };

  const login = async (email: string, password: string) => {
    try {
      const res = await loginUser({
        body: { email, password },
        throwOnError: false,
      });

      if (res.error) {
        const errPayload = res.error as { message?: string };
        return { success: false, error: errPayload?.message || 'Login failed' };
      }

      const data = res.data;
      if (!data) {
        return { success: false, error: 'Login failed' };
      }

      const newSession = {
        accessToken: data.accessToken,
        refreshToken: data.refreshToken,
        user: data.user,
      };

      localStorage.setItem(SESSION_KEY, JSON.stringify(newSession));
      setAccessToken(data.accessToken);
      setRefreshToken(data.refreshToken);

      applyAuthTokenToApiClient(data.accessToken);

      setSession(newSession as unknown as AuthResponse);
      setUser(data.user);

      // Return mustChangePassword flag from backend response
      const mustChangePassword =
        (data as unknown as { mustChangePassword?: boolean }).mustChangePassword ?? false;
      return { success: true, mustChangePassword };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : 'Login failed' };
    }
  };

  const value: AuthContextType = {
    user,
    session,
    isLoading,
    isAuthenticated: !!session,
    login,
    activate,
    dailyLoginRequest,
    dailyLoginVerify,
    logout,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
