import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// We need to reset module state between tests, so we use dynamic imports
// and resetModules to get fresh state for each test.
describe('metricsCollector', () => {
  let initMetrics: () => void;
  let recordNavigation: () => void;
  let destroyMetrics: () => void;

  beforeEach(async () => {
    vi.resetModules();

    // Stub import.meta.env values
    vi.stubEnv('VITE_METRICS_ENABLED', 'true');
    vi.stubEnv('VITE_METRICS_URL', 'http://localhost:9091/api/metrics');

    // Mock PerformanceObserver
    vi.stubGlobal(
      'PerformanceObserver',
      vi.fn().mockImplementation(() => ({
        observe: vi.fn(),
        disconnect: vi.fn(),
      }))
    );

    // Mock fetch
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('ok')));

    // Use fake timers
    vi.useFakeTimers();

    const mod = await import('../metrics/metricsCollector');
    initMetrics = mod.initMetrics;
    recordNavigation = mod.recordNavigation;
    destroyMetrics = mod.destroyMetrics;
  });

  afterEach(() => {
    destroyMetrics();
    vi.useRealTimers();
    vi.unstubAllEnvs();
    vi.unstubAllGlobals();
  });

  it('should initialize only once (idempotent)', () => {
    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    initMetrics();
    initMetrics(); // second call should be a no-op

    // console.log is called once during init
    expect(consoleSpy).toHaveBeenCalledTimes(1);
    consoleSpy.mockRestore();
  });

  it('should allow recording navigation events', () => {
    // recordNavigation should not throw even before init
    expect(() => recordNavigation()).not.toThrow();
  });

  it('destroyMetrics should reset state and allow re-initialization', () => {
    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    initMetrics();
    destroyMetrics();
    initMetrics(); // should work again after destroy

    expect(consoleSpy).toHaveBeenCalledTimes(2);
    consoleSpy.mockRestore();
  });
});
