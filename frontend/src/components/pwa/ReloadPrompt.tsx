import { useRegisterSW } from 'virtual:pwa-register/react';
import { useEffect } from 'react';
import { toast } from 'sonner';

export function ReloadPrompt() {
    const {
        needRefresh: [needRefresh, setNeedRefresh],
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
        if (needRefresh) {
            toast.info('New content available', {
                action: {
                    label: 'Reload',
                    onClick: () => {
                        updateServiceWorker(true);
                        setNeedRefresh(false);
                    },
                },
                duration: Infinity,
                onDismiss: () => setNeedRefresh(false),
            });
        }
    }, [needRefresh, updateServiceWorker, setNeedRefresh]);

    return null;
}
