const SESSION_KEY = 'iviss_session';

function getBaseUrl(): string {
  return (import.meta.env.VITE_API_URL || 'http://localhost:3000').replace(/\/+$/, '');
}

function getAccessTokenFromStorage(): string | undefined {
  try {
    const raw = localStorage.getItem(SESSION_KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw) as { token?: unknown };
    return typeof parsed.token === 'string' ? parsed.token : undefined;
  } catch {
    return;
  }
}

function isBackendUrl(url: string): boolean {
  return url.startsWith(getBaseUrl());
}

export async function fetchWithAuth(input: string, init?: RequestInit): Promise<Response> {
  const url = input.startsWith('http')
    ? input
    : `${getBaseUrl()}${input.startsWith('/') ? '' : '/'}${input}`;

  const token = getAccessTokenFromStorage();
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
    if (response.status === 401) {
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
