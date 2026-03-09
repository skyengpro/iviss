import { useState, useEffect, ReactNode } from 'react';
import { mockAuthService } from '@/services/mockAuth';
import { AuthContext, AuthContextType } from '@/hooks/auth/use-auth';
import { UserProfile, AuthResponse } from '@/openapi-rq/requests/types.gen';
import { getDeviceId } from '@/services/deviceId';
import { client } from '@/openapi-rq/requests/services.gen';

const SESSION_KEY = 'iviss_session';
const REFRESH_TOKEN_KEY = 'iviss_refresh_token';

function applyAuthTokenToApiClient(token?: string) {
  client.setConfig({
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
              localStorage.removeItem(REFRESH_TOKEN_KEY);
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
    const baseUrl = import.meta.env.VITE_API_URL || 'http://localhost:3000';

    try {
      const res = await fetch(`${baseUrl}/auth/activate`, {
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
        const meRes = await fetch(`${baseUrl}/users/me`, {
          headers: {
            authorization: `Bearer ${data.accessToken}`,
          },
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

      applyAuthTokenToApiClient(data.accessToken);

      setSession(newSession);
      setUser(resolvedUser);

      return { success: true };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : 'Activation failed' };
    }
  };

  const dailyLoginRequest: AuthContextType['dailyLoginRequest'] = async ({ badgeId }) => {
    const baseUrl = import.meta.env.VITE_API_URL || 'http://localhost:3000';
    try {
      const res = await fetch(`${baseUrl}/auth/daily-login/request`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ badgeId }),
      });

      if (!res.ok) {
        const contentType = res.headers.get('content-type') || '';
        if (contentType.includes('application/json')) {
          const json = (await res.json()) as unknown;
          const friendly = humanizeActivationError(json);
          return { success: false, error: friendly || 'Failed to request OTP' };
        }
        const text = await res.text();
        return { success: false, error: text || 'Failed to request OTP' };
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
    const baseUrl = import.meta.env.VITE_API_URL || 'http://localhost:3000';
    try {
      const res = await fetch(`${baseUrl}/auth/daily-login/verify`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ badgeId, activationCode, deviceId }),
      });

      if (!res.ok) {
        const contentType = res.headers.get('content-type') || '';
        if (contentType.includes('application/json')) {
          const json = (await res.json()) as unknown;
          const friendly = humanizeActivationError(json);
          return { success: false, error: friendly || 'Verification failed' };
        }
        const text = await res.text();
        return { success: false, error: text || 'Verification failed' };
      }

      const data = (await res.json()) as {
        accessToken: string;
        refreshToken: string;
        user: UserProfile;
      };

      let resolvedUser: UserProfile = data.user;
      try {
        const meRes = await fetch(`${baseUrl}/users/me`, {
          headers: { authorization: `Bearer ${data.accessToken}` },
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

      applyAuthTokenToApiClient(data.accessToken);

      setSession(newSession);
      setUser(resolvedUser);

      return { success: true };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : 'Verification failed' };
    }
  };

  const login = async (username: string, password: string) => {
    const result = await mockAuthService.login(username, password);

    if (result.success && result.session) {
      const backendSession = result.session as unknown as AuthResponse;
      setSession(backendSession);
      setUser(backendSession.user);
      return { success: true };
    }

    return { success: false, error: result.error };
  };

  const logout = async () => {
    setSession(null);
    setUser(null);
    applyAuthTokenToApiClient();
    localStorage.removeItem(SESSION_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
  };

  const getMockCredentials = () => mockAuthService.getMockCredentials();

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
    getMockCredentials,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
