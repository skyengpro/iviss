import { useState, useEffect, ReactNode } from 'react';
import { AuthContext, AuthContextType } from '@/hooks/auth/use-auth';
import { UserProfile, AuthResponse } from '@/openapi-rq/requests/types.gen';
import { getDeviceId } from '@/services/device/deviceId';
import {
  setAccessToken,
  setRefreshToken,
  clearTokens,
  getAccessToken,
  clearAccessToken,
} from '@/services/auth/tokenManager';
import { client } from '@/openapi-rq/requests/services.gen';
import {
  activateDevice,
  getUserProfile,
  requestDailyLogin,
  verifyDailyLogin,
  loginUser,
  logoutUser,
} from '@/openapi-rq/requests/services.gen';

const SESSION_KEY = 'iviss_session';
const REFRESH_TOKEN_KEY = 'iviss_refresh_token';

function applyAuthTokenToApiClient(token?: string) {
  const baseUrl = import.meta.env.VITE_API_URL || 'http://localhost:3000';
  client.setConfig({
    baseUrl,
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  });
}

function humanizeActivationError(payload: unknown): string | undefined {
  if (!payload || typeof payload !== 'object') return;
  const maybe = payload as { code?: unknown; message?: unknown };
  const code = typeof maybe.code === 'string' ? maybe.code : undefined;
  const message = typeof maybe.message === 'string' ? maybe.message : undefined;

  if (!code && !message) return;

  if (code === 'NOT_FOUND') {
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

let isInterceptorRegistered = false;

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserProfile | null>(null);
  const [session, setSession] = useState<AuthResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // Initialize identity and check for existing session on mount
  useEffect(() => {
    const initIdentity = async () => {
      // Ensure device_id is generated and stored in IndexedDB
      await getDeviceId();

      const existingSession = localStorage.getItem(SESSION_KEY);
      if (existingSession) {
        const sessionData = JSON.parse(existingSession) as AuthResponse;
        setSession(sessionData);
        setUser(sessionData.user);
        applyAuthTokenToApiClient(sessionData.token);

        // Sync token manager with existing session token
        if (sessionData.token && !getAccessToken()) {
          setAccessToken(sessionData.token);
        }
      }
      setIsLoading(false);
    };

    initIdentity();

    if (!isInterceptorRegistered) {
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
      isInterceptorRegistered = true;
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
        token: data.token,
        user: data.user,
      };

      localStorage.setItem(SESSION_KEY, JSON.stringify(newSession));
      // Admin login returns token only (no separate refreshToken in AuthResponse)
      setAccessToken(data.token);
      setRefreshToken(data.token);

      applyAuthTokenToApiClient(data.token);

      setSession(newSession);
      setUser(data.user);

      return { success: true };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : 'Login failed' };
    }
  };

  const logout = async () => {
    try {
      await logoutUser({ throwOnError: false });
    } catch {
      // best-effort logout — clear client-side state regardless
    }
    clearTokens();
    localStorage.removeItem(SESSION_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
    setSession(null);
    setUser(null);
    applyAuthTokenToApiClient(undefined);
    return;
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
