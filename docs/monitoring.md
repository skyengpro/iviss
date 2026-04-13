# IVISS Monitoring Stack

Prometheus + Grafana observability for the IVISS frontend.

## Architecture

```
Browser (React SPA)
    │
    │  POST /api/metrics (every 10s)
    ▼
┌─────────────────────┐
│  Metrics Server     │ :9091
│  (Node.js/Express)  │
│  GET /metrics       │
└────────┬────────────┘
         │  Scrape (every 10s)
         ▼
┌─────────────────────┐
│    Prometheus       │ :9090
│  (Time-series DB)   │
└────────┬────────────┘
         │  Query
         ▼
┌─────────────────────┐
│     Grafana         │ :3001
│   (Dashboards)      │
└─────────────────────┘
```

## Quick Start

```bash
# From the project root — starts everything (including monitoring)
docker compose up -d

# Check all services are running
docker compose ps
```

## Access

| Service            | Precise URL                                                                | Credentials       |
| ------------------ | -------------------------------------------------------------------------- | ----------------- |
| **Frontend**       | [http://localhost:8080/](http://localhost:8080/)                           | —                 |
| **Grafana**        | [http://localhost:3001/login](http://localhost:3001/login)                 | `admin` / `admin` |
| **Prometheus**     | [http://localhost:9090/targets](http://localhost:9090/targets)             | —                 |
| **Metrics Data**   | [http://localhost:9091/metrics](http://localhost:9091/metrics)             | —                 |
| **Backend Health** | [http://localhost:3000/api/v1/health](http://localhost:3000/api/v1/health) | —                 |

## Verification Steps

1. **Start the stack:**

   ```bash
   docker compose up -d
   ```

2. **Generate Traffic:**
   - Open **Frontend** at [http://localhost:8080/](http://localhost:8080/)
   - Navigate to different pages to generate data.

3. **Check Status:**
   - **Prometheus**: Go to [http://localhost:9090/targets](http://localhost:9090/targets) -> Ensure `frontend-metrics` is **UP**.
   - **Grafana**: Go to [http://localhost:3001/](http://localhost:3001/) -> `admin`/`admin` -> **Dashboards** -> **IVISS Frontend Monitoring**.
   - **Metrics**: Go to [http://localhost:9091/metrics](http://localhost:9091/metrics) -> Should see raw text data.

A pre-built **"IVISS Frontend Monitoring"** dashboard is auto-provisioned with:

| Panel                    | Description                          |
| ------------------------ | ------------------------------------ |
| Frontend Availability    | UP/DOWN heartbeat status             |
| Active Sessions          | Concurrent browser sessions          |
| Total Errors             | Cumulative JS error count            |
| Route Navigations        | Client-side navigation count         |
| Page Load Duration       | Load time over time                  |
| Errors Over Time         | Error rate (per minute)              |
| LCP Gauge                | Largest Contentful Paint (Web Vital) |
| FID Gauge                | First Input Delay (Web Vital)        |
| CLS Gauge                | Cumulative Layout Shift (Web Vital)  |
| Route Navigations / Time | Navigation rate over time            |
| Active Sessions / Time   | Session count over time              |

## Collected Metrics

| Metric                             | Type      | Source                                  |
| ---------------------------------- | --------- | --------------------------------------- |
| `frontend_up`                      | Gauge     | Heartbeat from active sessions          |
| `frontend_active_sessions`         | Gauge     | Concurrent browser sessions             |
| `frontend_page_load_duration_ms`   | Histogram | Navigation Timing API                   |
| `frontend_lcp_ms`                  | Gauge     | PerformanceObserver                     |
| `frontend_fid_ms`                  | Gauge     | PerformanceObserver                     |
| `frontend_cls`                     | Gauge     | PerformanceObserver                     |
| `frontend_route_navigations_total` | Counter   | React Router location changes           |
| `frontend_errors_total`            | Counter   | `window.onerror` + `unhandledrejection` |

## Configuration

### Environment Variables

**Root `.env`:**
| Variable | Default | Description |
|----------|---------|-------------|
| `GRAFANA_ADMIN_USER` | `admin` | Grafana login username |
| `GRAFANA_ADMIN_PASSWORD` | `admin` | Grafana login password |
| `PROMETHEUS_RETENTION` | `15d` | How long Prometheus keeps metrics |

**Frontend `.env`:**
| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_METRICS_ENABLED` | `true` | Enable/disable browser metrics collection |
| `VITE_METRICS_URL` | `http://localhost:9091/api/metrics` | Metrics server endpoint |

### Disabling Metrics

Set `VITE_METRICS_ENABLED=false` in `frontend/.env` to disable browser-side metrics collection without removing any code.

## Adding Custom Metrics

1. Define new metrics in `frontend/metrics-server.js` using `prom-client`
2. Send them from the browser via `frontend/src/services/metricsCollector.ts`
3. Add panels to the Grafana dashboard in `monitoring/grafana/dashboards/frontend-dashboard.json`

## Troubleshooting

| Issue                           | Solution                                                                         |
| ------------------------------- | -------------------------------------------------------------------------------- |
| Prometheus shows target as DOWN | Check `docker compose logs metrics-server`                                       |
| No data in Grafana              | Open the frontend in a browser and wait ~15s                                     |
| Grafana can't reach Prometheus  | Ensure both are on `iviss-network`: `docker network inspect iviss_iviss-network` |
| Metrics not updating            | Check browser console for fetch errors to `/api/metrics`                         |
