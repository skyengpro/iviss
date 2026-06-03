# Ticket 1 — Database Setup (Admin)

**Admin creates:** CNPG cluster only. Everything else (ServiceAccount, ExternalSecrets, static secrets) is managed by the dev via ArgoCD.

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

# 3. External Secrets Operator (needed before dev deploys ExternalSecret CRDs)
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

kubectl apply -f infra/scripts/cluster-issuer.yaml

# 6. ArgoCD
helm upgrade --install argocd argo-cd \
  --repo https://argoproj.github.io/argo-helm \
  --version 7.8.0 \
  --namespace argocd --create-namespace \
  --set server.service.type=LoadBalancer \
  --wait
```

---

## Step 2 — ClusterSecretStore (connect ESO to AWS)

Create IAM credentials with `secretsmanager:GetSecretValue` on:
- `arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/prod/app-secrets-*`
- `arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/prod/provider-keys-*`
- `arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/prod/vehicle-api-keys-*`

Then apply:

```bash
kubectl apply -f infra/manifests/cluster-secret-store.yaml
```

---

## Step 3 — Create the Database

```bash
# 1. Generate passwords
POSTGRES_SUPER_PASSWORD=$(openssl rand -base64 32)
POSTGRES_APP_PASSWORD=$(openssl rand -base64 32)

echo "SAVE THESE:"
echo "  POSTGRES_SUPER_PASSWORD: ${POSTGRES_SUPER_PASSWORD}"
echo "  POSTGRES_APP_PASSWORD: ${POSTGRES_APP_PASSWORD}"

# 2. Edit cnpg-secrets.yaml and replace the placeholder passwords, then apply
kubectl apply -f infra/manifests/cnpg-secrets.yaml

# 3. Create the CNPG cluster
kubectl apply -f infra/manifests/cnpg-cluster.yaml

# 4. Wait for cluster to be ready
kubectl wait cluster/iviss-postgres -n iviss --for=condition=Ready --timeout=300s
```

---

## Step 4 — Verify

```bash
# CNPG cluster is ready
kubectl get cluster -n iviss iviss-postgres
kubectl get pods -n iviss -l cnpg.io/cluster=iviss-postgres

# Database accessible
DB_PASS=$(kubectl get secret iviss-postgres-app -n iviss -o jsonpath='{.data.password}' | base64 -d)
kubectl exec -n iviss iviss-postgres-1 -- psql "postgresql://iviss_user:${DB_PASS}@localhost:5432/iviss_dev" -c "SELECT 1"
```

---

## Admin's `infra/manifests/`

| File | What it creates |
|---|---|
| `cnpg-secrets.yaml` | K8s Secrets `iviss-postgres-superuser` + `iviss-postgres-app` |
| `cnpg-cluster.yaml` | CNPG Cluster `iviss-postgres` (3 instances, 10Gi) |

Everything else (ServiceAccount, ExternalSecrets, static secrets, backend, frontend) is managed by the dev via ArgoCD.