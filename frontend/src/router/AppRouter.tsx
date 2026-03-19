import { Suspense } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { Loader2 } from 'lucide-react';
import { publicRoutes, mobileRoutes, backOfficeRoutes, catchAllRoute } from './routes';
import { ProtectedRoute } from './ProtectedRoute';
import { getAccessToken, getRefreshToken } from '@/services/auth/tokenManager';

const PageLoader = () => (
  <div className="flex h-screen w-full items-center justify-center bg-background">
    <Loader2 className="h-8 w-8 animate-spin text-primary" />
  </div>
);

export const AppRouter = () => {
  const allProtectedRoutes = [...mobileRoutes, ...backOfficeRoutes];
  const accessToken = getAccessToken();
  const refreshToken = getRefreshToken();
  const entryRedirect =
    !refreshToken && !accessToken ? '/activate' : accessToken ? '/backoffice' : '/daily-login';

  return (
    <Suspense fallback={<PageLoader />}>
      <Routes>
        <Route path="/" element={<Navigate to={entryRedirect} replace />} />
        {/* Public Routes */}
        {publicRoutes.map((route) => (
          <Route
            key={route.path}
            path={route.path}
            element={
              route.redirectTo ? (
                <Navigate to={route.redirectTo} replace={route.replace} />
              ) : route.component ? (
                <route.component />
              ) : null
            }
          />
        ))}

        {/* Protected Routes */}
        {allProtectedRoutes.map((route) => (
          <Route
            key={route.path}
            path={route.path}
            element={
              <ProtectedRoute allowedRoles={route.allowedRoles}>
                {route.redirectTo ? (
                  <Navigate to={route.redirectTo} replace={route.replace} />
                ) : (
                  <route.component />
                )}
              </ProtectedRoute>
            }
          />
        ))}

        {/* Catch-all */}
        <Route path={catchAllRoute.path} element={<catchAllRoute.component />} />
      </Routes>
    </Suspense>
  );
};
