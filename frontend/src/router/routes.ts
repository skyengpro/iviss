import { lazy } from 'react';

// Lazy Pages
const Activate = lazy(() => import('../pages/auth/Activate'));
const DailyLogin = lazy(() => import('../pages/auth/DailyLogin'));
const AdminLogin = lazy(() => import('../pages/auth/AdminLogin'));
const NotFound = lazy(() => import('../pages/NotFound'));

// Mobile Pages
const MobileDashboard = lazy(() => import('../pages/mobile/MobileDashboard'));
const MobileSearch = lazy(() => import('../pages/mobile/MobileSearch'));
const MobileScan = lazy(() => import('../pages/mobile/MobileScan'));
const MobileHistory = lazy(() => import('../pages/mobile/MobileHistory'));
const MobileProfile = lazy(() => import('../pages/mobile/MobileProfile'));
const MobileVehicleResult = lazy(() => import('../pages/mobile/MobileVehicleResult'));
const MobileHistoryDetail = lazy(() => import('../pages/mobile/MobileHistoryDetail'));
const MobileCarteGrise = lazy(() => import('../pages/mobile/MobileCarteGrise'));
const MobileSupport = lazy(() => import('../pages/mobile/MobileSupport'));
const MobileSettings = lazy(() => import('../pages/mobile/MobileSettings'));

// Back Office Pages
const BackOfficeDashboard = lazy(() => import('../pages/backoffice/BackOfficeDashboard'));
const ControlHistory = lazy(() => import('../pages/backoffice/ControlHistory'));
const ControlDetail = lazy(() => import('../pages/backoffice/ControlDetail'));
const UserManagement = lazy(() => import('../pages/backoffice/UserManagement'));
const PendingVehicles = lazy(() => import('../pages/backoffice/PendingVehicles'));
const BackOfficeReports = lazy(() => import('../pages/backoffice/BackOfficeReports'));
const OrganizationManagement = lazy(() => import('../pages/backoffice/OrganizationManagement'));

export interface AppRoute {
  path: string;
  component: React.ComponentType | null;
  allowedRoles?: string[];
  redirectTo?: string;
  replace?: boolean;
}

export const publicRoutes: AppRoute[] = [
  { path: '/activate', component: Activate },
  { path: '/login', component: Activate },
  { path: '/daily-login', component: DailyLogin },
  { path: '/admin-login', component: AdminLogin },
  { path: '/', component: null, redirectTo: '/activate', replace: true },
];

export const mobileRoutes: AppRoute[] = [
  { path: '/mobile', component: MobileDashboard, allowedRoles: ['agent', 'manager', 'admin'] },
  { path: '/mobile/search', component: MobileSearch, allowedRoles: ['agent', 'manager', 'admin'] },
  { path: '/mobile/scan', component: MobileScan, allowedRoles: ['agent', 'manager', 'admin'] },
  {
    path: '/mobile/history',
    component: MobileHistory,
    allowedRoles: ['agent', 'manager', 'admin'],
  },
  {
    path: '/mobile/profile',
    component: MobileProfile,
    allowedRoles: ['agent', 'manager', 'admin'],
  },
  {
    path: '/mobile/vehicle/:plateNumber',
    component: MobileVehicleResult,
    allowedRoles: ['agent', 'manager', 'admin'],
  },
  {
    path: '/mobile/history/:id',
    component: MobileHistoryDetail,
    allowedRoles: ['agent', 'manager', 'admin'],
  },
  {
    path: '/mobile/carte-grise',
    component: MobileCarteGrise,
    allowedRoles: ['agent', 'manager', 'admin'],
  },
  {
    path: '/mobile/support',
    component: MobileSupport,
    allowedRoles: ['agent', 'manager', 'admin'],
  },
  {
    path: '/mobile/settings',
    component: MobileSettings,
    allowedRoles: ['agent', 'manager', 'admin'],
  },
];

export const backOfficeRoutes: AppRoute[] = [
  {
    path: '/backoffice',
    component: BackOfficeDashboard,
    allowedRoles: ['admin', 'manager', 'admin'],
  },
  {
    path: '/backoffice/controls',
    component: ControlHistory,
    allowedRoles: ['admin', 'manager', 'admin'],
  },
  {
    path: '/backoffice/controls/:controlId',
    component: ControlDetail,
    allowedRoles: ['admin', 'manager', 'admin'],
  },
  { path: '/backoffice/users', component: UserManagement, allowedRoles: ['admin'] },
  { path: '/backoffice/validation', component: PendingVehicles, allowedRoles: ['admin'] },
  {
    path: '/backoffice/reports',
    component: BackOfficeReports,
    allowedRoles: ['admin', 'manager', 'admin'],
  },
  { path: '/backoffice/vehicles', component: BackOfficeDashboard, allowedRoles: ['admin'] },
  {
    path: '/backoffice/organizations',
    component: OrganizationManagement,
    allowedRoles: ['admin'],
  },
  { path: '/backoffice/audit', component: BackOfficeDashboard, allowedRoles: ['admin'] },
  { path: '/backoffice/settings', component: BackOfficeDashboard, allowedRoles: ['admin'] },
];

export const catchAllRoute = { path: '*', component: NotFound };
