import { useState, useEffect, useRef } from 'react';

interface GeolocationState {
  lat: number | null;
  lng: number | null;
  accuracy: number | null;
  error: string | null;
  loading: boolean;
  permissionDenied: boolean;
}

export const useGeolocation = (options?: PositionOptions) => {
  const [state, setState] = useState<GeolocationState>({
    lat: null,
    lng: null,
    accuracy: null,
    error: null,
    loading: true,
    permissionDenied: false,
  });

  // Stable ref for options so changing the object reference doesn't restart the effect
  const optionsRef = useRef(options);
  optionsRef.current = options;

  useEffect(() => {
    if (!navigator.geolocation) {
      setState((s) => ({
        ...s,
        error: 'Geolocation not supported',
        loading: false,
        permissionDenied: false,
      }));
      return;
    }

    const handleSuccess = (position: GeolocationPosition) => {
      setState({
        lat: position.coords.latitude,
        lng: position.coords.longitude,
        accuracy: position.coords.accuracy,
        error: null,
        loading: false,
        permissionDenied: false,
      });
    };

    const handleError = (error: GeolocationPositionError) => {
      let errorMessage = 'An unknown error occurred';
      let permissionDenied = false;
      switch (error.code) {
        case error.PERMISSION_DENIED:
          errorMessage = 'User denied the request for Geolocation';
          permissionDenied = true;
          break;
        case error.POSITION_UNAVAILABLE:
          errorMessage = 'Location information is unavailable';
          break;
        case error.TIMEOUT:
          errorMessage = 'The request to get user location timed out';
          break;
      }
      setState((s) => ({ ...s, error: errorMessage, loading: false, permissionDenied }));
    };

    let watchId: number;

    // Check permission state first to avoid prompting when already denied
    if ('permissions' in navigator) {
      navigator.permissions.query({ name: 'geolocation' }).then((result) => {
        if (result.state === 'denied') {
          setState((s) => ({
            ...s,
            error: 'Location access denied. Please enable it in your browser settings.',
            loading: false,
            permissionDenied: true,
          }));
          return;
        }

        // 'granted' or 'prompt' — proceed normally
        navigator.geolocation.getCurrentPosition(handleSuccess, handleError, optionsRef.current);
        watchId = navigator.geolocation.watchPosition(
          handleSuccess,
          handleError,
          optionsRef.current
        );

        // React to permission changes (e.g. user re-enables in settings)
        result.onchange = () => {
          if (result.state === 'denied') {
            setState((s) => ({
              ...s,
              error: 'Location access denied. Please enable it in your browser settings.',
              loading: false,
              permissionDenied: true,
            }));
            navigator.geolocation.clearWatch(watchId);
          } else if (result.state === 'granted') {
            setState((s) => ({ ...s, error: null, loading: true, permissionDenied: false }));
            navigator.geolocation.getCurrentPosition(
              handleSuccess,
              handleError,
              optionsRef.current
            );
            watchId = navigator.geolocation.watchPosition(
              handleSuccess,
              handleError,
              optionsRef.current
            );
          }
        };
      });
    } else {
      // Fallback for browsers without Permissions API
      navigator.geolocation.getCurrentPosition(handleSuccess, handleError, optionsRef.current);
      watchId = navigator.geolocation.watchPosition(handleSuccess, handleError, optionsRef.current);
    }

    return () => {
      if (watchId !== undefined) navigator.geolocation.clearWatch(watchId);
    };
  }, []); // Empty deps — options are read via ref, no restarts on re-render

  return state;
};
