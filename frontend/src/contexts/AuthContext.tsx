import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { mockAuthService, User, AuthSession, UserRole } from '@/services/mockAuth';

interface AuthContextType {
  user: User | null;
  session: AuthSession | null;
  isLoading: boolean;
  isAuthenticated: boolean;
  login: (username: string, password: string) => Promise<{ success: boolean; error?: string }>;
  logout: () => Promise<void>;
  getMockCredentials: () => { role: string; username: string; password: string }[];
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

// Routes that don't require authentication
const publicRoutes = ['/', '/login'];

// Route access by role
const roleAccess: Record<UserRole, string[]> = {
  agent: ['/mobile', '/mobile/search', '/mobile/scan', '/mobile/history', '/mobile/settings', '/mobile/profile'],
  supervisor: ['/mobile', '/mobile/search', '/mobile/scan', '/mobile/history', '/mobile/settings', '/mobile/profile', '/backoffice', '/backoffice/controls'],
  admin: ['/backoffice', '/backoffice/controls', '/backoffice/users', '/backoffice/vehicles', '/backoffice/validation', '/backoffice/organizations', '/backoffice/audit', '/backoffice/settings'],
};

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

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}

// Protected route wrapper component
export function RequireAuth({
  children,
  allowedRoles
}: {
  children: ReactNode;
  allowedRoles?: UserRole[];
}) {
  const { user, isLoading, isAuthenticated } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    if (!isLoading) {
      if (!isAuthenticated) {
        // Redirect to login with return path
        navigate('/login', { state: { from: location.pathname } });
      } else if (allowedRoles && user && !allowedRoles.includes(user.role)) {
        // User doesn't have required role
        if (user.role === 'admin') {
          navigate('/backoffice');
        } else {
          navigate('/mobile');
        }
      }
    }
  }, [isLoading, isAuthenticated, user, allowedRoles, navigate, location]);

  if (isLoading) {
    return (
      <div className="flex h-screen items-center justify-center bg-background">
        <div className="flex flex-col items-center gap-4">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
          <p className="text-sm text-muted-foreground">Loading...</p>
        </div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return null;
  }

  if (allowedRoles && user && !allowedRoles.includes(user.role)) {
    return null;
  }

  return <>{children}</>;
}
