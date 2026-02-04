import { useState, useEffect, ReactNode } from 'react';
import { mockAuthService, User, AuthSession } from '@/services/mockAuth';
import { AuthContext, AuthContextType } from '@/hooks/use-auth';

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [session, setSession] = useState<AuthSession | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // Check for existing session on mount
  useEffect(() => {
    const existingSession = mockAuthService.getSession();
    if (existingSession) {
      setSession(existingSession);
      setUser(existingSession.user);
    }
    setIsLoading(false);
  }, []);

  const login = async (username: string, password: string) => {
    const result = await mockAuthService.login(username, password);

    if (result.success && result.session) {
      setSession(result.session);
      setUser(result.session.user);
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


