import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

vi.mock('@/openapi-rq/requests/services.gen', () => ({
  requestRefresh: vi.fn(),
  verifyRefresh: vi.fn(),
}));

vi.mock('../../device/deviceId', () => ({
  getDeviceId: vi.fn(),
}));

vi.mock('../signatureService', () => ({
  signNonce: vi.fn(),
}));

import { requestRefresh, verifyRefresh } from '@/openapi-rq/requests/services.gen';
import { getDeviceId } from '../../device/deviceId';
import { signNonce } from '../signatureService';
import { performTokenRefresh, setupAuthInterceptors, REFRESH_TIMEOUT_MS } from '../authInterceptor';

const mockedRequestRefresh = vi.mocked(requestRefresh);
const mockedVerifyRefresh = vi.mocked(verifyRefresh);
const mockedGetDeviceId = vi.mocked(getDeviceId);
const mockedSignNonce = vi.mocked(signNonce);

const RETRY_HEADER = 'X-Auth-Retry';

/** Resolves once the challenge succeeds; used across most agent-flow tests. */
function mockSuccessfulChallenge(accessToken: string) {
  mockedGetDeviceId.mockResolvedValue('device-1');
  mockedRequestRefresh.mockResolvedValue({ data: { nonce: 'nonce-1' }, error: undefined } as any);
  mockedSignNonce.mockResolvedValue('signed-nonce');
  mockedVerifyRefresh.mockResolvedValue({ data: { accessToken }, error: undefined } as any);
}

/** Mimics `fetch`'s real behavior of rejecting an in-flight call on abort. */
function mockAbortableHang() {
  const pending = (opts: { signal?: AbortSignal }) =>
    new Promise((_resolve, reject) => {
      opts.signal?.addEventListener('abort', () =>
        reject(new DOMException('Aborted', 'AbortError'))
      );
    });
  mockedGetDeviceId.mockResolvedValue('device-1');
  mockedRequestRefresh.mockImplementation(pending as any);
}

