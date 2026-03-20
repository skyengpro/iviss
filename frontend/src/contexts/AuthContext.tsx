import { useState, useEffect, ReactNode } from 'react';
import { loginUser } from '@/openapi-rq/requests/services.gen';
import { mockAuthService } from '@/services/mock/mockAuth';
import { AuthContext, AuthContextType } from '@/hooks/auth/use-auth';
import { UserProfile, AuthResponse } from '@/openapi-rq/requests/types.gen';
import { getDeviceId } from '@/services/device/deviceId';
import {
  setAccessToken,
  setRefreshToken,
  getAccessToken,
  clearAccessToken,
} from '@/services/auth/tokenManager';
import { client } from '@/openapi-rq/requests/services.gen';
import {
  activateDevice,
  getUserProfile,
  requestDailyLogin,
  verifyDailyLogin,
} from '@/openapi-rq/requests/services.gen';
import { toast } from 'sonner';

const SESSION_KEY = 'iviss_session';
const REFRESH_TOKEN_KEY = 'iviss_refresh_token';

// We manage a global logout reference to allow the interceptor to trigger a logout
let globalLogout: (() => Promise<void>) | null = null;

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

let isInterceptorRegistered = false;

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserProfile | null>(null);
  const [session, setSession] = useState<AuthResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const logout = async (forced = false) => {
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

        let isSessionRevoked = false;

        if (response.status === 401) {
          isSessionRevoked = true;
        } else {
          try {
            // Clone the response so we don't consume the body in case it's needed elsewhere
            const clonedResponse = response.clone();
            const body = await clonedResponse.json();
            if (body && typeof body === 'object' && 'code' in body) {
              if (body.code === 'SESSION_REVOKED') {
                isSessionRevoked = true;
              }
            }
          } catch (e) {
            // Not a JSON response or unable to parse, ignore
          }
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
          const parts = sessionData.token?.split('.');
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
            applyAuthTokenToApiClient(sessionData.token);

            // Sync token manager with existing session token
            if (sessionData.token && !getAccessToken()) {
              setAccessToken(sessionData.token);
            }
          } else {
            // Silently clear the stale/expired session
            localStorage.removeItem(SESSION_KEY);
            localStorage.removeItem(REFRESH_TOKEN_KEY);
          }
        } catch {
          localStorage.removeItem(SESSION_KEY);
          localStorage.removeItem(REFRESH_TOKEN_KEY);
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
        const err = res.error as any;
        if (err && typeof err === 'object' && err.code === 'NOT_FOUND') {
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
        const err = res.error as any;
        if (err && typeof err === 'object' && err.code === 'NOT_FOUND') {
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

  const login = async (username: string, password: string) => {
    try {
      const result = await loginUser({
        body: {
          email: username, // Backend LoginRequest uses email field for identification
          password,
        },
        throwOnError: true,
      });

      if (result.data) {
        const backendSession = {
          token: result.data.token,
          user: result.data.user as unknown as UserProfile,
        } as unknown as AuthResponse;

        localStorage.setItem(SESSION_KEY, JSON.stringify(backendSession));
        // Note: Backend login might not return refresh token yet
        const responseData = result.data as Record<string, unknown>;
        if (responseData.refreshToken && typeof responseData.refreshToken === 'string') {
          localStorage.setItem(REFRESH_TOKEN_KEY, responseData.refreshToken);
        }
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
      return { success: false, error: 'Login failed' };
    } catch (err) {
      return {
        success: false,
        error: err instanceof Error ? err.message : 'Invalid credentials',
      };
    }
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
