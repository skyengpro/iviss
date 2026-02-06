import { useRegisterSW } from 'virtual:pwa-register/react';
import { toast } from 'sonner';
import { useEffect } from 'react';
import { RefreshCw } from 'lucide-react';

export function ReloadPrompt() {
    const {
        offlineReady: [offlineReady, setOfflineReady],
        needRefresh: [needRefresh],
        updateServiceWorker,
    } = useRegisterSW({
        onRegistered(r) {
            console.log('SW Registered: ' + r);
        },
        onRegisterError(error) {
            console.log('SW registration error', error);
        },
    });

    useEffect(() => {
        if (offlineReady) {
            toast('App ready to work offline', {
                description: 'You can now use IVISS without internet connection.',
                action: {
                    label: 'Close',
                    onClick: () => setOfflineReady(false),
                },
            });
        }
    }, [offlineReady, setOfflineReady]);

    useEffect(() => {
        if (needRefresh) {
            toast('New content available', {
                description: 'Update to the latest version for new features.',
                icon: <RefreshCw className="h-4 w-4" />,
                action: {
                    label: 'Update',
                    onClick: () => updateServiceWorker(true),
                },
                duration: Infinity,
            });
        }
    }, [needRefresh, updateServiceWorker]);

    return null;
}
