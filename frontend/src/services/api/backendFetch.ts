import { getAccessToken } from '../auth/tokenManager';

function getBaseUrl(): string {
  const apiBaseUrl = import.meta.env.VITE_API_URL || '';
  return apiBaseUrl.replace(/\/+$/, '');
}

function isBackendUrl(url: string): boolean {
  return url.startsWith(getBaseUrl());
}

export async function fetchWithAuth(input: string, init?: RequestInit): Promise<Response> {
  const url = input.startsWith('http')
    ? input
    : `${getBaseUrl()}${input.startsWith('/') ? '' : '/'}${input}`;

  const token = getAccessToken() || undefined;
  const headers = new Headers(init?.headers);

  if (token && isBackendUrl(url) && !headers.has('Authorization')) {
    headers.set('Authorization', `Bearer ${token}`);
  }

  const response = await fetch(url, {
    ...init,
    headers,
  });

  // Only check for explicit SESSION_REVOKED code, not generic 401s
  // Generic 401s (token expired) are handled by the auth interceptor which refreshes tokens
  if (!response.ok && response.status !== 401) {
    try {
      const cloned = response.clone();
      const body = await cloned.json();
      if (body && typeof body === 'object' && body.code === 'SESSION_REVOKED') {
        window.dispatchEvent(new CustomEvent('iviss:session-revoked'));
      }
    } catch {
      // Not JSON or other error - ignore
    }
  }

  return response;
}
