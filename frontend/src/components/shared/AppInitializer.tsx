import { useEffect, ReactNode } from 'react';
import { KeyManagement } from '@/services/keyManagement/keyManagement';

interface AppInitializerProps {
  children: ReactNode;
}

const MAX_INIT_ATTEMPTS = 3;
const RETRY_BASE_DELAY_MS = 500;

/**
 * Best-effort attempt at (re)initializing the device key pair. Returns true
 * on success, false on failure. Errors are logged but never rethrown — this
 * runs at app boot and must not block rendering.
 */
async function tryInitializeKeys(): Promise<boolean> {
  try {
    await KeyManagement();
    return true;
  } catch (error) {
    console.error('[AppInitializer] Failed to initialize cryptographic keys:', error);
    return false;
  }
}

/**
 * AppInitializer handles global initialization at app boot. Its main job is
 * to make the device key pair available in IndexedDB before any code path
 * that signs a nonce runs (refresh flow, activation).
 *
 * KeyManagement() now propagates transient errors (IndexedDB hiccups, locked
 * databases) instead of silently wiping the device identity — so this
 * component owns the retry/recovery loop:
 *   1. Try a few times with a short backoff at startup.
 *   2. If still failing, retry once more the next time the tab becomes
 *      visible (typical mobile-wake scenario after the OS releases IndexedDB).
 * The app is never blocked from rendering; downstream calls that actually
 * need a key will error and the user's next foreground event triggers
 * another attempt.
 */
export const AppInitializer = ({ children }: AppInitializerProps) => {
  useEffect(() => {
    let cancelled = false;
    let visibilityHandler: (() => void) | null = null;

    const attachVisibilityRetry = () => {
      if (typeof document === 'undefined') return;
      visibilityHandler = () => {
        if (document.visibilityState !== 'visible') return;
        void (async () => {
          const ok = await tryInitializeKeys();
          if (ok && visibilityHandler) {
            document.removeEventListener('visibilitychange', visibilityHandler);
            visibilityHandler = null;
          }
        })();
      };
      document.addEventListener('visibilitychange', visibilityHandler);
    };

    (async () => {
      for (let attempt = 1; attempt <= MAX_INIT_ATTEMPTS; attempt++) {
        if (cancelled) return;
        const ok = await tryInitializeKeys();
        if (ok) return;
        if (attempt < MAX_INIT_ATTEMPTS) {
          const delay = RETRY_BASE_DELAY_MS * attempt;
          await new Promise((resolve) => setTimeout(resolve, delay));
        }
      }
      if (!cancelled) attachVisibilityRetry();
    })();

    return () => {
      cancelled = true;
      if (visibilityHandler && typeof document !== 'undefined') {
        document.removeEventListener('visibilitychange', visibilityHandler);
      }
    };
  }, []);

  return <>{children}</>;
};
