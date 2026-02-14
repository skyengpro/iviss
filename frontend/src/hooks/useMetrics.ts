import { useEffect, useRef } from 'react';
import { useLocation } from 'react-router-dom';
import {
    initMetrics,
    recordNavigation,
    destroyMetrics,
} from '@/services/metricsCollector';

/**
 * React hook that initializes the frontend metrics collector
 * and tracks route navigations.
 *
 * Should be used once in the App component (inside BrowserRouter).
 */
export function useMetrics(): void {
    const location = useLocation();
    const isFirstRender = useRef(true);

    // Initialize metrics collector on mount
    useEffect(() => {
        initMetrics();
        return () => {
            destroyMetrics();
        };
    }, []);

    // Track route changes (skip the initial render)
    useEffect(() => {
        if (isFirstRender.current) {
            isFirstRender.current = false;
            return;
        }
        recordNavigation();
    }, [location.pathname]);
}
