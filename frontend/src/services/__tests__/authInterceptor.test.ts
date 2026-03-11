import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { setupAuthInterceptors } from '../auth/authInterceptor';
import * as tokenManager from '../auth/tokenManager';

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

      // Mock the refresh endpoint
      const fetchSpy = vi.spyOn(globalThis, 'fetch');

      // First call: POST /auth/refresh -> returns nonce
      fetchSpy.mockResolvedValueOnce(
        new Response(JSON.stringify({ nonce: 'backend-nonce-123' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      );

      // Second call: POST /auth/refresh/verify -> returns new token
      fetchSpy.mockResolvedValueOnce(
        new Response(JSON.stringify({ accessToken: 'new-access-token' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      );

      // Third call: retry of original request
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
    });

    it('should return original 401 response when refresh endpoint fails', async () => {
      vi.mocked(tokenManager.getRefreshToken).mockReturnValue('my-refresh-token');

      const fetchSpy = vi.spyOn(globalThis, 'fetch');
      fetchSpy.mockResolvedValueOnce(new Response('Server Error', { status: 500 }));

      const request = new Request('http://localhost:3000/api/test');
      const response = new Response('Unauthorized', { status: 401 });

      const interceptor = mockClient._responseInterceptors[0];
      const result = await interceptor(response, request);
      expect(result).toBe(response);
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

      const fetchSpy = vi.spyOn(globalThis, 'fetch');

      // Mock implementation to track precisely what is called
      fetchSpy.mockImplementation(async (req) => {
        const url = typeof req === 'string' ? req : (req as Request).url;

        if (url.includes('/auth/refresh/verify')) {
          return new Response(JSON.stringify({ accessToken: 'shared-new-token' }), {
            status: 200,
            headers: { 'Content-Type': 'application/json' },
          });
        }

        if (url.includes('/auth/refresh')) {
          // Add delay to ensure concurrency
          await new Promise((resolve) => setTimeout(resolve, 50));
          return new Response(JSON.stringify({ nonce: 'shared-nonce' }), {
            status: 200,
            headers: { 'Content-Type': 'application/json' },
          });
        }

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

      // Verify refresh calls: /auth/refresh should be called ONCE, /auth/refresh/verify ONCE
      const refreshCalls = fetchSpy.mock.calls.filter((call) => {
        const url = typeof call[0] === 'string' ? call[0] : (call[0] as Request).url;
        return url.includes('/auth/refresh');
      });

      // Total of 2 calls for refresh flow (1 init, 1 verify)
      // If queueing works, it's not 4 calls (2 init, 2 verify)
      expect(refreshCalls).toHaveLength(2);

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
