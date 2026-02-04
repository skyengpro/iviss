import React from 'react';
import { useAuth } from '@/hooks/use-auth';
import { RequireAuth } from '@/router/RequireAuth';
import { UserRole } from '@/services/mockAuth';

interface ProtectedRouteProps {
  children: React.ReactNode;
  allowedRoles?: UserRole[];
}

export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ children, allowedRoles }) => {
  return <RequireAuth allowedRoles={allowedRoles}>{children}</RequireAuth>;
};
