import { lazy } from "react";
import { UserRole } from "@/services/mockAuth";

// Lazy Pages
const Login = lazy(() => import("../pages/auth/Login"));
const NotFound = lazy(() => import("../pages/NotFound"));

// Mobile Pages
const MobileDashboard = lazy(() => import("../pages/mobile/MobileDashboard"));
const MobileSearch = lazy(() => import("../pages/mobile/MobileSearch"));
const MobileScan = lazy(() => import("../pages/mobile/MobileScan"));
const MobileHistory = lazy(() => import("../pages/mobile/MobileHistory"));
const MobileProfile = lazy(() => import("../pages/mobile/MobileProfile"));
const MobileVehicleResult = lazy(() => import("../pages/mobile/MobileVehicleResult"));
const MobileCarteGrise = lazy(() => import("../pages/mobile/MobileCarteGrise"));

// Back Office Pages
const BackOfficeDashboard = lazy(() => import("../pages/backoffice/BackOfficeDashboard"));
const ControlHistory = lazy(() => import("../pages/backoffice/ControlHistory"));
const ControlDetail = lazy(() => import("../pages/backoffice/ControlDetail"));
const UserManagement = lazy(() => import("../pages/backoffice/UserManagement"));
const PendingVehicles = lazy(() => import("../pages/backoffice/PendingVehicles"));

export interface AppRoute {
    path: string;
    component: React.ComponentType;
    allowedRoles?: UserRole[];
    redirectTo?: string;
    replace?: boolean;
}

export const publicRoutes: AppRoute[] = [
    { path: "/login", component: Login as any },
    { path: "/", component: null as any, redirectTo: "/login", replace: true },
];

export const mobileRoutes: AppRoute[] = [
    { path: "/mobile", component: MobileDashboard as any, allowedRoles: ['agent', 'supervisor'] },
    { path: "/mobile/search", component: MobileSearch as any, allowedRoles: ['agent', 'supervisor'] },
    { path: "/mobile/scan", component: MobileScan as any, allowedRoles: ['agent', 'supervisor'] },
    { path: "/mobile/history", component: MobileHistory as any, allowedRoles: ['agent', 'supervisor'] },
    { path: "/mobile/profile", component: MobileProfile as any, allowedRoles: ['agent', 'supervisor'] },
    { path: "/mobile/vehicle/:plateNumber", component: MobileVehicleResult as any, allowedRoles: ['agent', 'supervisor'] },
    { path: "/mobile/carte-grise", component: MobileCarteGrise as any, allowedRoles: ['agent', 'supervisor'] },
    { path: "/mobile/settings", component: null as any, redirectTo: "/mobile/profile", replace: true, allowedRoles: ['agent', 'supervisor'] },
];

export const backOfficeRoutes: AppRoute[] = [
    { path: "/backoffice", component: BackOfficeDashboard as any, allowedRoles: ['admin', 'supervisor'] },
    { path: "/backoffice/controls", component: ControlHistory as any, allowedRoles: ['admin', 'supervisor'] },
    { path: "/backoffice/controls/:controlId", component: ControlDetail as any, allowedRoles: ['admin', 'supervisor'] },
    { path: "/backoffice/users", component: UserManagement as any, allowedRoles: ['admin'] },
    { path: "/backoffice/validation", component: PendingVehicles as any, allowedRoles: ['admin'] },
    { path: "/backoffice/vehicles", component: BackOfficeDashboard as any, allowedRoles: ['admin'] },
    { path: "/backoffice/organizations", component: BackOfficeDashboard as any, allowedRoles: ['admin'] },
    { path: "/backoffice/audit", component: BackOfficeDashboard as any, allowedRoles: ['admin'] },
    { path: "/backoffice/settings", component: BackOfficeDashboard as any, allowedRoles: ['admin'] },
];

export const catchAllRoute = { path: "*", component: NotFound as any };
