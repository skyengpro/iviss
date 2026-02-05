import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerFooter,
    DrawerHeader,
    DrawerTitle,
} from '@/components/ui/drawer';

interface BeforeInstallPromptEvent extends Event {
    readonly platforms: string[];
    readonly userChoice: Promise<{
        outcome: 'accepted' | 'dismissed';
        platform: string;
    }>;
    prompt(): Promise<void>;
}

export function InstallPrompt() {
    const [deferredPrompt, setDeferredPrompt] = useState<BeforeInstallPromptEvent | null>(null);
    const [showDrawer, setShowDrawer] = useState(false);

    useEffect(() => {
        const handler = (e: Event) => {
            // Check if user has dismissed the prompt recently (within 7 days)
            const dismissedUntil = localStorage.getItem('pwa-prompt-dismissed-until');
            if (dismissedUntil && Date.now() < parseInt(dismissedUntil, 10)) {
                return;
            }

            // Prevent the mini-infobar from appearing on mobile
            e.preventDefault();
            // Stash the event so it can be triggered later.
            setDeferredPrompt(e as BeforeInstallPromptEvent);
            // Show the custom prompt
            setShowDrawer(true);
        };

        window.addEventListener('beforeinstallprompt', handler);

        return () => {
            window.removeEventListener('beforeinstallprompt', handler);
        };
    }, []);

    const handleInstall = async () => {
        if (!deferredPrompt) return;

        // Show the install prompt
        await deferredPrompt.prompt();

        // Wait for the user to respond to the prompt
        const { outcome } = await deferredPrompt.userChoice;
        console.log(`User response to the install prompt: ${outcome}`);

        // We've used the prompt, and can't use it again, throw it away
        setDeferredPrompt(null);
        setShowDrawer(false);
    };

    const handleClose = () => {
        // Dismiss for 7 days
        const sevenDaysInMs = 7 * 24 * 60 * 60 * 1000;
        localStorage.setItem('pwa-prompt-dismissed-until', (Date.now() + sevenDaysInMs).toString());
        setShowDrawer(false);
    };

    return (
        <Drawer open={showDrawer} onOpenChange={setShowDrawer}>
            <DrawerContent>
                <div className="mx-auto w-full max-w-sm">
                    <DrawerHeader className="flex flex-col items-center text-center">
                        <div className="mb-4 h-16 w-16 overflow-hidden rounded-2xl bg-slate-900 p-2 shadow-lg">
                            <img
                                src="/pwa-192x192.png"
                                alt="IVISS Logo"
                                className="h-full w-full object-contain"
                            />
                        </div>
                        <DrawerTitle className="text-xl">Get the app</DrawerTitle>
                        <DrawerDescription>
                            Install IVISS Security for a faster, more reliable experience—even offline.
                        </DrawerDescription>
                    </DrawerHeader>
                    <DrawerFooter className="flex flex-col gap-2 pt-2">
                        <Button onClick={handleInstall} className="w-full bg-slate-900 text-white hover:bg-slate-800">
                            Install App
                        </Button>
                        <Button variant="ghost" onClick={handleClose} className="w-full">
                            Later
                        </Button>
                    </DrawerFooter>
                </div>
            </DrawerContent>
        </Drawer>
    );
}
