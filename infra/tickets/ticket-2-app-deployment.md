# Ticket 2 — Application Deployment (Dev Team via ArgoCD)

**Deployer:** IVISS Dev Team
**Namespace:** `iviss` (already created by admin in Ticket 1)
**Method:** ArgoCD auto-sync from `charts/iviss/` in the `main` branch

## Prerequisites (from Ticket 1)

Before deploying, confirm the admin has completed:
- [ ] Namespace `iviss` exists
- [ ] CNPG cluster `iviss-postgres` is running and Ready
- [ ] Secrets `iviss-secrets`, `iviss-postgres-superuser`, `iviss-postgres-app` exist
- [ ] ServiceAccount `iviss` exists
- [ ] Nginx Ingress Controller is installed
- [ ] CNPG operator is installed

## What ArgoCD Deploys

All resources in `charts/iviss/templates/` **except** the ones created by the admin in Ticket 1:

| Resource | Type | File |
|---|---|---|
| Backend ConfigMap | ConfigMap | `configmap.yaml` |
| Backend Deployment | Deployment | `backend-deployment.yaml` |
| Backend Service | Service | `backend-service.yaml` |
| Backend HPA | HorizontalPodAutoscaler | `backend-hpa.yaml` |
| Frontend Deployment | Deployment | `frontend-deployment.yaml` |
| Frontend Service | Service | `frontend-service.yaml` |
| Frontend HPA | HorizontalPodAutoscaler | `frontend-hpa.yaml` |
| Frontend Ingress | Ingress | `ingress.yaml` |
| API Ingress | Ingress | `ingress-api.yaml` |

## Deployment Flow

```
Push to main branch
  → GitHub Actions builds Docker images → pushes to GHCR
  → GitHub Actions updates charts/iviss/values-production.yaml with new image tag
  → ArgoCD detects change → syncs Helm chart → rolls out new pods
```

## How to Deploy

Option A — ArgoCD auto-sync (recommended):
```bash
# Already configured in argocd/application.yaml
# ArgoCD watches the main branch and auto-syncs on every push
```

Option B — Manual sync:
```bash
argocd app sync iviss
```

Option C — Force image tag update:
```bash
# Edit values-production.yaml, commit and push
argocd app set iviss -p global.imageTag=v1.2.3
```

## How to Update Secrets

Secrets are managed by the admin (Ticket 1). To rotate:

1. Admin updates the secret: `kubectl edit secret iviss-secrets -n iviss`
2. ArgoCD re-deploys the backend (checksum annotation triggers rollout)

## How to Scale Database

Update `instances` in `charts/iviss/values-production.yaml`:
```yaml
database:
  instances: 3  # scale up from 1
```

ArgoCD syncs the change and CNPG handles the rolling update.

## Verification

```bash
# Check ArgoCD app status
argocd app get iviss

# Check pod status
kubectl get pods -n iviss

# Check backend health
kubectl port-forward -n iviss svc/iviss-backend 3000:3000
curl http://localhost:3000/api/v1/health

# Check frontend
kubectl port-forward -n iviss svc/iviss-frontend 8080:80
curl http://localhost:8080

# Check database connectivity
kubectl exec -n iviss deploy/iviss-backend -- env | grep DATABASE_URL
```