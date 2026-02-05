import { ReactNode, useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { UserRole } from '@/services/mockAuth';
import { useAuth } from '@/hooks/use-auth';

export function RequireAuth({
  children,
  allowedRoles,
}: {
  readonly children: ReactNode;
  readonly allowedRoles?: UserRole[];
}) {
  const { user, isLoading, isAuthenticated } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    if (!isLoading) {
      if (!isAuthenticated) {
        navigate('/login', { state: { from: location.pathname } });
      } else if (allowedRoles && user && !allowedRoles.includes(user.role)) {
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
