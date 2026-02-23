import { createRoot } from 'react-dom/client';
import App from './App.tsx';
import './index.css';
import './i18n/config.ts';
import { Suspense } from 'react';

createRoot(document.getElementById('root')!).render(
  <Suspense fallback="loading...">
    <App />
  </Suspense>
);
