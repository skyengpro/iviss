# Ticket 2 — Application Deployment (Dev Team via ArgoCD)

**Deployer:** IVISS Dev Team  
**Namespace:** `iviss` (created by admin in Ticket 1)  
**Method:** ArgoCD auto-sync from `charts/` directory in the `main` branch

## Prerequisites (from Ticket 1)

Before deploying, confirm the admin has completed:

- [ ] Namespace `iviss` exists
- [ ] CNPG operator, ESO, Nginx Ingress, cert-manager, ArgoCD are installed
- [ ] `iviss-database` ArgoCD app is synced and healthy
- [ ] CNPG cluster `iviss-postgres` is running and Ready
- [ ] ESO is syncing: `kubectl get externalsecrets -n iviss` shows `Ready`
- [ ] Secret `iviss-secrets` exists (ESO-pulled from AWS)
- [ ] Secret `iviss-static-secrets` exists
- [ ] ServiceAccount `iviss` exists
- [ ] Nginx Ingress Controller is running
- [ ] ESO ClusterSecretStore `aws-secretsmanager` is ready

## ArgoCD Applications

Three applications exist in `argocd/`:

| App | Chart Path | Description | Sync Wave |
|---|---|---|---|
| `iviss-database` | `charts/database/` | CNPG, secrets, ExternalSecrets, ServiceAccount | 0 |
| `iviss-backend` | `charts/backend/` | API server, ConfigMap, Ingress (API) | 1 |
| `iviss-frontend` | `charts/frontend/` | Dashboard, ConfigMap, Ingress | 1 |

## What Each ArgoCD App Deploys

### iviss-database (sync-wave 0)
- ServiceAccount `iviss`
- CNPG Secrets (`iviss-postgres-superuser`, `iviss-postgres-app`)
- CNPG Cluster `iviss-postgres`
- 3 ExternalSecrets → `iviss-secrets` (app, provider, vehicle-api)
- Static Secret `iviss-static-secrets`

### iviss-frontend (sync-wave 1)
| Resource | Type | File |
|---|---|---|
| Frontend Config | ConfigMap | `configmap.yaml` |
| Frontend | Deployment | `deployment.yaml` |
| Frontend | Service | `service.yaml` |
| Frontend | HPA | `hpa.yaml` |
| Frontend Ingress | Ingress | `ingress.yaml` |

### iviss-backend (sync-wave 1)
| Resource | Type | File |
|---|---|---|
| Backend Config | ConfigMap | `configmap.yaml` |
| Backend | Deployment | `deployment.yaml` |
| Backend | Service | `service.yaml` |
| Backend | HPA | `hpa.yaml` |
| API Ingress | Ingress | `ingress.yaml` |

## Secret Flow

```
AWS Secrets Manager
  ├── iviss/prod/app-secrets         ──ESO──▶  iviss-secrets (JWT, pepper, admin pw)
  ├── iviss/prod/provider-keys       ──ESO──▶  iviss-secrets (SMS, email, API keys)
  └── iviss/prod/vehicle-api-keys    ──ESO──▶  iviss-secrets (vehicle API credentials)

Helm-managed (iviss-database chart):
  └── iviss-static-secrets (bootstrap, SMS/EMAIL provider choice)

CNPG-managed:
  ├── iviss-postgres-superuser
  └── iviss-postgres-app
```

## Deployment Flow

```
Push to main branch
  → GitHub Actions builds Docker images → pushes to GHCR
  → GitHub Actions updates values-production.yaml with new image tag
  → ArgoCD detects change → syncs Helm charts → rolls out new pods
```

## How to Deploy

**Auto-sync (recommended):**
ArgoCD watches the `main` branch and auto-syncs on every push.

**Manual sync:**
```bash
argocd app sync iviss-database
argocd app sync iviss-backend
argocd app sync iviss-frontend
```

**Force image tag:**
```bash
argocd app set iviss-backend -p global.imageTag=v1.2.3
argocd app set iviss-frontend -p global.imageTag=v1.2.3
```

## Updating Config Values

Non-sensitive config (e.g., `SHIFT_START_HOUR`, `SMS_PROVIDER`) is managed in:
- `charts/backend/values-production.yaml`
- `charts/frontend/values-production.yaml`

Edit, commit, push — ArgoCD picks it up.

Sensitive values are managed in:
- **AWS Secrets Manager** → ESO syncs them into `iviss-secrets` automatically
- **`iviss-static-secrets`** → managed by Helm values in `charts/database/values.yaml`

## Updating Secrets

**AWS-managed secrets (JWT, API keys, vehicle API):**
1. Update in AWS Secrets Manager console or CLI
2. ESO syncs every 1 hour — or force: `kubectl annotate externalsecret iviss-app-secrets -n iviss force-sync=$(date +%s) --overwrite`

**Static secrets (admin bootstrap, provider choices):**
1. Edit `charts/database/values.yaml`
2. Commit and push — ArgoCD applies the change

**Database password:**
1. CNPG manages it — `iviss-postgres-app` secret
2. To rotate: update the Helm values, ArgoCD reconciles

## Verification

```bash
# ArgoCD app status
argocd app get iviss-database
argocd app get iviss-backend
argocd app get iviss-frontend

# Pod status
kubectl get pods -n iviss

# Backend health
kubectl port-forward -n iviss svc/iviss-backend 3000:3000
curl http://localhost:3000/api/v1/health

# Database connectivity
kubectl exec -n iviss deploy/iviss-backend -- env | grep DATABASE_URL

# CNPG cluster
kubectl get cluster -n iviss iviss-postgres
```