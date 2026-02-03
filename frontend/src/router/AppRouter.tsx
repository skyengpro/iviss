import { Suspense } from "react";
import { Routes, Route, Navigate } from "react-router-dom";
import { Loader2 } from "lucide-react";
import { publicRoutes, mobileRoutes, backOfficeRoutes, catchAllRoute } from "./routes";
import { ProtectedRoute } from "./ProtectedRoute";

const PageLoader = () => (
    <div className="flex h-screen w-full items-center justify-center bg-background">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
    </div>
);

export const AppRouter = () => {
    const allProtectedRoutes = [...mobileRoutes, ...backOfficeRoutes];

    return (
        <Suspense fallback={<PageLoader />}>
            <Routes>
                {/* Public Routes */}
                {publicRoutes.map((route) => (
                    <Route
                        key={route.path}
                        path={route.path}
                        element={
                            route.redirectTo
                                ? <Navigate to={route.redirectTo} replace={route.replace} />
                                : route.component ? <route.component /> : null
                        }
                    />
                ))}

                {/* Protected Routes */}
                {allProtectedRoutes.map((route) => (
                    <Route
                        key={route.path}
                        path={route.path}
                        element={
                            <ProtectedRoute allowedRoles={route.allowedRoles}>
                                {route.redirectTo
                                    ? <Navigate to={route.redirectTo} replace={route.replace} />
                                    : <route.component />
                                }
                            </ProtectedRoute>
                        }
                    />
                ))}

                {/* Catch-all */}
                <Route path={catchAllRoute.path} element={<catchAllRoute.component />} />
            </Routes>
        </Suspense>
    );
};
