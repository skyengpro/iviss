import React from "react";
import { RequireAuth } from "@/contexts/AuthContext";
import { UserRole } from "@/services/mockAuth";

interface ProtectedRouteProps {
    children: React.ReactNode;
    allowedRoles?: UserRole[];
}

export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ children, allowedRoles }) => {
    return (
        <RequireAuth allowedRoles={allowedRoles}>
            {children}
        </RequireAuth>
    );
};
