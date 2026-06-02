# Ticket 1 — Infrastructure (Admin)

**Requester:** IVISS Dev Team
**Cluster:** Hetzner K8s (k3s)
**Namespace:** `iviss`

## Prerequisites

The CNPG operator must be installed on the cluster before applying these resources:

```bash
kubectl apply -f https://raw.githubusercontent.com/cloudnative-pg/cloudnative-pg/main/releases/cnpg-1.24.0.yaml
kubectl wait deployment -n cnpg-system cnpg-controller-manager --for=condition=Available --timeout=120s
```

Nginx Ingress Controller must also be installed:

```bash
helm upgrade --install ingress-nginx ingress-nginx \
  --repo https://kubernetes.github.io/ingress-nginx \
  --version 1.12.1 \
  --namespace ingress-nginx --create-namespace \
  --set controller.service.type=LoadBalancer
```

## Resources to Create

### 1. Namespace

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: iviss
  labels:
    app.kubernetes.io/part-of: iviss
```

### 2. CNPG Database Secrets

Replace all `REPLACE_ME` values with actual credentials before applying.

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: iviss-postgres-superuser
  namespace: iviss
  labels:
    app.kubernetes.io/part-of: iviss
    app.kubernetes.io/component: database
    cnpg.io/reload: "true"
type: kubernetes.io/basic-auth
stringData:
  username: postgres
  password: REPLACE_ME                  # generate: openssl rand -base64 32
---
apiVersion: v1
kind: Secret
metadata:
  name: iviss-postgres-app
  namespace: iviss
  labels:
    app.kubernetes.io/part-of: iviss
    app.kubernetes.io/component: database
    cnpg.io/reload: "true"
type: kubernetes.io/basic-auth
stringData:
  username: iviss_user
  password: REPLACE_ME                  # generate: openssl rand -base64 32
```

### 3. CNPG Cluster

```yaml
apiVersion: postgresql.cnpg.io/v1
kind: Cluster
metadata:
  name: iviss-postgres
  namespace: iviss
  labels:
    app.kubernetes.io/part-of: iviss
    app.kubernetes.io/component: database
spec:
  instances: 3
  imageName: ghcr.io/cloudnative-pg/postgresql:15
  primaryUpdateStrategy: supervised
  superuserSecret:
    name: iviss-postgres-superuser
  bootstrap:
    initdb:
      database: iviss_dev
      owner: iviss_user
      secret:
        name: iviss-postgres-app
  storage:
    size: 10Gi
  resources:
    requests:
      cpu: 250m
      memory: 256Mi
    limits:
      cpu: 500m
      memory: 512Mi
  monitoring:
    enablePodMonitor: false
```

### 4. Application Secrets

Replace all `REPLACE_ME` values with actual credentials before applying.

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: iviss-secrets
  namespace: iviss
  labels:
    app.kubernetes.io/part-of: iviss
type: Opaque
stringData:
  JWT_PRIVATE_KEY_PEM: REPLACE_ME
  JWT_PUBLIC_KEY_PEM: REPLACE_ME
  ACTIVATION_CODE_PEPPER: REPLACE_ME
  SMS_PROVIDER: vonage
  VONAGE_API_KEY: REPLACE_ME
  VONAGE_API_SECRET: REPLACE_ME
  EMAIL_PROVIDER: mock
  EXTERNAL_API_BASE_URL: REPLACE_ME
  EXTERNAL_API_USERNAME: REPLACE_ME
  EXTERNAL_API_PASSWORD: REPLACE_ME
  EXTERNAL_API_LOCK_NDIA: REPLACE_ME
  EXTERNAL_API_KINDIA: REPLACE_ME
  EXTERNAL_API_USER: REPLACE_ME
  EXTERNAL_API_CLIENT: REPLACE_ME
  EXTERNAL_API_CTR: REPLACE_ME
  EXTERNAL_API_TLS_CERT_B64: REPLACE_ME
  ADMIN_BOOTSTRAP_EMAIL: admin@iviss.cloud
  ADMIN_BOOTSTRAP_PASSWORD: REPLACE_ME
  ADMIN_BOOTSTRAP_PHONE: "+237600000000"
  ADMIN_BOOTSTRAP_USERNAME: admin
```

### 5. ServiceAccount

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: iviss
  namespace: iviss
  labels:
    app.kubernetes.io/part-of: iviss
```

### 6. TLS Certificate (optional — for Ingress)

If not using CloudFront for TLS termination, request a TLS certificate for the domain:

```yaml
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
```

## Verification

After applying, verify everything is ready before Wave 2:

```bash
# Namespace exists
kubectl get namespace iviss

# CNPG cluster is ready
kubectl get cluster -n iviss iviss-postgres
kubectl wait cluster/iviss-postgres -n iviss --for=condition=Ready --timeout=300s

# CNPG pods are running
kubectl get pods -n iviss -l cnpg.io/cluster=iviss-postgres

# Secrets exist
kubectl get secrets -n iviss

# Database is accessible
kubectl exec -n iviss iviss-postgres-1 -- psql -U iviss_user -d iviss_dev -c "SELECT 1"
```

## What This Enables

Once all resources above are created, the dev team can deploy Wave 2 (the application) via ArgoCD without any admin permissions required.