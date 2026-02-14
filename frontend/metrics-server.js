/**
 * IVISS Frontend Metrics Server
 *
 * Lightweight Express server that bridges browser-collected metrics
 * to Prometheus. The frontend pushes metrics via POST /api/metrics,
 * and Prometheus scrapes GET /metrics.
 */

import express from 'express';
import { Registry, Gauge, Counter, Histogram, collectDefaultMetrics } from 'prom-client';

const app = express();
const register = new Registry();

// Collect default Node.js metrics (event loop, memory, etc.)
collectDefaultMetrics({ register });

// ————————————————————————————————————————
// Frontend Metrics Definitions
// ————————————————————————————————————————

const frontendUp = new Gauge({
  name: 'frontend_up',
  help: 'Frontend availability heartbeat (1 = at least one active session)',
  registers: [register],
});

const activeSessions = new Gauge({
  name: 'frontend_active_sessions',
  help: 'Number of active browser sessions',
  registers: [register],
});

const pageLoadDuration = new Histogram({
  name: 'frontend_page_load_duration_ms',
  help: 'Page load duration from Navigation Timing API (milliseconds)',
  buckets: [100, 250, 500, 1000, 2000, 3000, 5000, 10000],
  registers: [register],
});

const lcpGauge = new Gauge({
  name: 'frontend_lcp_ms',
  help: 'Largest Contentful Paint (milliseconds)',
  registers: [register],
});

const fidGauge = new Gauge({
  name: 'frontend_fid_ms',
  help: 'First Input Delay (milliseconds)',
  registers: [register],
});

const clsGauge = new Gauge({
  name: 'frontend_cls',
  help: 'Cumulative Layout Shift score',
  registers: [register],
});

const routeNavigations = new Counter({
  name: 'frontend_route_navigations_total',
  help: 'Total client-side route navigations',
  registers: [register],
});

const errorsTotal = new Counter({
  name: 'frontend_errors_total',
  help: 'Total uncaught frontend JavaScript errors',
  registers: [register],
});

// Track heartbeat timeout per session
const sessionHeartbeats = new Map();
const HEARTBEAT_TIMEOUT_MS = 30_000; // Consider session dead after 30s without heartbeat

// ————————————————————————————————————————
// Middleware
// ————————————————————————————————————————

app.use(express.json());

// CORS — allow requests from any origin (frontend dev servers)
app.use((_req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');
  res.header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.header('Access-Control-Allow-Headers', 'Content-Type');
  if (_req.method === 'OPTIONS') {
    return res.sendStatus(204);
  }
  next();
});

// ————————————————————————————————————————
// Routes
// ————————————————————————————————————————

/**
 * POST /api/metrics
 * Receives a batch of metrics from the browser.
 *
 * Expected payload:
 * {
 *   sessionId: string,
 *   metrics: {
 *     pageLoadDuration?: number,
 *     lcp?: number,
 *     fid?: number,
 *     cls?: number,
 *     routeNavigations?: number,
 *     errors?: number,
 *     heartbeat?: boolean
 *   }
 * }
 */
app.post('/api/metrics', (req, res) => {
  try {
    const { sessionId, metrics } = req.body;

    if (!sessionId || !metrics) {
      return res.status(400).json({ error: 'Missing sessionId or metrics' });
    }

    // Update heartbeat tracking
    if (metrics.heartbeat) {
      // Clear old timeout
      if (sessionHeartbeats.has(sessionId)) {
        clearTimeout(sessionHeartbeats.get(sessionId));
      }

      // Set new timeout
      sessionHeartbeats.set(
        sessionId,
        setTimeout(() => {
          sessionHeartbeats.delete(sessionId);
          updateSessionMetrics();
        }, HEARTBEAT_TIMEOUT_MS)
      );

      updateSessionMetrics();
    }

    // Record metrics
    if (typeof metrics.pageLoadDuration === 'number' && metrics.pageLoadDuration > 0) {
      pageLoadDuration.observe(metrics.pageLoadDuration);
    }

    if (typeof metrics.lcp === 'number') {
      lcpGauge.set(metrics.lcp);
    }

    if (typeof metrics.fid === 'number') {
      fidGauge.set(metrics.fid);
    }

    if (typeof metrics.cls === 'number') {
      clsGauge.set(metrics.cls);
    }

    if (typeof metrics.routeNavigations === 'number' && metrics.routeNavigations > 0) {
      routeNavigations.inc(metrics.routeNavigations);
    }

    if (typeof metrics.errors === 'number' && metrics.errors > 0) {
      errorsTotal.inc(metrics.errors);
    }

    res.status(200).json({ status: 'ok' });
  } catch (err) {
    console.error('Error processing metrics:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

/**
 * GET /metrics
 * Prometheus scrape endpoint — returns all metrics in text format.
 */
app.get('/metrics', async (_req, res) => {
  try {
    res.set('Content-Type', register.contentType);
    res.end(await register.metrics());
  } catch (err) {
    console.error('Error generating metrics:', err);
    res.status(500).end();
  }
});

/**
 * GET /health
 * Simple health check for the metrics server itself.
 */
app.get('/health', (_req, res) => {
  res.status(200).json({ status: 'ok', service: 'iviss-metrics-server' });
});

// ————————————————————————————————————————
// Helpers
// ————————————————————————————————————————

function updateSessionMetrics() {
  const count = sessionHeartbeats.size;
  activeSessions.set(count);
  frontendUp.set(count > 0 ? 1 : 0);
}

// ————————————————————————————————————————
// Start Server
// ————————————————————————————————————————

const PORT = process.env.METRICS_PORT || 9091;

app.listen(PORT, () => {
  console.log(`[iviss-metrics-server] Listening on port ${PORT}`);
  console.log(`[iviss-metrics-server] POST /api/metrics — receive browser metrics`);
  console.log(`[iviss-metrics-server] GET  /metrics     — Prometheus scrape endpoint`);
  console.log(`[iviss-metrics-server] GET  /health      — health check`);
});
