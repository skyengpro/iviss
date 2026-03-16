import React from 'react';
import { RequireAuth } from '@/router/RequireAuth';
import { UserRole } from '@/services/mock/mockAuth';

interface ProtectedRouteProps {
  children: React.ReactNode;
  allowedRoles?: UserRole[];
}

export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ children, allowedRoles }) => {
  return <RequireAuth allowedRoles={allowedRoles}>{children}</RequireAuth>;
};
