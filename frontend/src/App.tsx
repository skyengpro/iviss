import { Toaster } from '@/components/ui/toaster';
import { Toaster as Sonner } from '@/components/ui/sonner';
import { TooltipProvider } from '@/components/ui/tooltip';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AuthProvider } from '@/contexts/AuthContext';
import { AppRouter } from '@/router/AppRouter';
import { useMetrics } from '@/hooks/useMetrics';

import { client } from '@/openapi-rq/requests/services.gen';
import { setupAuthInterceptors } from '@/services/auth/authInterceptor';
import { clearTokens } from '@/services/auth/tokenManager';

import { AppInitializer } from '@/components/shared/AppInitializer';

const queryClient = new QueryClient();

// Configure the generated API client
const apiBaseUrl = import.meta.env.VITE_API_URL || '';

client.setConfig({
  baseUrl: apiBaseUrl,
});

// Register auth interceptors for automatic token refresh with device signature.
// Guard against HMR re-registration — the client singleton persists across reloads
// so calling setupAuthInterceptors multiple times stacks duplicate interceptors.
if (!(window as { __iviss_interceptors_registered?: boolean }).__iviss_interceptors_registered) {
  (window as { __iviss_interceptors_registered?: boolean }).__iviss_interceptors_registered = true;
  setupAuthInterceptors(client, {
    baseUrl: apiBaseUrl,
    onSessionExpired: () => {
      // Dispatch event to trigger AuthContext's globalLogout
      window.dispatchEvent(new CustomEvent('iviss:session-revoked'));
    },
  });
}

const AppInner = () => {
  useMetrics();

  return (
    <AuthProvider>
      <AppRouter />
    </AuthProvider>
  );
};

const App = () => (
  <QueryClientProvider client={queryClient}>
    <AppInitializer>
      <TooltipProvider>
        <Toaster />
        <Sonner />
        <AppInner />
      </TooltipProvider>
    </AppInitializer>
  </QueryClientProvider>
);

export default App;
