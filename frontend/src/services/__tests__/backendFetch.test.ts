import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Mock tokenManager before importing the module under test
vi.mock('../auth/tokenManager', () => ({
  getAccessToken: vi.fn(),
}));

import { getAccessToken } from '../auth/tokenManager';
import { fetchWithAuth } from '../api/backendFetch';

const mockedGetAccessToken = vi.mocked(getAccessToken);

describe('fetchWithAuth', () => {
  const BASE_URL = 'http://localhost:3000';
  let fetchSpy: ReturnType<typeof vi.fn>;
  let dispatchSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    vi.stubEnv('VITE_API_URL', BASE_URL);

    fetchSpy = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ ok: true }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    );
    vi.stubGlobal('fetch', fetchSpy);

    dispatchSpy = vi.spyOn(window, 'dispatchEvent').mockImplementation(() => true);
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllEnvs();
    vi.unstubAllGlobals();
  });

  it('should attach Authorization header when token exists', async () => {
    mockedGetAccessToken.mockReturnValue('test-token');

    await fetchWithAuth(`${BASE_URL}/api/test`);

    const calledHeaders = fetchSpy.mock.calls[0][1].headers;
    expect(calledHeaders.get('Authorization')).toBe('Bearer test-token');
  });

  it('should not attach Authorization header for external URLs', async () => {
    mockedGetAccessToken.mockReturnValue('test-token');

    await fetchWithAuth('https://external-api.com/data');

    const calledHeaders = fetchSpy.mock.calls[0][1].headers;
    expect(calledHeaders.get('Authorization')).toBeNull();
  });

  it('should not overwrite an existing Authorization header', async () => {
    mockedGetAccessToken.mockReturnValue('stored-token');

    await fetchWithAuth(`${BASE_URL}/api/test`, {
      headers: { Authorization: 'Bearer explicit-token' },
    });

    const calledHeaders = fetchSpy.mock.calls[0][1].headers;
    expect(calledHeaders.get('Authorization')).toBe('Bearer explicit-token');
  });

  it('should not attach header when no token exists', async () => {
    mockedGetAccessToken.mockReturnValue(null);

    await fetchWithAuth(`${BASE_URL}/api/test`);

    const calledHeaders = fetchSpy.mock.calls[0][1].headers;
    expect(calledHeaders.get('Authorization')).toBeNull();
  });

  it('should NOT dispatch iviss:session-revoked on 401 (handled by auth interceptor)', async () => {
    mockedGetAccessToken.mockReturnValue('expired-token');
    fetchSpy.mockResolvedValueOnce(new Response('Unauthorized', { status: 401 }));

    await fetchWithAuth(`${BASE_URL}/api/test`);

    // 401s are now handled by the auth interceptor for token refresh
    // backendFetch should not dispatch session-revoked for generic 401s
    expect(dispatchSpy).not.toHaveBeenCalled();
  });

  it('should dispatch iviss:session-revoked when body contains SESSION_REVOKED code', async () => {
    mockedGetAccessToken.mockReturnValue('test-token');
    fetchSpy.mockResolvedValueOnce(
      new Response(JSON.stringify({ code: 'SESSION_REVOKED' }), {
        status: 403,
        headers: { 'Content-Type': 'application/json' },
      })
    );

    await fetchWithAuth(`${BASE_URL}/api/test`);

    expect(dispatchSpy).toHaveBeenCalledTimes(1);
    const event = dispatchSpy.mock.calls[0][0] as CustomEvent;
    expect(event.type).toBe('iviss:session-revoked');
  });

  it('should not dispatch session-revoked on successful responses', async () => {
    mockedGetAccessToken.mockReturnValue('test-token');

    await fetchWithAuth(`${BASE_URL}/api/test`);

    expect(dispatchSpy).not.toHaveBeenCalled();
  });

  it('should prepend base URL for relative paths', async () => {
    mockedGetAccessToken.mockReturnValue(null);

    await fetchWithAuth('/api/data');

    const calledUrl = fetchSpy.mock.calls[0][0];
    expect(calledUrl).toBe(`${BASE_URL}/api/data`);
  });
});
