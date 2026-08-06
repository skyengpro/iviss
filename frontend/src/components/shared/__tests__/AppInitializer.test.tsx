import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render } from '@testing-library/react';

vi.mock('@/services/keyManagement/keyManagement', () => ({
  KeyManagement: vi.fn(),
}));

import { AppInitializer } from '../AppInitializer';
import { KeyManagement } from '@/services/keyManagement/keyManagement';

const mockedKeyManagement = vi.mocked(KeyManagement);

function dispatchVisibility(state: 'visible' | 'hidden') {
  Object.defineProperty(document, 'visibilityState', {
    configurable: true,
    get: () => state,
  });
  document.dispatchEvent(new Event('visibilitychange'));
}

/** Flush the microtask queue several times so chained awaits resolve. */
async function flushMicrotasks(rounds = 5) {
  for (let i = 0; i < rounds; i++) {
    await Promise.resolve();
  }
}

describe('AppInitializer', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it('renders children even when key initialization fails on every attempt (never blocks the app)', async () => {
    mockedKeyManagement.mockRejectedValue(new Error('IndexedDB unavailable'));

    const { getByText } = render(
      <AppInitializer>
        <div>app-content</div>
      </AppInitializer>
    );

    expect(getByText('app-content')).toBeTruthy();

    // Drain retry backoff (setTimeouts between attempts).
    await vi.runAllTimersAsync();
    await flushMicrotasks();

    expect(mockedKeyManagement).toHaveBeenCalledTimes(3);
  });

  it('stops retrying as soon as an attempt succeeds', async () => {
    mockedKeyManagement
      .mockRejectedValueOnce(new Error('transient IDB'))
      .mockResolvedValueOnce({ publicKey: {}, privateKey: {} } as never);

    render(
      <AppInitializer>
        <div>ok</div>
      </AppInitializer>
    );

    await vi.runAllTimersAsync();
    await flushMicrotasks();

    expect(mockedKeyManagement).toHaveBeenCalledTimes(2);
  });

  it('re-attempts initialization on the next visibilitychange to visible after exhausting startup retries', async () => {
    mockedKeyManagement
      .mockRejectedValueOnce(new Error('fail-1'))
      .mockRejectedValueOnce(new Error('fail-2'))
      .mockRejectedValueOnce(new Error('fail-3'))
      .mockResolvedValueOnce({ publicKey: {}, privateKey: {} } as never);

    render(
      <AppInitializer>
        <div>ok</div>
      </AppInitializer>
    );

    await vi.runAllTimersAsync();
    await flushMicrotasks();
    expect(mockedKeyManagement).toHaveBeenCalledTimes(3);

    dispatchVisibility('visible');
    await flushMicrotasks();

    expect(mockedKeyManagement).toHaveBeenCalledTimes(4);
  });

  it('does not re-attempt on visibility changes to hidden', async () => {
    mockedKeyManagement.mockRejectedValue(new Error('always-fail'));

    render(
      <AppInitializer>
        <div>ok</div>
      </AppInitializer>
    );
    await vi.runAllTimersAsync();
    await flushMicrotasks();
    expect(mockedKeyManagement).toHaveBeenCalledTimes(3);

    dispatchVisibility('hidden');
    await flushMicrotasks();

    expect(mockedKeyManagement).toHaveBeenCalledTimes(3);
  });
});
