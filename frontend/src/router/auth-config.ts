import { UserRole } from '@/services/mockAuth';

// Routes that don't require authentication
export const publicRoutes = ['/', '/login'];

// Route access by role
export const roleAccess: Record<UserRole, string[]> = {
  agent: [
    '/mobile',
    '/mobile/search',
    '/mobile/scan',
    '/mobile/history',
    '/mobile/settings',
    '/mobile/profile',
  ],
  supervisor: [
    '/mobile',
    '/mobile/search',
    '/mobile/scan',
    '/mobile/history',
    '/mobile/settings',
    '/mobile/profile',
    '/backoffice',
    '/backoffice/controls',
  ],
  admin: [
    '/backoffice',
    '/backoffice/controls',
    '/backoffice/users',
    '/backoffice/vehicles',
    '/backoffice/validation',
    '/backoffice/organizations',
    '/backoffice/audit',
    '/backoffice/settings',
  ],
};
