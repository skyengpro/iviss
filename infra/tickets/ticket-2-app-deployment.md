# Ticket 2 — Application Deployment (Dev Team via ArgoCD)

**Deployer:** IVISS Dev Team  
**Namespace:** `iviss` (created by admin in Ticket 1)  
**Method:** ArgoCD auto-sync from `charts/` in the `main` branch

## Prerequisites (from Ticket 1)

Before deploying, confirm the admin has completed:

- [ ] Namespace `iviss` exists
- [ ] CNPG operator, ESO, Nginx Ingress, cert-manager, ArgoCD are installed
- [ ] CNPG cluster `iviss-postgres` is running and Ready
- [ ] ESO is syncing: `kubectl get externalsecrets -n iviss` shows `Ready`
- [ ] Secret `iviss-secrets` exists (ESO-pulled from AWS)
- [ ] Secret `iviss-static-secrets` exists (admin-created)
- [ ] ServiceAccount `iviss` exists

## ArgoCD Applications

Two applications (admin creates the project + apps):

| App | Chart Path | What it deploys |
|---|---|---|
| `iviss-backend` | `charts/backend/` | API Deployment, Service, HPA, ConfigMap, API Ingress |
| `iviss-frontend` | `charts/frontend/` | Dashboard Deployment, Service, HPA, ConfigMap, Frontend Ingress |

Register them:
```bash
kubectl apply -k argocd/
```

## Secret Flow

```
AWS Secrets Manager (admin creates these)
  ├── iviss/prod/app-secrets         ──ESO──▶  iviss-secrets (JWT, pepper, admin pw)
  ├── iviss/prod/provider-keys       ──ESO──▶  iviss-secrets (SMS, email, API keys)
  └── iviss/prod/vehicle-api-keys    ──ESO──▶  iviss-secrets (vehicle API credentials)

Admin-created (static):
  └── iviss-static-secrets (bootstrap, SMS/EMAIL provider choice)

CNPG-managed (auto-created):
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
argocd app sync iviss-backend
argocd app sync iviss-frontend
```

**Force image tag:**
```bash
argocd app set iviss-backend -p global.imageTag=v1.2.3
argocd app set iviss-frontend -p global.imageTag=v1.2.3
```

## Updating Config

Non-sensitive config is in Helm values:
- `charts/backend/values-production.yaml` (shift hours, SMS provider, etc.)
- `charts/frontend/values-production.yaml` (API URL, etc.)

Sensitive values are managed in:
- **AWS Secrets Manager** → ESO syncs them into `iviss-secrets` automatically
- **`iviss-static-secrets`** → admin updates directly: `kubectl apply -f infra/manifests/static-secrets.yaml`

## Updating Secrets

**AWS-managed secrets (JWT, API keys, vehicle API):**
1. Update in AWS Secrets Manager console or CLI
2. ESO syncs every 1 hour — or force: `kubectl annotate externalsecret iviss-app-secrets -n iviss force-sync=$(date +%s) --overwrite`

**Static secrets (admin bootstrap, provider choices):**
1. Edit `infra/manifests/static-secrets.yaml`
2. `kubectl apply -f infra/manifests/static-secrets.yaml`

**Database password:**
1. CNPG manages it via `iviss-postgres-app` secret
2. To rotate: update the secret, CNPG reloads automatically

## Verification

```bash
# ArgoCD app status
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