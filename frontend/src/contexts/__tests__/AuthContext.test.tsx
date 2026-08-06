import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { ReactNode } from 'react';
import { AuthProvider } from '../AuthContext';

// ─── Mock all external services ─────────────────────────────────────────────

vi.mock('@/openapi-rq/requests/services.gen', () => ({
  client: {
    setConfig: vi.fn(),
    interceptors: {
      response: { use: vi.fn(), eject: vi.fn() },
    },
  },
  activateDevice: vi.fn(),
  getUserProfile: vi.fn(),
  requestDailyLogin: vi.fn(),
  verifyDailyLogin: vi.fn(),
  loginUser: vi.fn(),
  logoutUser: vi.fn(),
}));

vi.mock('@/services/auth/tokenManager', async (importOriginal) => {
  // Keep the real isTokenExpired/getTokenExpiry (pure JWT-decoding helpers)
  // so the makeJwt()-based fixtures below are evaluated with production
  // logic; only the storage-touching functions are mocked.
  const actual = await importOriginal<typeof import('@/services/auth/tokenManager')>();
  return {
    ...actual,
    setAccessToken: vi.fn(),
    setRefreshToken: vi.fn(),
    getAccessToken: vi.fn().mockReturnValue(null),
    getRefreshToken: vi.fn().mockReturnValue(null),
    clearAccessToken: vi.fn(),
  };
});

vi.mock('@/services/device/deviceId', () => ({
  getDeviceId: vi.fn().mockResolvedValue('test-device-id'),
}));

import {
  client,
  activateDevice,
  getUserProfile,
  requestDailyLogin,
  verifyDailyLogin,
  loginUser,
  logoutUser,
} from '@/openapi-rq/requests/services.gen';
import * as tokenManager from '@/services/auth/tokenManager';
import { useAuth } from '@/hooks/auth/use-auth';

// ─── JWT helpers ─────────────────────────────────────────────────────────────
function makeJwt(exp: number) {
  const payload = btoa(JSON.stringify({ exp, sub: 'u1' }))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
  return `header.${payload}.sig`;
}

const FUTURE_EXP = Math.floor(Date.now() / 1000) + 3600;
const PAST_EXP = Math.floor(Date.now() / 1000) - 3600;

const SESSION_KEY = 'iviss_session';
const REFRESH_KEY = 'iviss_refresh_token';

// ─── Wrapper ─────────────────────────────────────────────────────────────────
function wrapper({ children }: { children: ReactNode }) {
  return <AuthProvider>{children}</AuthProvider>;
}

