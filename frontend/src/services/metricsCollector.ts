/**
 * Frontend Metrics Collector
 *
 * Browser-side module that collects Web Vitals, navigation timing,
 * error counts, and session heartbeats. Periodically pushes them
 * to the metrics server via POST /api/metrics.
 */

const METRICS_URL = import.meta.env.VITE_METRICS_URL || 'http://localhost:9091/api/metrics';
const METRICS_ENABLED = import.meta.env.VITE_METRICS_ENABLED !== 'false';
const PUSH_INTERVAL_MS = 10_000; // 10 seconds

// ————————————————————————————————————————
// State
// ————————————————————————————————————————

let sessionId: string = '';
let routeNavigationCount = 0;
let errorCount = 0;
let lcpValue: number | null = null;
let fidValue: number | null = null;
let clsValue: number | null = null;
let pageLoadDuration: number | null = null;
let intervalId: ReturnType<typeof setInterval> | null = null;
let initialized = false;

// ————————————————————————————————————————
// Generate session ID
// ————————————————————————————————————————

function generateSessionId(): string {
  return `${Date.now()}-${Math.random().toString(36).substring(2, 10)}`;
}

// ————————————————————————————————————————
// Web Vitals Observers
// ————————————————————————————————————————

function observeLCP(): void {
  try {
    const observer = new PerformanceObserver((list) => {
      const entries = list.getEntries();
      const lastEntry = entries[entries.length - 1];
      if (lastEntry) {
        lcpValue = lastEntry.startTime;
      }
    });
    observer.observe({ type: 'largest-contentful-paint', buffered: true });
  } catch {
    // PerformanceObserver not supported
  }
}

function observeFID(): void {
  try {
    const observer = new PerformanceObserver((list) => {
      const entries = list.getEntries();
      const firstEntry = entries[0] as PerformanceEventTiming | undefined;
      if (firstEntry) {
        fidValue = firstEntry.processingStart - firstEntry.startTime;
      }
    });
    observer.observe({ type: 'first-input', buffered: true });
  } catch {
    // PerformanceObserver not supported
  }
}

function observeCLS(): void {
  try {
    let clsScore = 0;
    const observer = new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        const layoutShift = entry as PerformanceEntry & {
          hadRecentInput?: boolean;
          value?: number;
        };
        if (!layoutShift.hadRecentInput && layoutShift.value) {
          clsScore += layoutShift.value;
          clsValue = clsScore;
        }
      }
    });
    observer.observe({ type: 'layout-shift', buffered: true });
  } catch {
    // PerformanceObserver not supported
  }
}

function collectPageLoadDuration(): void {
  try {
    // Use Navigation Timing API Level 2
    const [navEntry] = performance.getEntriesByType('navigation') as PerformanceNavigationTiming[];
    if (navEntry) {
      pageLoadDuration = navEntry.loadEventEnd - navEntry.startTime;
    }
  } catch {
    // Navigation timing not available
  }
}

// ————————————————————————————————————————
// Error Tracking
// ————————————————————————————————————————

function setupErrorTracking(): void {
  window.addEventListener('error', () => {
    errorCount++;
  });
  window.addEventListener('unhandledrejection', () => {
    errorCount++;
  });
}

// ————————————————————————————————————————
// Push Metrics to Server
// ————————————————————————————————————————

async function pushMetrics(): Promise<void> {
  // Collect page load duration on first push if not yet collected
  if (pageLoadDuration === null) {
    collectPageLoadDuration();
  }

  const payload = {
    sessionId,
    metrics: {
      heartbeat: true,
      pageLoadDuration: pageLoadDuration,
      lcp: lcpValue,
      fid: fidValue,
      cls: clsValue,
      routeNavigations: routeNavigationCount,
      errors: errorCount,
    },
  };

  // Reset counters after sending (they're cumulative on the server)
  routeNavigationCount = 0;
  errorCount = 0;

  try {
    await fetch(METRICS_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
      // Don't block page unload for metrics
      keepalive: true,
    });
  } catch {
    // Silently fail — monitoring should not affect user experience
  }
}

// ————————————————————————————————————————
// Public API
// ————————————————————————————————————————

/**
 * Initialize the metrics collector.
 * Safe to call multiple times — only initializes once.
 */
export function initMetrics(): void {
  if (!METRICS_ENABLED || initialized) return;

  initialized = true;
  sessionId = generateSessionId();

  // Set up Web Vitals observers
  observeLCP();
  observeFID();
  observeCLS();

  // Set up error tracking
  setupErrorTracking();

  // Collect page load duration after window load
  if (document.readyState === 'complete') {
    collectPageLoadDuration();
  } else {
    window.addEventListener('load', () => {
      // Wait a tick for loadEventEnd to be populated
      setTimeout(collectPageLoadDuration, 0);
    });
  }

  // Push metrics on interval
  intervalId = setInterval(pushMetrics, PUSH_INTERVAL_MS);

  // Push on first load (after short delay to gather initial data)
  setTimeout(pushMetrics, 2000);

  // Push final metrics before page unload
  window.addEventListener('beforeunload', () => {
    pushMetrics();
  });

  console.log('[iviss-metrics] Metrics collector initialized', {
    sessionId,
    metricsUrl: METRICS_URL,
    pushInterval: `${PUSH_INTERVAL_MS / 1000}s`,
  });
}

/**
 * Record a client-side route navigation event.
 */
export function recordNavigation(): void {
  if (!METRICS_ENABLED) return;
  routeNavigationCount++;
}

/**
 * Stop the metrics collector and clean up resources.
 */
export function destroyMetrics(): void {
  if (intervalId) {
    clearInterval(intervalId);
    intervalId = null;
  }
  initialized = false;
}
