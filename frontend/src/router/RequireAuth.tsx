import { ReactNode, useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { useAuth } from '@/hooks/auth/use-auth';
import { getAccessToken, getRefreshToken } from '@/services/auth/tokenManager';

export function RequireAuth({
  children,
  allowedRoles,
}: {
  children: ReactNode;
  allowedRoles?: string[];
}) {
  const { user, isLoading, isAuthenticated } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    if (!isLoading) {
      if (!isAuthenticated) {
        const accessToken = getAccessToken();
        const refreshToken = getRefreshToken();
        if (!refreshToken && !accessToken) {
          navigate('/activate', { state: { from: location.pathname } });
        } else if (refreshToken && !accessToken) {
          navigate('/daily-login', { state: { from: location.pathname } });
        } else {
          navigate('/activate', { state: { from: location.pathname } });
        }
      } else if (allowedRoles && user && !allowedRoles.includes(user.role as unknown as string)) {
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
