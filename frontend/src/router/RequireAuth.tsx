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
        const deviceActivated = localStorage.getItem('iviss_device_activated') === 'true';
        // A valid refresh token means the session is recoverable — the interceptor will
        // silently refresh the access token on the next API call. Never force daily-login
        // while a valid refresh token exists, as that requires a new OTP.
        const hasValidRefreshToken = !!refreshToken && refreshToken !== 'null';

        if (!hasValidRefreshToken && !accessToken) {
          if (deviceActivated) {
            navigate('/daily-login', { state: { from: location.pathname } });
          } else {
            navigate('/activate', { state: { from: location.pathname } });
          }
        } else if (hasValidRefreshToken && !accessToken) {
          // Refresh token exists but no access token yet — do NOT redirect to daily-login.
          // AuthContext.initIdentity will restore the session; on next API call the interceptor
          // will use the refresh token to silently issue a new access token.
          // No redirect needed here.
        } else if (!hasValidRefreshToken && accessToken) {
          // Access token present but no refresh token and not authenticated — token is invalid.
          if (deviceActivated) {
            navigate('/daily-login', { state: { from: location.pathname } });
          } else {
            navigate('/activate', { state: { from: location.pathname } });
          }
        }
        // If both tokens exist but not authenticated yet — AuthContext is still initializing,
        // the loader above will show. No redirect needed.
      } else if (allowedRoles && user && !allowedRoles.includes(user.role)) {
        // Prevent infinite redirect loop if we are already at the target
        if (user.role === 'admin' || user.role === 'manager' || user.role === 'org_admin') {
          if (location.pathname !== '/backoffice') {
            navigate('/backoffice');
          }
        } else {
          if (location.pathname !== '/mobile') {
            navigate('/mobile');
          }
        }
      }
    }
  }, [isLoading, isAuthenticated, user, allowedRoles, navigate, location]);

  if (isLoading) {
    return (
      <div className="flex h-screen items-center justify-center bg-background">
        <div className="flex flex-col items-center gap-4">
          <div className="relative flex items-center justify-center">
            <div className="h-16 w-16 animate-spin rounded-full border-4 border-muted border-t-primary" />
            <img src="/pwa-64x64.png" alt="IVISS" className="absolute h-8 w-8 rounded-full" />
          </div>
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
