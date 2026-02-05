import React from 'react';
import { RequireAuth } from '@/router/RequireAuth';
import { UserRole } from '@/services/mockAuth';

interface ProtectedRouteProps {
  readonly children: React.ReactNode;
  readonly allowedRoles?: UserRole[];
}

export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ children, allowedRoles }) => {
  return <RequireAuth allowedRoles={allowedRoles}>{children}</RequireAuth>;
};
