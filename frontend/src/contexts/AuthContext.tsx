import { useState, useEffect, ReactNode } from 'react';
import { mockAuthService } from '@/services/mock/mockAuth';
import { AuthContext, AuthContextType } from '@/hooks/auth/use-auth';
import { UserProfile, AuthResponse } from '@/openapi-rq/requests/types.gen';
import { getDeviceId } from '@/services/device/deviceId';
import {
  setAccessToken,
  setRefreshToken,
  clearTokens,
  getAccessToken,
} from '@/services/auth/tokenManager';
import { client } from '@/openapi-rq/requests/services.gen';
import { fetchWithAuth } from '@/services/api/backendFetch';

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
  }, []);

  const activate: AuthContextType['activate'] = async ({
    badgeId,
    activationCode,
    deviceId,
    publicKeyBase64,
  }) => {
    try {
      const res = await fetchWithAuth('/auth/activate', {
        method: 'POST',
        headers: {
          'content-type': 'application/json',
        },
        body: JSON.stringify({
          badgeId,
          activationCode,
          deviceId,
          publicKeyBase64,
        }),
      });

      if (!res.ok) {
        const contentType = res.headers.get('content-type') || '';

        if (contentType.includes('application/json')) {
          const json = (await res.json()) as unknown;
          const friendly = humanizeActivationError(json);
          return { success: false, error: friendly || 'Activation failed' };
        }

        const text = await res.text();
        return { success: false, error: text || 'Activation failed' };
      }

      const data = (await res.json()) as {
        accessToken: string;
        refreshToken: string;
        user: UserProfile;
      };

      let resolvedUser: UserProfile = data.user;
      try {
        const meRes = await fetchWithAuth('/users/me', {
          headers: { Authorization: `Bearer ${data.accessToken}` },
        });
        if (meRes.ok) {
          resolvedUser = (await meRes.json()) as UserProfile;
        }
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

  const login = async (username: string, password: string) => {
    const result = await mockAuthService.login(username, password);

    if (result.success && result.session) {
      const backendSession = {
        token: result.session.token,
        user: result.session.user as unknown as UserProfile,
      } as unknown as AuthResponse;

      localStorage.setItem(SESSION_KEY, JSON.stringify(backendSession));
      applyAuthTokenToApiClient(backendSession.token);

      setSession(backendSession);
      setUser(backendSession.user);

      // Persist tokens for the auth interceptor and token manager
      if (backendSession.token) {
        setAccessToken(backendSession.token);
        // In a real flow, the backend would also return a refresh_token
        // For now with mock auth, we store the same token as refresh
        setRefreshToken(backendSession.token);

        // Ensure session persistence matching activation flow
        localStorage.setItem(SESSION_KEY, JSON.stringify(backendSession));
      }

      return { success: true };
    }

    return { success: false, error: result.error };
  };

  const logout = async () => {
    await mockAuthService.logout();
    clearTokens();
    localStorage.removeItem(SESSION_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
    setSession(null);
    setUser(null);
    applyAuthTokenToApiClient(undefined);
    localStorage.removeItem(SESSION_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
    return;
  };

  const getMockCredentials = () => mockAuthService.getMockCredentials();

  const value: AuthContextType = {
    user,
    session,
    isLoading,
    isAuthenticated: !!session,
    login,
    activate,
    logout,
    getMockCredentials,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
