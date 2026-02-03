import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { AuthProvider, RequireAuth } from "@/contexts/AuthContext";

// Pages
import Login from "./pages/auth/Login";

import NotFound from "./pages/NotFound";

// Mobile Pages
import MobileDashboard from "./pages/mobile/MobileDashboard";
import MobileSearch from "./pages/mobile/MobileSearch";
import MobileScan from "./pages/mobile/MobileScan";
import MobileHistory from "./pages/mobile/MobileHistory";
import MobileProfile from "./pages/mobile/MobileProfile";
import MobileVehicleResult from "./pages/mobile/MobileVehicleResult";
import MobileCarteGrise from "./pages/mobile/MobileCarteGrise";

// Back Office Pages
import BackOfficeDashboard from "./pages/backoffice/BackOfficeDashboard";
import ControlHistory from "./pages/backoffice/ControlHistory";
import ControlDetail from "./pages/backoffice/ControlDetail";
import UserManagement from "./pages/backoffice/UserManagement";
import PendingVehicles from "./pages/backoffice/PendingVehicles";

const queryClient = new QueryClient();

const App = () => (
  <QueryClientProvider client={queryClient}>
    <TooltipProvider>
      <Toaster />
      <Sonner />
      <BrowserRouter>
        <AuthProvider>
          <Routes>
            {/* Public Routes */}
            <Route path="/" element={<Navigate to="/login" replace />} />
            <Route path="/login" element={<Login />} />


            {/* Mobile Front Office - Agent & Supervisor */}
            <Route
              path="/mobile"
              element={
                <RequireAuth allowedRoles={['agent', 'supervisor']}>
                  <MobileDashboard />
                </RequireAuth>
              }
            />
            <Route
              path="/mobile/search"
              element={
                <RequireAuth allowedRoles={['agent', 'supervisor']}>
                  <MobileSearch />
                </RequireAuth>
              }
            />
            <Route
              path="/mobile/scan"
              element={
                <RequireAuth allowedRoles={['agent', 'supervisor']}>
                  <MobileScan />
                </RequireAuth>
              }
            />
            <Route
              path="/mobile/history"
              element={
                <RequireAuth allowedRoles={['agent', 'supervisor']}>
                  <MobileHistory />
                </RequireAuth>
              }
            />
            <Route
              path="/mobile/profile"
              element={
                <RequireAuth allowedRoles={['agent', 'supervisor']}>
                  <MobileProfile />
                </RequireAuth>
              }
            />
            <Route
              path="/mobile/vehicle/:plateNumber"
              element={
                <RequireAuth allowedRoles={['agent', 'supervisor']}>
                  <MobileVehicleResult />
                </RequireAuth>
              }
            />
            <Route
              path="/mobile/carte-grise"
              element={
                <RequireAuth allowedRoles={['agent', 'supervisor']}>
                  <MobileCarteGrise />
                </RequireAuth>
              }
            />
            <Route path="/mobile/settings" element={<Navigate to="/mobile/profile" replace />} />

            {/* Back Office - Admin & Supervisor */}
            <Route
              path="/backoffice"
              element={
                <RequireAuth allowedRoles={['admin', 'supervisor']}>
                  <BackOfficeDashboard />
                </RequireAuth>
              }
            />
            <Route
              path="/backoffice/controls"
              element={
                <RequireAuth allowedRoles={['admin', 'supervisor']}>
                  <ControlHistory />
                </RequireAuth>
              }
            />
            <Route
              path="/backoffice/controls/:controlId"
              element={
                <RequireAuth allowedRoles={['admin', 'supervisor']}>
                  <ControlDetail />
                </RequireAuth>
              }
            />
            <Route
              path="/backoffice/users"
              element={
                <RequireAuth allowedRoles={['admin']}>
                  <UserManagement />
                </RequireAuth>
              }
            />
            <Route
              path="/backoffice/validation"
              element={
                <RequireAuth allowedRoles={['admin']}>
                  <PendingVehicles />
                </RequireAuth>
              }
            />
            <Route
              path="/backoffice/vehicles"
              element={
                <RequireAuth allowedRoles={['admin']}>
                  <BackOfficeDashboard />
                </RequireAuth>
              }
            />
            <Route
              path="/backoffice/organizations"
              element={
                <RequireAuth allowedRoles={['admin']}>
                  <BackOfficeDashboard />
                </RequireAuth>
              }
            />
            <Route
              path="/backoffice/audit"
              element={
                <RequireAuth allowedRoles={['admin']}>
                  <BackOfficeDashboard />
                </RequireAuth>
              }
            />
            <Route
              path="/backoffice/settings"
              element={
                <RequireAuth allowedRoles={['admin']}>
                  <BackOfficeDashboard />
                </RequireAuth>
              }
            />

            {/* Catch-all */}
            <Route path="*" element={<NotFound />} />
          </Routes>
        </AuthProvider>
      </BrowserRouter>
    </TooltipProvider>
  </QueryClientProvider>
);

export default App;
