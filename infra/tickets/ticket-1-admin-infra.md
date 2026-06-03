# Ticket 1 — Database Setup (Admin)

**Admin creates:** CNPG cluster only. Everything else is managed by the dev via ArgoCD.

---

## Step 1 — Prerequisites (Operators)

```bash
# 1. Namespace
kubectl apply -f - <<EOF
apiVersion: v1
kind: Namespace
metadata:
  name: iviss
  labels:
    app.kubernetes.io/part-of: iviss
EOF

# 2. Cloud Native PG operator
kubectl apply -f https://raw.githubusercontent.com/cloudnative-pg/cloudnative-pg/main/releases/cnpg-1.24.0.yaml
kubectl wait deployment -n cnpg-system cnpg-controller-manager --for=condition=Available --timeout=120s

# 3. External Secrets Operator
helm upgrade --install external-secrets external-secrets \
  --repo https://charts.external-secrets.io \
  --version 0.12.1 \
  --namespace external-secrets-system --create-namespace \
  --wait

# 4. Nginx Ingress Controller
helm upgrade --install ingress-nginx ingress-nginx \
  --repo https://kubernetes.github.io/ingress-nginx \
  --version 1.12.1 \
  --namespace ingress-nginx --create-namespace \
  --set controller.service.type=LoadBalancer \
  --set controller.config.use-forwarded-headers=true \
  --wait

# 5. cert-manager + ClusterIssuer
helm upgrade --install cert-manager cert-manager \
  --repo https://charts.jetstack.io \
  --version 1.17.1 \
  --namespace cert-manager --create-namespace \
  --set crds.enabled=true \
  --wait
```

**ClusterIssuer** (created by `bootstrap-k8s.sh` or manually):
```bash
kubectl apply -f - <<EOF
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@iviss.cloud
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
---
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-staging
spec:
  acme:
    server: https://acme-staging-v02.api.letsencrypt.org/directory
    email: admin@iviss.cloud
    privateKeySecretRef:
      name: letsencrypt-staging
    solvers:
    - http01:
        ingress:
          class: nginx
EOF
```

**ClusterSecretStore** (connect ESO to AWS):
The IAM user needs `secretsmanager:GetSecretValue` on:
- `arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/prod/*`

```bash
kubectl apply -f infra/manifests/cluster-secret-store.yaml
```

---

## Step 2 — Verify AWS Secrets Manager Contents

```bash
aws secretsmanager get-secret-value \
  --secret-id iviss/prod/app-secrets --region eu-west-1 \
  --query SecretString --output text | python3 -m json.tool

# Provider keys (SMS, email, SMTP)
aws secretsmanager get-secret-value \
  --secret-id iviss/prod/provider-keys --region eu-west-1 \
  --query SecretString --output text | python3 -m json.tool

# Vehicle API keys
aws secretsmanager get-secret-value \
  --secret-id iviss/prod/vehicle-api-keys --region eu-west-1 \
  --query SecretString --output text | python3 -m json.tool
```

---

## Step 3 — Create the Database

```bash
kubectl apply -f infra/manifests/cnpg-external-secret.yaml

# 2. CNPG Cluster + ConfigMap + NetworkPolicy
kubectl apply -f infra/manifests/cnpg-cluster.yaml

# 3. Wait for cluster ready
kubectl wait cluster/iviss-db -n iviss --for=condition=Ready --timeout=300s
```

---

## Step 4 — Verify

```bash
# CNPG cluster is ready
kubectl get cluster -n iviss iviss-db
kubectl get pods -n iviss -l cnpg.io/cluster=iviss-db

# Database accessible
DB_PASS=$(kubectl get secret iviss-db-app -n iviss -o jsonpath='{.data.password}' | base64 -d)
kubectl exec -n iviss iviss-db-1 -- psql "postgresql://iviss:${DB_PASS}@localhost:5432/iviss" -c "SELECT 1"
```

---

## What the Admin Creates (manual)

| File | What it creates |
|---|---|
| `infra/manifests/cnpg-cluster.yaml` | CNPG Cluster `iviss-db`, ConfigMap, NetworkPolicy |

## What the Dev Creates (ArgoCD, `kubectl apply -k argocd/`)

| App | Chart | What it manages |
|---|---|---|
| `iviss-infra` | `charts/infra/` | ServiceAccount, 3 ExternalSecrets, static secrets |
| `iviss-backend` | `charts/backend/` | API Deployment, ConfigMap, Service, HPA, Ingress |
| `iviss-frontend` | `charts/frontend/` | Dashboard Deployment, ConfigMap, Service, HPA, Ingress |