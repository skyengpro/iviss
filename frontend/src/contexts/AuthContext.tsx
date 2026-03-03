import { useState, useEffect, ReactNode } from 'react';
import { mockAuthService } from '@/services/mockAuth';
import { AuthContext, AuthContextType } from '@/hooks/auth/use-auth';
import { UserProfile, AuthResponse } from '@/openapi-rq/requests/types.gen';
import { getDeviceId } from '@/services/deviceId';
import {
  setAccessToken,
  setRefreshToken,
  clearTokens,
  getAccessToken,
} from '@/services/tokenManager';

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserProfile | null>(null);
  const [session, setSession] = useState<AuthResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // Initialize identity and check for existing session on mount
  useEffect(() => {
    const initIdentity = async () => {
      // Ensure device_id is generated and stored in IndexedDB
      await getDeviceId();

      const existingSession = mockAuthService.getSession() as unknown as AuthResponse | null;
      if (existingSession) {
        setSession(existingSession);
        setUser(existingSession.user);

        // Sync token manager with existing session token
        if (existingSession.token && !getAccessToken()) {
          setAccessToken(existingSession.token);
        }
      }
      setIsLoading(false);
    };

    initIdentity();
  }, []);

  const login = async (username: string, password: string) => {
    const result = await mockAuthService.login(username, password);

    if (result.success && result.session) {
      const backendSession = result.session as unknown as AuthResponse;
      setSession(backendSession);
      setUser(backendSession.user);

      // Persist tokens for the auth interceptor
      if (backendSession.token) {
        setAccessToken(backendSession.token);
        // In a real flow, the backend would also return a refresh_token
        // For now with mock auth, we store the same token as refresh
        setRefreshToken(backendSession.token);
      }

      return { success: true };
    }

    return { success: false, error: result.error };
  };

  const logout = async () => {
    await mockAuthService.logout();
    clearTokens();
    setSession(null);
    setUser(null);
  };

  const getMockCredentials = () => mockAuthService.getMockCredentials();

  const value: AuthContextType = {
    user,
    session,
    isLoading,
    isAuthenticated: !!session,
    login,
    logout,
    getMockCredentials,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
