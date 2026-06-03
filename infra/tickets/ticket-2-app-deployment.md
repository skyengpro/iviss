# Ticket 2 — Application Deployment (Dev Team via ArgoCD)

**Deployer:** IVISS Dev Team  
**Namespace:** `iviss` (created by admin in Ticket 1)

## Prerequisites (from Ticket 1)

- [ ] Namespace `iviss` exists
- [ ] CNPG cluster `iviss-db` is running and Ready
- [ ] ExternalSecret `iviss-db-app` exists (ESO-pulled from AWS)
- [ ] ESO + ClusterSecretStore `aws-secretsmanager` are installed and ready
- [ ] Nginx Ingress Controller is running
- [ ] cert-manager + ClusterIssuer installed
- [ ] ArgoCD is installed

## ArgoCD Applications (3 apps)

```
kubectl apply -k argocd/
```

| App | Chart Path | Sync Wave | What it manages |
|---|---|---|---|
| `iviss-infra` | `charts/infra/` | 0 | ServiceAccount, 3 ExternalSecrets, static secrets |
| `iviss-backend` | `charts/backend/` | 1 | API Deployment, Service, HPA, ConfigMap, API Ingress |
| `iviss-frontend` | `charts/frontend/` | 1 | Dashboard Deployment, Service, HPA, ConfigMap, Frontend Ingress |

### What `iviss-infra` creates (sync-wave 0)

| Resource | File |
|---|---|
| ServiceAccount `iviss` | `serviceaccount.yaml` |
| ExternalSecret `iviss-app-secrets` | `externalsecret-app.yaml` |
| ExternalSecret `iviss-provider-keys` | `externalsecret-provider.yaml` |
| ExternalSecret `iviss-vehicle-api-keys` | `externalsecret-vehicle-api.yaml` |
| Secret `iviss-static-secrets` | `static-secrets.yaml` |

### What the admin creates (manual, NOT in ArgoCD)

| Resource | File |
|---|---|
| ExternalSecret `iviss-db-app` | `infra/manifests/cnpg-external-secret.yaml` |
| CNPG Cluster `iviss-db` + ConfigMap + NetworkPolicy | `infra/manifests/cnpg-cluster.yaml` |

## Secret Flow

```
AWS Secrets Manager (admin populates these)
  ├── iviss/prod/app-secrets         ──ESO──▶  iviss-secrets (JWT, pepper, admin pw)
  ├── iviss/prod/provider-keys       ──ESO──▶  iviss-secrets (SMS, email, API keys)
  ├── iviss/prod/vehicle-api-keys    ──ESO──▶  iviss-secrets (vehicle API credentials)
  └── iviss/prod/app-secrets         ──ESO──▶  iviss-db-app (db_password for CNPG)

ArgoCD-managed (iviss-infra chart):
  └── iviss-static-secrets (bootstrap, SMS/EMAIL provider choice)

Admin-created (manual):
  └── CNPG Cluster iviss-db (auto-creates superuser secret)
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
ArgoCD watches `main` and auto-syncs.

**Manual sync:**
```bash
argocd app sync iviss-infra
argocd app sync iviss-backend
argocd app sync iviss-frontend
```

## Updating Secrets

**AWS-managed secrets (JWT, API keys, vehicle API):**
1. Update in AWS Secrets Manager
2. ESO syncs every 1h — or force: `kubectl annotate externalsecret iviss-app-secrets -n iviss force-sync=$(date +%s) --overwrite`

**Static secrets (admin bootstrap, provider choices):**
1. Edit `charts/infra/values.yaml`
2. Commit and push — ArgoCD applies the change

**Database password:**
1. Update `db_password` in AWS Secrets Manager (`iviss/prod/app-secrets`)
2. ESO syncs `iviss-db-app` secret → CNPG runs ALTER ROLE automatically

## Verification

```bash
# ArgoCD app status
argocd app get iviss-infra
argocd app get iviss-backend
argocd app get iviss-frontend

# External Secrets synced
kubectl get externalsecrets -n iviss
kubectl get secret iviss-secrets -n iviss -o jsonpath='{.data}' | jq 'keys'

# CNPG database
kubectl get cluster -n iviss iviss-db

# Pods running
kubectl get pods -n iviss

# Backend health
kubectl port-forward -n iviss svc/iviss-backend 3000:3000
curl http://localhost:3000/api/v1/health

# Database connectivity
kubectl exec -n iviss deploy/iviss-backend -- env | grep DATABASE_URL
```