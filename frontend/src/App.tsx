import { Toaster } from '@/components/ui/toaster';
import { Toaster as Sonner } from '@/components/ui/sonner';
import { TooltipProvider } from '@/components/ui/tooltip';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BrowserRouter } from 'react-router-dom';
import { AuthProvider } from '@/contexts/AuthContext';
import { AppRouter } from '@/router/AppRouter';
import { ErrorBoundary } from '@/components/shared/ErrorBoundary';

import { client } from '@/openapi-rq/requests/services.gen';

const queryClient = new QueryClient();

// Configure the generated API client
client.setConfig({
  baseUrl: import.meta.env.VITE_API_URL || 'http://localhost:3000',
});

const App = () => (
  <QueryClientProvider client={queryClient}>
    <TooltipProvider>
      <Toaster />
      <Sonner />
      <BrowserRouter>
        <ErrorBoundary>
          <AuthProvider>
            <AppRouter />
          </AuthProvider>
        </ErrorBoundary>
      </BrowserRouter>
    </TooltipProvider>
  </QueryClientProvider>
);

export default App;