describe('authInterceptor', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('performTokenRefresh', () => {
    it('returns null without calling the backend when no refresh token is stored', async () => {
      const result = await performTokenRefresh();

      expect(result).toBeNull();
      expect(mockedRequestRefresh).not.toHaveBeenCalled();
    });

    it('completes the 2-step challenge and persists the new access token', async () => {
      localStorage.setItem('iviss_refresh_token', 'rt-1');
      mockSuccessfulChallenge('at-2');

      const result = await performTokenRefresh();

      expect(result).toBe('at-2');
      expect(localStorage.getItem('iviss_access_token')).toBe('at-2');
      expect(mockedRequestRefresh).toHaveBeenCalledTimes(1);
      expect(mockedRequestRefresh).toHaveBeenCalledWith(
        expect.objectContaining({ body: { refreshToken: 'rt-1', deviceId: 'device-1' } })
      );
      expect(mockedVerifyRefresh).toHaveBeenCalledTimes(1);
    });

    it('retries the challenge once when the nonce is raced/expired, then succeeds', async () => {
      localStorage.setItem('iviss_refresh_token', 'rt-1');
      mockedGetDeviceId.mockResolvedValue('device-1');
      mockedRequestRefresh.mockResolvedValue({
        data: { nonce: 'nonce-1' },
        error: undefined,
      } as any);
      mockedSignNonce.mockResolvedValue('signed-nonce');
      mockedVerifyRefresh
        .mockResolvedValueOnce({
          data: undefined,
          error: {
            code: 'UNAUTHORIZED',
            message: 'Nonce expired or not found — request a new challenge',
          },
        } as any)
        .mockResolvedValueOnce({ data: { accessToken: 'at-retry' }, error: undefined } as any);

      const result = await performTokenRefresh();

      expect(result).toBe('at-retry');
      expect(mockedRequestRefresh).toHaveBeenCalledTimes(2);
      expect(mockedVerifyRefresh).toHaveBeenCalledTimes(2);
    });

    it('gives up without throwing after exhausting nonce retries', async () => {
      localStorage.setItem('iviss_refresh_token', 'rt-1');
      mockedGetDeviceId.mockResolvedValue('device-1');
      mockedRequestRefresh.mockResolvedValue({
        data: { nonce: 'nonce-1' },
        error: undefined,
      } as any);
      mockedSignNonce.mockResolvedValue('signed-nonce');
      mockedVerifyRefresh.mockResolvedValue({
        data: undefined,
        error: { code: 'UNAUTHORIZED', message: 'Nonce mismatch' },
      } as any);

      const result = await performTokenRefresh();

      expect(result).toBeNull();
      // Bounded: exactly MAX_NONCE_ATTEMPTS (2), never an unbounded loop.
      expect(mockedRequestRefresh).toHaveBeenCalledTimes(2);
      expect(mockedVerifyRefresh).toHaveBeenCalledTimes(2);
    });

    it('does not retry a genuine "invalid refresh token" rejection', async () => {
      localStorage.setItem('iviss_refresh_token', 'rt-1');
      mockedGetDeviceId.mockResolvedValue('device-1');
      mockedRequestRefresh.mockResolvedValue({
        data: undefined,
        error: { code: 'UNAUTHORIZED', message: 'Invalid or expired refresh token' },
      } as any);

      const result = await performTokenRefresh();

      expect(result).toBeNull();
      expect(mockedRequestRefresh).toHaveBeenCalledTimes(1);
      expect(mockedVerifyRefresh).not.toHaveBeenCalled();
    });

    it('resolves to null instead of hanging or rejecting when the refresh times out, and clears state for the next attempt', async () => {
      vi.useFakeTimers();
      localStorage.setItem('iviss_refresh_token', 'rt-1');
      mockAbortableHang();

      const timedOutPromise = performTokenRefresh();
      await vi.advanceTimersByTimeAsync(REFRESH_TIMEOUT_MS + 100);
      const result = await timedOutPromise;

      expect(result).toBeNull();

      // The poisoned-promise regression: a fresh attempt right after must not
      // be stuck reusing the timed-out promise — it should hit the backend again.
      mockSuccessfulChallenge('at-after-timeout');
      const secondResult = await performTokenRefresh();
      expect(secondResult).toBe('at-after-timeout');
    });

    it('deduplicates concurrent calls into a single in-flight refresh', async () => {
      localStorage.setItem('iviss_refresh_token', 'rt-1');
      mockSuccessfulChallenge('at-shared');

      const [first, second] = await Promise.all([performTokenRefresh(), performTokenRefresh()]);

      expect(first).toBe('at-shared');
      expect(second).toBe('at-shared');
      expect(mockedRequestRefresh).toHaveBeenCalledTimes(1);
    });

    it('uses the single-step admin refresh when the stored session role is admin', async () => {
      localStorage.setItem('iviss_refresh_token', 'rt-admin');
      localStorage.setItem('iviss_session', JSON.stringify({ user: { role: 'admin' } }));
      mockedRequestRefresh.mockResolvedValue({
        data: { accessToken: 'admin-at' },
        error: undefined,
      } as any);

      const result = await performTokenRefresh();

      expect(result).toBe('admin-at');
      expect(mockedRequestRefresh).toHaveBeenCalledWith(
        expect.objectContaining({ body: { refreshToken: 'rt-admin' } })
      );
      expect(mockedVerifyRefresh).not.toHaveBeenCalled();
      expect(mockedGetDeviceId).not.toHaveBeenCalled();
    });
  });

  describe('setupAuthInterceptors', () => {
    function createFakeClient() {
      let requestFn!: (req: Request) => Promise<Request> | Request;
      let responseFn!: (res: Response, req: Request) => Promise<Response> | Response;
      const requestUse = vi.fn((fn: typeof requestFn) => (requestFn = fn));
      const responseUse = vi.fn((fn: typeof responseFn) => (responseFn = fn));
      const client = {
        interceptors: {
          request: { use: requestUse },
          response: { use: responseUse },
        },
      };
      return {
        client,
        requestUse,
        responseUse,
        request: (req: Request) => requestFn(req),
        respond: (res: Response, req: Request) => responseFn(res, req),
      };
    }

    it('registers both a request and a response interceptor', () => {
      const { client, requestUse, responseUse } = createFakeClient();
      setupAuthInterceptors(client, { baseUrl: '', onSessionExpired: vi.fn() });
      expect(requestUse).toHaveBeenCalledOnce();
      expect(responseUse).toHaveBeenCalledOnce();
    });

    describe('request interceptor', () => {
      it('attaches the current access token as a Bearer header', async () => {
        const { client, request } = createFakeClient();
        setupAuthInterceptors(client, { baseUrl: '', onSessionExpired: vi.fn() });
        localStorage.setItem('iviss_access_token', 'my-access-token');

        const result = await request(new Request('http://x/api/test'));

        expect(result.headers.get('Authorization')).toBe('Bearer my-access-token');
      });

      it('preserves an explicit Authorization header provided by the caller', async () => {
        const { client, request } = createFakeClient();
        setupAuthInterceptors(client, { baseUrl: '', onSessionExpired: vi.fn() });
        localStorage.setItem('iviss_access_token', 'stale-access-token');

        const result = await request(
          new Request('http://x/api/test', {
            headers: { Authorization: 'Bearer caller-provided' },
          })
        );

        expect(result.headers.get('Authorization')).toBe('Bearer caller-provided');
      });

      it('leaves headers untouched when no access token is stored', async () => {
        const { client, request } = createFakeClient();
        setupAuthInterceptors(client, { baseUrl: '', onSessionExpired: vi.fn() });

        const result = await request(new Request('http://x/api/test'));

        expect(result.headers.get('Authorization')).toBeNull();
      });
    });

    it('passes non-401 responses through unchanged', async () => {
      const { client, respond } = createFakeClient();
      const onSessionExpired = vi.fn();
      setupAuthInterceptors(client, { baseUrl: '', onSessionExpired });

      const req = new Request('http://x/api/test');
      const res = new Response('OK', { status: 200 });
      const out = await respond(res, req);

      expect(out).toBe(res);
      expect(onSessionExpired).not.toHaveBeenCalled();
    });

    it('returns the original 401 without logging out when the refresh flow fails with an unknown error (network / backend down)', async () => {
      const { client, request, respond } = createFakeClient();
      const onSessionExpired = vi.fn();
      setupAuthInterceptors(client, { baseUrl: '', onSessionExpired });

      localStorage.setItem('iviss_access_token', 'old-at');
      localStorage.setItem('iviss_refresh_token', 'rt-1');
      mockedGetDeviceId.mockResolvedValue('device-1');
      mockedRequestRefresh.mockResolvedValue({
        data: undefined,
        error: { message: 'Server Error' },
      } as any);

      const req = await request(new Request('http://x/api/test'));
      const res = new Response('Unauthorized', { status: 401 });

      const out = await respond(res, req);

      expect(out).toBe(res);
      expect(onSessionExpired).not.toHaveBeenCalled();
    });

    it('retries a POST request with a body after a successful refresh, without a "body already used" error', async () => {
      const { client, request, respond } = createFakeClient();
      const onSessionExpired = vi.fn();
      setupAuthInterceptors(client, { baseUrl: '', onSessionExpired });

      localStorage.setItem('iviss_access_token', 'old-at');
      localStorage.setItem('iviss_refresh_token', 'rt-1');

      const original = new Request('http://x/api/v1/controls', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ plate: 'ABC-123' }),
      });
      const finalRequest = await request(original);
      expect(finalRequest.headers.get('Authorization')).toBe('Bearer old-at');

      mockSuccessfulChallenge('new-at');
      const fetchSpy = vi.fn().mockResolvedValue(new Response(null, { status: 200 }));
      vi.stubGlobal('fetch', fetchSpy);

      const res401 = new Response(
        JSON.stringify({ code: 'UNAUTHORIZED', message: 'Token expired' }),
        {
          status: 401,
          headers: { 'Content-Type': 'application/json' },
        }
      );

      const finalResponse = await respond(res401, finalRequest);

      expect(finalResponse.status).toBe(200);
      expect(fetchSpy).toHaveBeenCalledTimes(1);
      const retriedRequest = fetchSpy.mock.calls[0][0] as Request;
      expect(retriedRequest.headers.get('Authorization')).toBe('Bearer new-at');
      expect(retriedRequest.headers.get(RETRY_HEADER)).toBe('1');
      await expect(retriedRequest.clone().json()).resolves.toEqual({ plate: 'ABC-123' });
      expect(onSessionExpired).not.toHaveBeenCalled();

      vi.unstubAllGlobals();
    });

    it('surfaces the original 401 instead of throwing when no stashed clone exists for the request', async () => {
      const { client, respond } = createFakeClient();
      setupAuthInterceptors(client, { baseUrl: '', onSessionExpired: vi.fn() });

      localStorage.setItem('iviss_access_token', 'old-at');
      localStorage.setItem('iviss_refresh_token', 'rt-1');
      mockSuccessfulChallenge('new-at');

      // A request that never went through the request interceptor, so it has
      // no entry in the retry WeakMap.
      const untrackedRequest = new Request('http://x/api/v1/controls', {
        method: 'POST',
        body: '{}',
      });
      const res401 = new Response(JSON.stringify({ message: 'Token expired' }), {
        status: 401,
        headers: { 'Content-Type': 'application/json' },
      });

      const finalResponse = await respond(res401, untrackedRequest);

      expect(finalResponse.status).toBe(401);
    });

    it('does not log out on a raced/expired refresh nonce (false-positive regression)', async () => {
      const { client, respond } = createFakeClient();
      const onSessionExpired = vi.fn();
      setupAuthInterceptors(client, { baseUrl: '', onSessionExpired });

      localStorage.setItem('iviss_access_token', 'old-at');

      const req = new Request('http://x/api/v1/auth/refresh/verify', { method: 'POST' });
      const res = new Response(
        JSON.stringify({
          code: 'UNAUTHORIZED',
          message: 'Nonce expired or not found — request a new challenge',
        }),
        { status: 401, headers: { 'Content-Type': 'application/json' } }
      );

      await respond(res, req);

      expect(onSessionExpired).not.toHaveBeenCalled();
    });

    it('logs out when the refresh token itself is rejected by the refresh endpoint', async () => {
      const { client, respond } = createFakeClient();
      const onSessionExpired = vi.fn();
      setupAuthInterceptors(client, { baseUrl: '', onSessionExpired });

      localStorage.setItem('iviss_access_token', 'old-at');

      const req = new Request('http://x/api/v1/auth/refresh', { method: 'POST' });
      const res = new Response(
        JSON.stringify({ code: 'UNAUTHORIZED', message: 'Invalid or expired refresh token' }),
        { status: 401, headers: { 'Content-Type': 'application/json' } }
      );

      await respond(res, req);

      expect(onSessionExpired).toHaveBeenCalledTimes(1);
    });

    it('marks device reactivation and logs out when the device must be reactivated', async () => {
      const { client, respond } = createFakeClient();
      const onSessionExpired = vi.fn();
      setupAuthInterceptors(client, { baseUrl: '', onSessionExpired });

      localStorage.setItem('iviss_access_token', 'old-at');
      localStorage.setItem('iviss_device_activated', 'true');

      const req = new Request('http://x/api/v1/auth/refresh/verify', { method: 'POST' });
      const res = new Response(
        JSON.stringify({ code: 'UNAUTHORIZED', message: 'Device not found or revoked' }),
        { status: 401, headers: { 'Content-Type': 'application/json' } }
      );

      await respond(res, req);

      expect(onSessionExpired).toHaveBeenCalledTimes(1);
      expect(localStorage.getItem('iviss_device_activated')).toBeNull();
      expect(localStorage.getItem('iviss_forced_logout_reason')).toBe(
        'DEVICE_REACTIVATION_REQUIRED'
      );
    });

    it('does not attempt a refresh twice for the same request (retry-header guard)', async () => {
      const { client, respond } = createFakeClient();
      setupAuthInterceptors(client, { baseUrl: '', onSessionExpired: vi.fn() });

      localStorage.setItem('iviss_access_token', 'old-at');
      localStorage.setItem('iviss_refresh_token', 'rt-1');

      const alreadyRetried = new Request('http://x/api/v1/controls', {
        headers: { [RETRY_HEADER]: '1' },
      });
      const res401 = new Response(null, { status: 401 });

      const finalResponse = await respond(res401, alreadyRetried);

      expect(finalResponse.status).toBe(401);
      expect(mockedRequestRefresh).not.toHaveBeenCalled();
    });
  });
});
