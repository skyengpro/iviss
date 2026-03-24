import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { setupAuthInterceptors } from '../auth/authInterceptor';
import * as tokenManager from '../auth/tokenManager';

vi.mock('@/openapi-rq/requests/services.gen', () => ({
  requestRefresh: vi.fn(),
  verifyRefresh: vi.fn(),
}));

import { requestRefresh, verifyRefresh } from '@/openapi-rq/requests/services.gen';

type RequestRefreshResult = Awaited<ReturnType<typeof requestRefresh>>;
type VerifyRefreshResult = Awaited<ReturnType<typeof verifyRefresh>>;

// Mock dependencies
vi.mock('../auth/tokenManager', () => ({
  getAccessToken: vi.fn(),
  getRefreshToken: vi.fn(),
  setAccessToken: vi.fn(),
  setRefreshToken: vi.fn(),
  clearTokens: vi.fn(),
}));

vi.mock('../device/deviceId', () => ({
  getDeviceId: vi.fn().mockResolvedValue('test-device-id'),
}));

vi.mock('../auth/signatureService', () => ({
  signNonce: vi.fn().mockResolvedValue('signed-nonce-jws'),
}));

// Helper: create a mock hey-api client
function createMockClient() {
  const requestInterceptors: Array<(req: Request) => Promise<Request> | Request> = [];
  const responseInterceptors: Array<(res: Response, req: Request) => Promise<Response> | Response> =
    [];

  return {
    interceptors: {
      request: {
        use: vi.fn((fn: (req: Request) => Promise<Request> | Request) => {
          requestInterceptors.push(fn);
        }),
      },
      response: {
        use: vi.fn((fn: (res: Response, req: Request) => Promise<Response> | Response) => {
          responseInterceptors.push(fn);
        }),
      },
    },
    // Expose for test invocation
    _requestInterceptors: requestInterceptors,
    _responseInterceptors: responseInterceptors,
  };
}

