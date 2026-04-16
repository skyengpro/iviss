import { useEffect, useState } from 'react';
import { useRegisterSW } from 'virtual:pwa-register/react';

interface BeforeInstallPromptEvent extends Event {
  prompt: () => Promise<void>;
  userChoice: Promise<{ outcome: 'accepted' | 'dismissed' }>;
}

export function usePWA() {
  const [showInstallPrompt, setShowInstallPrompt] = useState(false);
  const [deferredPrompt, setDeferredPrompt] = useState<BeforeInstallPromptEvent | null>(null);

  const {
    offlineReady: [offlineReady],
    needRefresh: [needRefresh],
    updateServiceWorker,
  } = useRegisterSW({
    onRegistered(r) {
      console.log('SW Registered: ' + r);
    },
    onRegisterError(error) {
      console.log('SW registration error', error);
    },
    onNeedRefresh() {
      // Automatically update without showing prompt
      updateServiceWorker(true);
    },
  });

  useEffect(() => {
    const handleBeforeInstallPrompt = (e: Event) => {
      e.preventDefault();
      setDeferredPrompt(e as BeforeInstallPromptEvent);

      // Check if user dismissed the prompt in the last 2 hours
      const dismissedTime = localStorage.getItem('pwa-install-dismissed');
      if (dismissedTime) {
        const hoursSinceDismissed = (Date.now() - parseInt(dismissedTime, 10)) / (1000 * 60 * 60);
        if (hoursSinceDismissed < 2) {
          return;
        }
      }

      // Show prompt after 3 seconds
      setTimeout(() => {
        setShowInstallPrompt(true);
      }, 3000);
    };

    window.addEventListener('beforeinstallprompt', handleBeforeInstallPrompt);

    return () => {
      window.removeEventListener('beforeinstallprompt', handleBeforeInstallPrompt);
    };
  }, []);

  const handleInstall = async () => {
    if (!deferredPrompt) return;

    deferredPrompt.prompt();
    const { outcome } = await deferredPrompt.userChoice;

    if (outcome === 'accepted') {
      console.log('User accepted the install prompt');
    }

    setDeferredPrompt(null);
    setShowInstallPrompt(false);
  };

  const dismissInstall = () => {
    setShowInstallPrompt(false);
    // Dismiss for 2 hours
    localStorage.setItem('pwa-install-dismissed', Date.now().toString());
  };

  return {
    showInstallPrompt,
    handleInstall,
    dismissInstall,
    offlineReady,
    needRefresh,
  };
}
