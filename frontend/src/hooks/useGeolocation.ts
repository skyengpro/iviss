import { useState, useEffect } from 'react';

interface GeolocationState {
    lat: number | null;
    lng: number | null;
    accuracy: number | null;
    error: string | null;
    loading: boolean;
}

export const useGeolocation = (options?: PositionOptions) => {
    const [state, setState] = useState<GeolocationState>({
        lat: null,
        lng: null,
        accuracy: null,
        error: null,
        loading: true,
    });

    useEffect(() => {
        if (!navigator.geolocation) {
            setState((s) => ({ ...s, error: 'Geolocation not supported', loading: false }));
            return;
        }

        const handleSuccess = (position: GeolocationPosition) => {
            setState({
                lat: position.coords.latitude,
                lng: position.coords.longitude,
                accuracy: position.coords.accuracy,
                error: null,
                loading: false,
            });
        };

        const handleError = (error: GeolocationPositionError) => {
            let errorMessage = 'An unknown error occurred';
            switch (error.code) {
                case error.PERMISSION_DENIED:
                    errorMessage = 'User denied the request for Geolocation';
                    break;
                case error.POSITION_UNAVAILABLE:
                    errorMessage = 'Location information is unavailable';
                    break;
                case error.TIMEOUT:
                    errorMessage = 'The request to get user location timed out';
                    break;
            }
            setState((s) => ({ ...s, error: errorMessage, loading: false }));
        };

        navigator.geolocation.getCurrentPosition(handleSuccess, handleError, options);

        // Watch for changes
        const watchId = navigator.geolocation.watchPosition(handleSuccess, handleError, options);

        return () => navigator.geolocation.clearWatch(watchId);
    }, [options]);

    return state;
};
