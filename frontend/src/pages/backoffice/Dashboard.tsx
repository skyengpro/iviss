import { lazy, Suspense } from 'react';
import { useAuth } from '@/hooks/auth/use-auth';

const BackOfficeDashboard = lazy(() => import('./BackOfficeDashboard'));
const OrgAdminDashboard = lazy(() => import('./OrgAdminDashboard'));

export default function Dashboard() {
  const { user } = useAuth();

  // Choose the dashboard based on role
  const isOrgAdmin = user?.role === 'org_admin';

  return (
    <Suspense
      fallback={
        <div className="flex h-[calc(100vh-4rem)] items-center justify-center">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
        </div>
      }
    >
      {isOrgAdmin ? <OrgAdminDashboard /> : <BackOfficeDashboard />}
    </Suspense>
  );
}
