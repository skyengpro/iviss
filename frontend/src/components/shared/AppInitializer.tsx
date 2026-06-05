import { useEffect, ReactNode } from 'react';
import { KeyManagement } from '@/services/keyManagement/keyManagement';

interface AppInitializerProps {
  children: ReactNode;
}

/**
 * AppInitializer component handles global application initialization logic.
 * Specifically, it ensures that cryptographic key pairs are generated and
 * available in IndexedDB as soon as the app starts.
 */
export const AppInitializer = ({ children }: AppInitializerProps) => {
  useEffect(() => {
    const initializeKeys = async () => {
      try {
        await KeyManagement();
        console.log('Cryptographic keys initialized successfully');
      } catch (error) {
        console.error('Failed to initialize cryptographic keys:', error);
      }
    };

    initializeKeys();
  }, []);

  return <>{children}</>;
};
