import { renderHook } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useMetrics } from '../useMetrics';
import { initMetrics, recordNavigation, destroyMetrics } from '@/services/metrics/metricsCollector';
import { useLocation } from 'react-router-dom';

vi.mock('@/services/metrics/metricsCollector', () => ({
  initMetrics: vi.fn(),
  recordNavigation: vi.fn(),
  destroyMetrics: vi.fn(),
}));

vi.mock('react-router-dom', () => ({
  useLocation: vi.fn(),
}));

describe('useMetrics', () => {
  let mockLocation: any;

  beforeEach(() => {
    vi.clearAllMocks();

    mockLocation = { pathname: '/home' };
    vi.mocked(useLocation).mockImplementation(() => mockLocation);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('initMetrics is called on mount', () => {
    renderHook(() => useMetrics());

    expect(initMetrics).toHaveBeenCalledTimes(1);
  });

  it('destroyMetrics is called on unmount', () => {
    const { unmount } = renderHook(() => useMetrics());

    expect(destroyMetrics).not.toHaveBeenCalled();

    unmount();

    expect(destroyMetrics).toHaveBeenCalledTimes(1);
  });

  it('recordNavigation is NOT called on the initial render', () => {
    renderHook(() => useMetrics());

    expect(recordNavigation).not.toHaveBeenCalled();
  });

  it('recordNavigation IS called when location.pathname changes', () => {
    const { rerender } = renderHook(() => useMetrics());

    expect(recordNavigation).not.toHaveBeenCalled();

    // Change pathname
    mockLocation = { pathname: '/about' };
    vi.mocked(useLocation).mockImplementation(() => mockLocation);

    rerender();

    expect(recordNavigation).toHaveBeenCalledTimes(1);

    // Change to a new pathname
    mockLocation = { pathname: '/contact' };
    vi.mocked(useLocation).mockImplementation(() => mockLocation);

    rerender();

    expect(recordNavigation).toHaveBeenCalledTimes(2);
  });

  it('recordNavigation is not called if location changes but pathname remains the same', () => {
    const { rerender } = renderHook(() => useMetrics());

    // Same pathname, different search params
    mockLocation = { pathname: '/home', search: '?q=test' };
    vi.mocked(useLocation).mockImplementation(() => mockLocation);

    rerender();

    expect(recordNavigation).not.toHaveBeenCalled();
  });
});
