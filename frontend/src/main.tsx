import { createRoot } from 'react-dom/client';
import App from './App.tsx';
import './index.css';
import { ErrorBoundary } from './components/shared/ErrorBoundary.tsx';
import { BrowserRouter } from 'react-router-dom';
import './i18n/config.ts';
import { Suspense } from 'react';

createRoot(document.getElementById('root')!).render(
  <Suspense fallback="loading...">
    <BrowserRouter>
      <ErrorBoundary>
        <App />
      </ErrorBoundary>
    </BrowserRouter>
  </Suspense>
);
