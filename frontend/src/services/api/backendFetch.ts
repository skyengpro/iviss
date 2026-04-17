import { getAccessToken } from '../auth/tokenManager';

function getBaseUrl(): string {
  const apiBaseUrl = import.meta.env.VITE_API_URL || '/api';
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

  if (!response.ok) {
    let isSessionRevoked = false;
    if (response.status === 401 && token) {
      // Only treat as session revoked if we had a token — a login failure is not a revocation
      isSessionRevoked = true;
    } else {
      try {
        const cloned = response.clone();
        const body = await cloned.json();
        if (body && typeof body === 'object' && body.code === 'SESSION_REVOKED') {
          isSessionRevoked = true;
        }
      } catch {
        // Not JSON or other error
      }
    }

    if (isSessionRevoked) {
      window.dispatchEvent(new CustomEvent('iviss:session-revoked'));
    }
  }

  return response;
}
