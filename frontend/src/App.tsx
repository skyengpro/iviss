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
import { PWAInstallPrompt } from '@/components/shared/PWAInstallPrompt';

const queryClient = new QueryClient();

// Configure the generated API client
const apiBaseUrl = import.meta.env.VITE_API_URL || 'http://localhost:3000';

client.setConfig({
  baseUrl: apiBaseUrl,
});

// Register auth interceptors for automatic token refresh with device signature
setupAuthInterceptors(client, {
  baseUrl: apiBaseUrl,
  onSessionExpired: () => {
    // Dispatch event to trigger AuthContext's globalLogout which shows a toast
    window.dispatchEvent(new CustomEvent('iviss:session-revoked'));

    // Fallback in case AuthContext isn't mounted
    setTimeout(() => {
      if (window.location.pathname !== '/daily-login' && window.location.pathname !== '/login') {
        clearTokens();
        localStorage.removeItem('iviss_session');
        localStorage.removeItem('iviss_refresh_token');
        window.location.href = '/daily-login';
      }
    }, 100);
  },
});

const AppInner = () => {
  useMetrics();

  return (
    <AuthProvider>
      <AppRouter />
      <PWAInstallPrompt />
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
