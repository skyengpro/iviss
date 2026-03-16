import { getAccessToken } from '../auth/tokenManager';

function getBaseUrl(): string {
  return (import.meta.env.VITE_API_URL || 'http://localhost:3000').replace(/\/+$/, '');
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

  return fetch(url, {
    ...init,
    headers,
  });
}
