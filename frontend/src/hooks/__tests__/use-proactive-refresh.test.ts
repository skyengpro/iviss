import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import { focusManager } from '@tanstack/react-query';

vi.mock('@/services/auth/authInterceptor', () => ({
  performTokenRefresh: vi.fn(),
}));

import { performTokenRefresh } from '@/services/auth/authInterceptor';
import { useProactiveRefresh } from '../auth/use-proactive-refresh';

const mockedPerformRefresh = vi.mocked(performTokenRefresh);

/**
 * Build a JWT-shaped string whose payload has the given `exp` (seconds
 * since epoch). The header and signature segments are meaningless — the
 * hook only base64-decodes the payload to read `exp`.
 */
function makeJwt(expSecs: number): string {
  const header = btoa(JSON.stringify({ alg: 'ES256' }))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
  const payload = btoa(JSON.stringify({ exp: expSecs }))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
  return `${header}.${payload}.sig`;
}

function dispatchVisibility(state: 'visible' | 'hidden') {
  Object.defineProperty(document, 'visibilityState', {
    configurable: true,
    get: () => state,
  });
  // The hook attaches its listener to `window`. In real browsers a
  // `visibilitychange` fired on `document` reaches window listeners; JSDOM
  // does not propagate it, so dispatch directly on `window` to mirror
  // real-browser behavior.
  window.dispatchEvent(new Event('visibilitychange'));
}

/**
 * The hook calls `setFocused()` on visibilitychange, but does so *after*
 * awaiting the refresh. Flush pending microtasks and then a macrotask so
 * every awaited step (localStorage lookups, performTokenRefresh, then
 * setFocused) has had a chance to run before the test assertions.
 */
async function flushAsync() {
  await Promise.resolve();
  await new Promise((resolve) => setTimeout(resolve, 0));
}

describe('useProactiveRefresh', () => {
  const nowSecs = () => Math.floor(Date.now() / 1000);

  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    mockedPerformRefresh.mockResolvedValue('new-at');
  });

  afterEach(() => {
    // Restore TanStack's default focus listener between tests to avoid
    // leaking custom listeners across the suite.
    focusManager.setEventListener((setFocused) => {
      const listener = () => setFocused();
      window.addEventListener('visibilitychange', listener, false);
      return () => window.removeEventListener('visibilitychange', listener);
    });
  });

  it('does nothing when isAuthenticated is false', () => {
    const setEventListenerSpy = vi.spyOn(focusManager, 'setEventListener');

    renderHook(() => useProactiveRefresh(false));

    expect(setEventListenerSpy).not.toHaveBeenCalled();
  });

  it('installs a custom focus listener when isAuthenticated is true', () => {
    const setEventListenerSpy = vi.spyOn(focusManager, 'setEventListener');

    renderHook(() => useProactiveRefresh(true));

    expect(setEventListenerSpy).toHaveBeenCalledTimes(1);
  });

  it('does not refresh when there is no access token in storage', async () => {
    renderHook(() => useProactiveRefresh(true));

    dispatchVisibility('visible');
    await flushAsync();

    expect(mockedPerformRefresh).not.toHaveBeenCalled();
  });

  it('does not refresh when the access token still has plenty of life left', async () => {
    // Token expires an hour from now, well beyond the 120s leeway — no
    // proactive refresh needed.
    localStorage.setItem('iviss_access_token', makeJwt(nowSecs() + 3600));
    localStorage.setItem('iviss_refresh_token', 'rt-1');

    renderHook(() => useProactiveRefresh(true));
    dispatchVisibility('visible');
    await flushAsync();

    expect(mockedPerformRefresh).not.toHaveBeenCalled();
  });

  it('triggers a refresh when the access token expires within the leeway window', async () => {
    // Token expires in 30s — well inside the 120s leeway.
    localStorage.setItem('iviss_access_token', makeJwt(nowSecs() + 30));
    localStorage.setItem('iviss_refresh_token', 'rt-1');

    renderHook(() => useProactiveRefresh(true));
    dispatchVisibility('visible');
    await flushAsync();

    expect(mockedPerformRefresh).toHaveBeenCalledTimes(1);
  });

  it('triggers a refresh when the access token is already expired', async () => {
    localStorage.setItem('iviss_access_token', makeJwt(nowSecs() - 60));
    localStorage.setItem('iviss_refresh_token', 'rt-1');

    renderHook(() => useProactiveRefresh(true));
    dispatchVisibility('visible');
    await flushAsync();

    expect(mockedPerformRefresh).toHaveBeenCalledTimes(1);
  });

  it('does not refresh on a hidden → visible-check when the page went hidden (guard against feedback)', async () => {
    localStorage.setItem('iviss_access_token', makeJwt(nowSecs() - 60));
    localStorage.setItem('iviss_refresh_token', 'rt-1');

    renderHook(() => useProactiveRefresh(true));
    dispatchVisibility('hidden');
    await flushAsync();

    expect(mockedPerformRefresh).not.toHaveBeenCalled();
  });

  it('skips refresh when the refresh token is the literal string "null" left by a legacy bug', async () => {
    localStorage.setItem('iviss_access_token', makeJwt(nowSecs() - 60));
    localStorage.setItem('iviss_refresh_token', 'null');

    renderHook(() => useProactiveRefresh(true));
    dispatchVisibility('visible');
    await flushAsync();

    expect(mockedPerformRefresh).not.toHaveBeenCalled();
  });

  it('awaits the refresh before notifying TanStack that focus regained (ordering guarantee)', async () => {
    localStorage.setItem('iviss_access_token', makeJwt(nowSecs() - 60));
    localStorage.setItem('iviss_refresh_token', 'rt-1');

    // Capture the listener-setup callback the hook installs on the
    // focusManager so we can drive it with our own `setFocused` and observe
    // the exact ordering: refresh must resolve BEFORE setFocused fires,
    // otherwise TanStack's refetches would race the refresh — the exact
    // regression this hook exists to prevent.
    type SetupFn = Parameters<typeof focusManager.setEventListener>[0];
    let capturedSetup: SetupFn | null = null;
    const setEventListenerSpy = vi
      .spyOn(focusManager, 'setEventListener')
      .mockImplementation((setup: SetupFn) => {
        capturedSetup = setup;
      });

    const order: string[] = [];
    mockedPerformRefresh.mockImplementation(async () => {
      await new Promise((resolve) => setTimeout(resolve, 20));
      order.push('refresh');
      return 'new-at';
    });

    renderHook(() => useProactiveRefresh(true));
    expect(capturedSetup).not.toBeNull();

    const setFocusedMock = vi.fn(() => order.push('focus-notified'));
    const cleanup = capturedSetup!(setFocusedMock);

    dispatchVisibility('visible');
    await new Promise((resolve) => setTimeout(resolve, 50));

    expect(order).toEqual(['refresh', 'focus-notified']);
    if (typeof cleanup === 'function') cleanup();
    setEventListenerSpy.mockRestore();
  });
});
