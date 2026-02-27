import { useState, useEffect, ReactNode } from 'react';
import { mockAuthService } from '@/services/mockAuth';
import { AuthContext, AuthContextType } from '@/hooks/auth/use-auth';
import { UserProfile, AuthResponse } from '@/openapi-rq/requests/types.gen';
import { getDeviceId } from '@/services/deviceId';

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
      return { success: true };
    }

    return { success: false, error: result.error };
  };

  const logout = async () => {
    await mockAuthService.logout();
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
