import { useState, useEffect, useCallback, useRef, ReactNode } from 'react';
import { mockAuthService } from '@/services/mockAuth';
import { AuthContext, AuthContextType } from '@/hooks/auth/use-auth';
import { UserProfile, AuthResponse } from '@/openapi-rq/requests/types.gen';
import { getDeviceId } from '@/services/deviceId';

const SHIFT_TOKEN_KEY = 'iviss_shift_token';
const SHIFT_EXPIRES_KEY = 'iviss_shift_expires_at';

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserProfile | null>(null);
  const [session, setSession] = useState<AuthResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // Shift session state
  const [shiftToken, setShiftToken] = useState<string | null>(null);
  const [shiftExpiresAt, setShiftExpiresAt] = useState<Date | null>(null);
  const shiftTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Compute whether the shift is currently active
  const isShiftActive = !!(shiftToken && shiftExpiresAt && shiftExpiresAt > new Date());

  // Clear the shift expiration timer
  const clearShiftTimer = useCallback(() => {
    if (shiftTimerRef.current) {
      clearTimeout(shiftTimerRef.current);
      shiftTimerRef.current = null;
    }
  }, []);

  // Clear shift session from state and storage
  const clearShiftSession = useCallback(() => {
    clearShiftTimer();
    setShiftToken(null);
    setShiftExpiresAt(null);
    sessionStorage.removeItem(SHIFT_TOKEN_KEY);
    sessionStorage.removeItem(SHIFT_EXPIRES_KEY);
  }, [clearShiftTimer]);

  // Set shift session — stores token and schedules auto-expiration
  const setShiftSession = useCallback(
    (token: string, expiresIn: number) => {
      const expiresAt = new Date(Date.now() + expiresIn * 1000);

      setShiftToken(token);
      setShiftExpiresAt(expiresAt);

      // Persist to sessionStorage (survives page refreshes within same tab)
      sessionStorage.setItem(SHIFT_TOKEN_KEY, token);
      sessionStorage.setItem(SHIFT_EXPIRES_KEY, expiresAt.toISOString());

      // Schedule auto-clear when the shift expires
      clearShiftTimer();
      shiftTimerRef.current = setTimeout(() => {
        clearShiftSession();
      }, expiresIn * 1000);
    },
    [clearShiftTimer, clearShiftSession]
  );

  // Initialize identity and check for existing session on mount
  useEffect(() => {
    const initIdentity = async () => {
      // Ensure device_id is generated and stored in IndexedDB
      await getDeviceId();

      const existingSession = mockAuthService.getSession() as unknown as AuthResponse | null;
      if (existingSession) {
        setSession(existingSession);
        setUser(existingSession.user);
      }

      // Restore shift session from sessionStorage
      const storedToken = sessionStorage.getItem(SHIFT_TOKEN_KEY);
      const storedExpires = sessionStorage.getItem(SHIFT_EXPIRES_KEY);
      if (storedToken && storedExpires) {
        const expiresAt = new Date(storedExpires);
        if (expiresAt > new Date()) {
          const remainingMs = expiresAt.getTime() - Date.now();
          setShiftToken(storedToken);
          setShiftExpiresAt(expiresAt);
          // Schedule auto-clear for the remaining time
          shiftTimerRef.current = setTimeout(() => {
            clearShiftSession();
          }, remainingMs);
        } else {
          // Expired — clean up
          sessionStorage.removeItem(SHIFT_TOKEN_KEY);
          sessionStorage.removeItem(SHIFT_EXPIRES_KEY);
        }
      }

      setIsLoading(false);
    };

    initIdentity();

    return () => {
      clearShiftTimer();
    };
  }, [clearShiftSession, clearShiftTimer]);

  const login = async (username: string, password: string) => {
    const result = await mockAuthService.login(username, password);

    if (result.success && result.session) {
      const backendSession = result.session as unknown as AuthResponse;
      setSession(backendSession);
      setUser(backendSession.user);
      return { success: true };
    }

    return { success: false, error: result.error };
  };

  const logout = async () => {
    await mockAuthService.logout();
    setSession(null);
    setUser(null);
    clearShiftSession();
  };

  const getMockCredentials = () => mockAuthService.getMockCredentials();

  const value: AuthContextType = {
    user,
    session,
    isLoading,
    isAuthenticated: !!session,
    login,
    logout,
    getMockCredentials,
    shiftToken,
    shiftExpiresAt,
    isShiftActive,
    setShiftSession,
    clearShiftSession,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