describe('authInterceptor', () => {
  let mockClient: ReturnType<typeof createMockClient>;
  const baseUrl = 'http://localhost:3000';
  let onSessionExpired: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.clearAllMocks();
    mockClient = createMockClient();
    onSessionExpired = vi.fn();

    setupAuthInterceptors(mockClient, {
      baseUrl,
      onSessionExpired,
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('setup', () => {
    it('should register both request and response interceptors', () => {
      expect(mockClient.interceptors.request.use).toHaveBeenCalledOnce();
      expect(mockClient.interceptors.response.use).toHaveBeenCalledOnce();
    });
  });

  describe('request interceptor', () => {
    it('should attach Authorization header when token exists', async () => {
      vi.mocked(tokenManager.getAccessToken).mockReturnValue('my-access-token');

      const request = new Request('http://localhost:3000/api/test');
      const interceptor = mockClient._requestInterceptors[0];
      const result = await interceptor(request);

      expect(result.headers.get('Authorization')).toBe('Bearer my-access-token');
    });

    it('should preserve an explicit Authorization header from the caller', async () => {
      vi.mocked(tokenManager.getAccessToken).mockReturnValue('stale-access-token');

      const request = new Request('http://localhost:3000/api/test', {
        headers: {
          Authorization: 'Bearer fresh-access-token',
        },
      });
      const interceptor = mockClient._requestInterceptors[0];
      const result = await interceptor(request);

      expect(result.headers.get('Authorization')).toBe('Bearer fresh-access-token');
    });

    it('should not modify headers when no token exists', async () => {
      vi.mocked(tokenManager.getAccessToken).mockReturnValue(null);

      const request = new Request('http://localhost:3000/api/test');
      const interceptor = mockClient._requestInterceptors[0];
      const result = await interceptor(request);

      expect(result.headers.get('Authorization')).toBeNull();
    });
  });

  describe('response interceptor', () => {
    it('should pass through non-401 responses unchanged', async () => {
      const request = new Request('http://localhost:3000/api/test');
      const response = new Response('OK', { status: 200 });

      const interceptor = mockClient._responseInterceptors[0];
      const result = await interceptor(response, request);

      expect(result).toBe(response);
      expect(onSessionExpired).not.toHaveBeenCalled();
    });

    it('should attempt refresh on 401 response', async () => {
      vi.mocked(tokenManager.getRefreshToken).mockReturnValue('my-refresh-token');

      vi.mocked(requestRefresh).mockResolvedValueOnce({
        data: { nonce: 'backend-nonce-123' },
        error: undefined,
      } as RequestRefreshResult);

      vi.mocked(verifyRefresh).mockResolvedValueOnce({
        data: { accessToken: 'new-access-token' },
        error: undefined,
      } as VerifyRefreshResult);

      // Retry of original request
      const fetchSpy = vi.spyOn(globalThis, 'fetch');
      fetchSpy.mockResolvedValueOnce(
        new Response('{"data":"success"}', {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      );

      const request = new Request('http://localhost:3000/api/test');
      const response = new Response('Unauthorized', { status: 401 });

      const interceptor = mockClient._responseInterceptors[0];
      const result = await interceptor(response, request);

      // Should have stored the new token
      expect(tokenManager.setAccessToken).toHaveBeenCalledWith('new-access-token');

      // Should have retried with the new token
      expect(result.status).toBe(200);
    });

    it('should return original 401 response when refresh token is missing', async () => {
      vi.mocked(tokenManager.getRefreshToken).mockReturnValue(null);

      const request = new Request('http://localhost:3000/api/test');
      const response = new Response('Unauthorized', { status: 401 });

      const interceptor = mockClient._responseInterceptors[0];
      const result = await interceptor(response, request);
      expect(result).toBe(response);
      expect(onSessionExpired).toHaveBeenCalledTimes(1);
    });

    it('should not attempt refresh when the 401 comes from refresh endpoints', async () => {
      vi.mocked(tokenManager.getRefreshToken).mockReturnValue('my-refresh-token');

      const interceptor = mockClient._responseInterceptors[0];

      const reqRefresh = new Request('http://localhost:3000/auth/refresh');
      const res401 = new Response('Unauthorized', { status: 401 });
      const out1 = await interceptor(res401, reqRefresh);

      expect(out1).toBe(res401);
      expect(requestRefresh).not.toHaveBeenCalled();
      expect(verifyRefresh).not.toHaveBeenCalled();
    });

    it('should return original 401 response when refresh endpoint fails', async () => {
      vi.mocked(tokenManager.getRefreshToken).mockReturnValue('my-refresh-token');

      vi.mocked(requestRefresh).mockResolvedValueOnce({
        data: undefined,
        error: { message: 'Server Error' },
      } as RequestRefreshResult);

      const request = new Request('http://localhost:3000/api/test');
      const response = new Response('Unauthorized', { status: 401 });

      const interceptor = mockClient._responseInterceptors[0];
      const result = await interceptor(response, request);
      expect(result).toBe(response);
      expect(onSessionExpired).toHaveBeenCalledTimes(1);
    });

    it('should prevent infinite retry loops on repeated 401', async () => {
      vi.mocked(tokenManager.getRefreshToken).mockReturnValue('my-refresh-token');

      // Create a request that already has the retry header
      const headers = new Headers();
      headers.set('X-Auth-Retry', '1');
      const request = new Request('http://localhost:3000/api/test', { headers });
      const response = new Response('Unauthorized', { status: 401 });

      const interceptor = mockClient._responseInterceptors[0];
      const result = await interceptor(response, request);

      // Should NOT attempt refresh
      expect(result).toBe(response);
    });

    it('should queue concurrent 401 requests and only perform one refresh flow', async () => {
      vi.mocked(tokenManager.getRefreshToken).mockReturnValue('my-refresh-token');

      vi.mocked(requestRefresh).mockImplementationOnce(async () => {
        await new Promise((resolve) => setTimeout(resolve, 50));
        return { data: { nonce: 'shared-nonce' }, error: undefined } as RequestRefreshResult;
      });

      vi.mocked(verifyRefresh).mockResolvedValueOnce({
        data: { accessToken: 'shared-new-token' },
        error: undefined,
      } as VerifyRefreshResult);

      const fetchSpy = vi.spyOn(globalThis, 'fetch');
      fetchSpy.mockImplementation(async () => {
        return new Response(JSON.stringify({ data: 'success' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        });
      });

      const interceptor = mockClient._responseInterceptors[0];

      const req1 = new Request('http://localhost:3000/api/1');
      const req2 = new Request('http://localhost:3000/api/2');
      const response401 = new Response('Unauthorized', { status: 401 });

      // Run both concurrently
      const [res1, res2] = await Promise.all([
        interceptor(response401.clone(), req1),
        interceptor(response401.clone(), req2),
      ]);

      // Verify refresh calls happen only once each
      expect(requestRefresh).toHaveBeenCalledTimes(1);
      expect(verifyRefresh).toHaveBeenCalledTimes(1);

      // Verify the original requests were retried (2 retries)
      expect(fetchSpy).toHaveBeenCalledTimes(2);

      // Verify retry calls: both should succeed with the same token
      expect(res1.status).toBe(200);
      expect(res2.status).toBe(200);

      const res1Body = await res1.json();
      const res2Body = await res2.json();
      expect(res1Body.data).toBe('success');
      expect(res2Body.data).toBe('success');
    });
  });
});
