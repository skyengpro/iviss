# Ticket 2 — Application Deployment (Dev Team via ArgoCD)

**Deployer:** IVISS Dev Team  
**Namespace:** `iviss` (already created by admin in Ticket 1)  
**Method:** ArgoCD auto-sync from `charts/iviss/` in the `main` branch

## Prerequisites (from Ticket 1)

Before deploying, confirm the admin has completed:

- [ ] Namespace `iviss` exists
- [ ] CNPG cluster `iviss-postgres` is running and Ready
- [ ] ESO is syncing: `kubectl get externalsecrets -n iviss` shows `Ready`
- [ ] Secret `iviss-secrets` exists (ESO-pulled from AWS)
- [ ] Secret `iviss-static-secrets` exists (admin-created)
- [ ] ServiceAccount `iviss` exists
- [ ] Nginx Ingress Controller is running
- [ ] CNPG operator is installed
- [ ] External Secrets Operator is installed

## What ArgoCD Deploys

All resources in `charts/iviss/templates/`:

| Resource | Type | File |
|---|---|---|
| App Config | ConfigMap | `configmap.yaml` |
| Backend | Deployment | `backend-deployment.yaml` |
| Backend | Service | `backend-service.yaml` |
| Backend | HPA | `backend-hpa.yaml` |
| Frontend | Deployment | `frontend-deployment.yaml` |
| Frontend | Service | `frontend-service.yaml` |
| Frontend | HPA | `frontend-hpa.yaml` |
| Ingress (frontend) | Ingress | `ingress.yaml` |
| Ingress (API) | Ingress | `ingress-api.yaml` |

**Admin-managed (NOT in chart — do NOT touch):**
- Namespace `iviss`
- CNPG Cluster `iviss-postgres`
- Secrets: `iviss-postgres-superuser`, `iviss-postgres-app`
- ExternalSecrets: `iviss-app-secrets`, `iviss-provider-keys`
- K8s Secrets: `iviss-secrets` (ESO-managed), `iviss-static-secrets`
- ServiceAccount `iviss`

## Secret Flow

```
AWS Secrets Manager
  ├── iviss/production/app-secrets       ──ESO──▶  iviss-secrets (JWT, pepper, admin pw)
  ├── iviss/production/provider-keys     ──ESO──▶  iviss-secrets (SMS, email, API keys)
  └── iviss/prod/provider-keys     ──ESO──▶  iviss-secrets (SMS, email, API keys)

Admin-created (static):
  └── iviss-static-secrets (bootstrap, external API, SMS/EMAIL provider choice)

CNPG-managed:
  ├── iviss-postgres-superuser
  └── iviss-postgres-app
```

## Deployment Flow

```
Push to main branch
  → GitHub Actions builds Docker images → pushes to GHCR
  → GitHub Actions updates charts/iviss/values-production.yaml with new image tag
  → ArgoCD detects change → syncs Helm chart → rolls out new pods
```

## How to Deploy

**Auto-sync (recommended):**
ArgoCD watches the `main` branch and auto-syncs on every push.

**Manual sync:**
```bash
argocd app sync iviss
```

**Force image tag:**
```bash
argocd app set iviss -p global.imageTag=v1.2.3
```

## Updating Config Values

Non-sensitive config (e.g., `SHIFT_START_HOUR`, `SMS_PROVIDER`) is managed in `charts/iviss/values-production.yaml`. Edit, commit, push — ArgoCD picks it up.

Sensitive values are managed in:
- **AWS Secrets Manager** → ESO syncs them into `iviss-secrets` automatically
- **`iviss-static-secrets`** → admin updates directly: `kubectl edit secret iviss-static-secrets -n iviss`

## Updating Secrets

**AWS-managed secrets (JWT, API keys, etc.):**
1. Update in AWS Secrets Manager console or CLI
2. ESO syncs every 1 hour — or force: `kubectl annotate externalsecret iviss-app-secrets -n iviss force-sync=$(date +%s) --overwrite`

**Static secrets (external API, admin bootstrap):**
1. `kubectl edit secret iviss-static-secrets -n iviss`
2. Pods restart automatically (checksum annotation in deployment)

**Database password:**
1. CNPG manages it — `iviss-postgres-app` secret
2. To rotate: update the secret, CNPG reloads automatically

## Verification

```bash
# ArgoCD app status
argocd app get iviss

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