import { Toaster } from '@/components/ui/toaster';
import { Toaster as Sonner } from '@/components/ui/sonner';
import { TooltipProvider } from '@/components/ui/tooltip';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AuthProvider } from '@/contexts/AuthContext';
import { AppRouter } from '@/router/AppRouter';
import { useMetrics } from '@/hooks/useMetrics';

import { client } from '@/openapi-rq/requests/services.gen';

import { AppInitializer } from '@/components/shared/AppInitializer';

const queryClient = new QueryClient();

// Configure the generated API client
client.setConfig({
  baseUrl: import.meta.env.VITE_API_URL || 'http://localhost:3000',
});

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