// ─── Tests ────────────────────────────────────────────────────────────────────
describe('AuthProvider', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    sessionStorage.clear();
    // Reset interceptor guard between tests
    vi.mocked(client.interceptors.response.use).mockClear();
  });

  afterEach(() => {
    localStorage.clear();
    sessionStorage.clear();
  });

  // ── Init ──────────────────────────────────────────────────────────────────

  it('sets isLoading to false after mount with no persisted session', async () => {
    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.isAuthenticated).toBe(false);
    expect(result.current.user).toBeNull();
  });

  it('restores a valid session from localStorage', async () => {
    const mockUser = { id: 'u1', role: 'agent', organizationId: 'org1' };
    const session = { accessToken: makeJwt(FUTURE_EXP), user: mockUser };
    localStorage.setItem(SESSION_KEY, JSON.stringify(session));
    localStorage.setItem(REFRESH_KEY, 'refresh-tok');

    vi.mocked(tokenManager.getAccessToken).mockReturnValue(null);

    const { result } = renderHook(() => useAuth(), { wrapper });

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.isAuthenticated).toBe(true);
    expect(result.current.user).toEqual(mockUser);
  });

  it('silently removes an expired session from localStorage', async () => {
    const mockUser = { id: 'u1', role: 'agent', organizationId: 'org1' };
    const session = { accessToken: makeJwt(PAST_EXP), user: mockUser };
    localStorage.setItem(SESSION_KEY, JSON.stringify(session));

    const { result } = renderHook(() => useAuth(), { wrapper });

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.isAuthenticated).toBe(false);
    expect(localStorage.getItem(SESSION_KEY)).toBeNull();
  });

  it('silently removes an unparseable session from localStorage', async () => {
    localStorage.setItem(SESSION_KEY, 'NOT_JSON');

    const { result } = renderHook(() => useAuth(), { wrapper });

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.isAuthenticated).toBe(false);
    expect(localStorage.getItem(SESSION_KEY)).toBeNull();
  });

  // ── login() ───────────────────────────────────────────────────────────────

  it('login() — success: stores tokens and sets user', async () => {
    const mockUser = { id: 'u1', role: 'admin', organizationId: 'org1' };
    vi.mocked(loginUser).mockResolvedValueOnce({
      data: { accessToken: 'acc', refreshToken: 'ref', user: mockUser },
      error: undefined,
    } as never);

    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    let loginResult!: Awaited<ReturnType<typeof result.current.login>>;
    await act(async () => {
      loginResult = await result.current.login('admin@test.com', 'password');
    });

    expect(loginResult.success).toBe(true);
    expect(tokenManager.setAccessToken).toHaveBeenCalledWith('acc');
    expect(tokenManager.setRefreshToken).toHaveBeenCalledWith('ref');
    expect(result.current.user).toEqual(mockUser);
  });

  it('login() — API error: returns { success: false, error }', async () => {
    vi.mocked(loginUser).mockResolvedValueOnce({
      data: undefined,
      error: { message: 'Invalid credentials' },
    } as never);

    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    let loginResult!: Awaited<ReturnType<typeof result.current.login>>;
    await act(async () => {
      loginResult = await result.current.login('x@x.com', 'wrong');
    });

    expect(loginResult.success).toBe(false);
    expect(loginResult.error).toBe('Invalid credentials');
  });

  it('login() — no data in response: returns failure', async () => {
    vi.mocked(loginUser).mockResolvedValueOnce({
      data: undefined,
      error: undefined,
    } as never);

    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    let loginResult!: Awaited<ReturnType<typeof result.current.login>>;
    await act(async () => {
      loginResult = await result.current.login('x@x.com', 'pw');
    });

    expect(loginResult.success).toBe(false);
  });

  // ── activate() ────────────────────────────────────────────────────────────

  it('activate() — success: stores tokens and updates session', async () => {
    const mockUser = { id: 'u2', role: 'agent', organizationId: 'org1' };
    vi.mocked(activateDevice).mockResolvedValueOnce({
      data: { accessToken: 'acc2', refreshToken: 'ref2', user: mockUser },
      error: undefined,
    } as never);
    vi.mocked(getUserProfile).mockResolvedValueOnce({
      data: mockUser,
      error: undefined,
    } as never);

    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    let activateResult!: Awaited<ReturnType<typeof result.current.activate>>;
    await act(async () => {
      activateResult = await result.current.activate({
        badgeId: 'B1',
        activationCode: '123456',
        deviceId: 'dev1',
        publicKeyBase64: 'key',
      });
    });

    expect(activateResult.success).toBe(true);
    expect(localStorage.getItem('iviss_device_activated')).toBe('true');
  });

  it('activate() — NOT_FOUND error is humanised', async () => {
    vi.mocked(activateDevice).mockResolvedValueOnce({
      data: undefined,
      error: { code: 'NOT_FOUND', message: 'device is not registered' },
    } as never);

    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    let activateResult!: Awaited<ReturnType<typeof result.current.activate>>;
    await act(async () => {
      activateResult = await result.current.activate({
        badgeId: 'B1',
        activationCode: '000000',
        deviceId: 'dev1',
        publicKeyBase64: 'key',
      });
    });

    expect(activateResult.success).toBe(false);
    expect(activateResult.error).toMatch(/device is not registered/i);
  });

  it('activate() — BAD_REQUEST+expired is humanised', async () => {
    vi.mocked(activateDevice).mockResolvedValueOnce({
      data: undefined,
      error: { code: 'BAD_REQUEST', message: 'OTP code has expired' },
    } as never);

    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    let activateResult!: Awaited<ReturnType<typeof result.current.activate>>;
    await act(async () => {
      activateResult = await result.current.activate({
        badgeId: 'B1',
        activationCode: '000000',
        deviceId: 'dev1',
        publicKeyBase64: 'key',
      });
    });

    expect(activateResult.success).toBe(false);
    expect(activateResult.error).toMatch(/expired/i);
  });

  // ── dailyLoginRequest() ───────────────────────────────────────────────────

  it('dailyLoginRequest() — success', async () => {
    vi.mocked(requestDailyLogin).mockResolvedValueOnce({
      data: {},
      error: undefined,
    } as never);

    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    let res!: Awaited<ReturnType<typeof result.current.dailyLoginRequest>>;
    await act(async () => {
      res = await result.current.dailyLoginRequest({ badgeId: 'B1' });
    });

    expect(res.success).toBe(true);
  });

  it('dailyLoginRequest() — missing badge keeps iviss_device_activated', async () => {
    localStorage.setItem('iviss_device_activated', 'true');
    vi.mocked(requestDailyLogin).mockResolvedValueOnce({
      data: undefined,
      error: { code: 'NOT_FOUND', message: 'User not found' },
    } as never);

    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    let res!: Awaited<ReturnType<typeof result.current.dailyLoginRequest>>;
    await act(async () => {
      res = await result.current.dailyLoginRequest({ badgeId: 'GHOST' });
    });

    expect(res.requiresActivation).toBe(false);
    expect(localStorage.getItem('iviss_device_activated')).toBe('true');
  });

  it('dailyLoginRequest() — unregistered device clears iviss_device_activated', async () => {
    localStorage.setItem('iviss_device_activated', 'true');
    vi.mocked(requestDailyLogin).mockResolvedValueOnce({
      data: undefined,
      error: { code: 'NOT_FOUND', message: 'Device is not registered. Please re-activate.' },
    } as never);

    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    let res!: Awaited<ReturnType<typeof result.current.dailyLoginRequest>>;
    await act(async () => {
      res = await result.current.dailyLoginRequest({ badgeId: 'B1' });
    });

    expect(res.requiresActivation).toBe(true);
    expect(localStorage.getItem('iviss_device_activated')).toBeNull();
  });

  // ── dailyLoginVerify() ────────────────────────────────────────────────────

  it('dailyLoginVerify() — success: stores all tokens', async () => {
    const mockUser = { id: 'u3', role: 'agent', organizationId: 'org1' };
    vi.mocked(verifyDailyLogin).mockResolvedValueOnce({
      data: { accessToken: 'acc3', refreshToken: 'ref3' },
      error: undefined,
    } as never);
    vi.mocked(getUserProfile).mockResolvedValueOnce({
      data: mockUser,
      error: undefined,
    } as never);

    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    let res!: Awaited<ReturnType<typeof result.current.dailyLoginVerify>>;
    await act(async () => {
      res = await result.current.dailyLoginVerify({
        badgeId: 'B1',
        activationCode: '112233',
        deviceId: 'dev1',
      });
    });

    expect(res.success).toBe(true);
    expect(tokenManager.setAccessToken).toHaveBeenCalledWith('acc3');
    expect(tokenManager.setRefreshToken).toHaveBeenCalledWith('ref3');
  });

  // ── logout() ──────────────────────────────────────────────────────────────

  it('logout() — clears all storage and resets state', async () => {
    // Seed a session
    const mockUser = { id: 'u1', role: 'agent', organizationId: 'org1' };
    const session = { accessToken: makeJwt(FUTURE_EXP), user: mockUser };
    localStorage.setItem(SESSION_KEY, JSON.stringify(session));
    localStorage.setItem(REFRESH_KEY, 'ref');
    localStorage.setItem('iviss_device_activated', 'true');

    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await act(async () => {
      await result.current.logout();
    });

    expect(result.current.isAuthenticated).toBe(false);
    expect(result.current.user).toBeNull();
    expect(localStorage.getItem(SESSION_KEY)).toBeNull();
    expect(localStorage.getItem(REFRESH_KEY)).toBeNull();
  });

  it('logout() — calls backend logout endpoint when an access token is present', async () => {
    const mockUser = { id: 'u1', role: 'admin', organizationId: 'org1' };
    const session = { accessToken: makeJwt(FUTURE_EXP), user: mockUser };
    localStorage.setItem(SESSION_KEY, JSON.stringify(session));
    localStorage.setItem(REFRESH_KEY, 'ref');

    vi.mocked(tokenManager.getAccessToken).mockReturnValue(session.accessToken);
    vi.mocked(logoutUser).mockResolvedValueOnce({
      data: undefined,
      error: undefined,
    } as never);

    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await act(async () => {
      await result.current.logout();
    });

    expect(logoutUser).toHaveBeenCalledWith({
      headers: { Authorization: `Bearer ${session.accessToken}` },
      throwOnError: false,
    });
    expect(result.current.isAuthenticated).toBe(false);
  });
});
