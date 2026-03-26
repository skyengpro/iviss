import React from 'react';
import { RequireAuth } from '@/router/RequireAuth';

interface ProtectedRouteProps {
  children: React.ReactNode;
  allowedRoles?: string[];
}

export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ children, allowedRoles }) => {
  return <RequireAuth allowedRoles={allowedRoles}>{children}</RequireAuth>;
};
