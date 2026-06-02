# Ticket 1 — Infrastructure (Admin)

**Requester:** IVISS Dev Team  
**Cluster:** Hetzner K8s (k3s)  
**Namespace:** `iviss`  

---

## Prerequisites

### A. Namespace

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: iviss
  labels:
    app.kubernetes.io/part-of: iviss
```

### B. Operators (must be installed before anything else)

```bash
# 1. Cloud Native PG operator
kubectl apply -f https://raw.githubusercontent.com/cloudnative-pg/cloudnative-pg/main/releases/cnpg-1.24.0.yaml
kubectl wait deployment -n cnpg-system cnpg-controller-manager --for=condition=Available --timeout=120s

# 2. External Secrets Operator
helm upgrade --install external-secrets external-secrets \
  --repo https://charts.external-secrets.io \
  --version 0.12.1 \
  --namespace external-secrets-system --create-namespace \
  --wait

# 3. Nginx Ingress Controller
helm upgrade --install ingress-nginx ingress-nginx \
  --repo https://kubernetes.github.io/ingress-nginx \
  --version 1.12.1 \
  --namespace ingress-nginx --create-namespace \
  --set controller.service.type=LoadBalancer \
  --set controller.config.use-forwarded-headers=true \
  --wait

# 4. cert-manager
helm upgrade --install cert-manager cert-manager \
  --repo https://charts.jetstack.io \
  --version 1.17.1 \
  --namespace cert-manager --create-namespace \
  --set crds.enabled=true \
  --wait
```

### C. IAM Role for External Secrets

The ESO pods need permission to read from AWS Secrets Manager. Create an IAM role that the ESO service account can assume (via IRSA if using EKS, or via access keys for Hetzner):

```yaml
# ClusterSecretStore — connects ESO to AWS Secrets Manager
apiVersion: external-secrets.io/v1beta1
kind: ClusterSecretStore
metadata:
  name: aws-secretsmanager
spec:
  provider:
    aws:
      service: SecretsManager
      region: eu-west-1
      auth:
        secretRef:
          accessKeyIDSecretRef:
            name: aws-eso-credentials
            namespace: external-secrets-system
            key: access-key-id
          secretAccessKeySecretRef:
            name: aws-eso-credentials
            namespace: external-secrets-system
            key: secret-access-key
```

The `aws-eso-credentials` secret in `external-secrets-system` namespace:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: aws-eso-credentials
  namespace: external-secrets-system
type: Opaque
stringData:
  access-key-id: REPLACE_ME        # AWS IAM user with SecretsManager read access
  secret-access-key: REPLACE_ME    # corresponding secret key
```

> **The IAM user needs `secretsmanager:GetSecretValue` on the two ARNs:**
> - `arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/production/app-secrets-*`
> - `arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/production/provider-keys-*`
> - `arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/production/cloudfront-origin-secret-*`

---

## Resources to Create

### 1. ServiceAccount

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: iviss
  namespace: iviss
  labels:
    app.kubernetes.io/part-of: iviss
```

---

### 2. ExternalSecret — App Secrets

Pulls from AWS Secrets Manager secret `iviss/production/app-secrets` and creates K8s Secret `iviss-secrets`.

The AWS secret has this JSON shape (already seeded in AWS):
```json
{
  "jwt_private_key_pem": "...",
  "jwt_public_key_pem": "...",
  "activation_code_pepper": "...",
  "db_password": "...",
  "admin_bootstrap_password": "..."
}
```

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: iviss-app-secrets
  namespace: iviss
  labels:
    app.kubernetes.io/part-of: iviss
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secretsmanager
    kind: ClusterSecretStore
  target:
    name: iviss-secrets
    creationPolicy: Owner
    template:
      type: Opaque
      data:
        JWT_PRIVATE_KEY_PEM: "{{ .jwt_private_key_pem }}"
        JWT_PUBLIC_KEY_PEM: "{{ .jwt_public_key_pem }}"
        ACTIVATION_CODE_PEPPER: "{{ .activation_code_pepper }}"
        ADMIN_BOOTSTRAP_PASSWORD: "{{ .admin_bootstrap_password }}"
  data:
    - remoteRef:
        key: iviss/production/app-secrets
        property: jwt_private_key_pem
      secretKey: jwt_private_key_pem
    - remoteRef:
        key: iviss/production/app-secrets
        property: jwt_public_key_pem
      secretKey: jwt_public_key_pem
    - remoteRef:
        key: iviss/production/app-secrets
        property: activation_code_pepper
      secretKey: activation_code_pepper
    - remoteRef:
        key: iviss/production/app-secrets
        property: admin_bootstrap_password
      secretKey: admin_bootstrap_password
```

**Resulting K8s Secret `iviss-secrets`** (auto-created by ESO):
```
JWT_PRIVATE_KEY_PEM     ← from AWS iviss/production/app-secrets.jwt_private_key_pem
JWT_PUBLIC_KEY_PEM      ← from AWS iviss/production/app-secrets.jwt_public_key_pem
ACTIVATION_CODE_PEPPER  ← from AWS iviss/production/app-secrets.activation_code_pepper
ADMIN_BOOTSTRAP_PASSWORD← from AWS iviss/production/app-secrets.admin_bootstrap_password
```

---

### 3. ExternalSecret — Provider Keys

Pulls from AWS Secrets Manager secret `iviss/production/provider-keys` and merges into the same `iviss-secrets` K8s Secret.

The AWS secret has this JSON shape:
```json
{
  "twilio_account_sid": "...",
  "twilio_auth_token": "...",
  "twilio_from_number": "...",
  "vonage_api_key": "...",
  "vonage_api_secret": "...",
  "orange_client_id": "...",
  "orange_client_secret": "...",
  "orange_sender_number": "...",
  "resend_api_key": "...",
  "smtp_password": "..."
}
```

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: iviss-provider-keys
  namespace: iviss
  labels:
    app.kubernetes.io/part-of: iviss
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secretsmanager
    kind: ClusterSecretStore
  target:
    name: iviss-secrets
    creationPolicy: Merge
    template:
      type: Opaque
      data:
        VONAGE_API_KEY: "{{ .vonage_api_key }}"
        VONAGE_API_SECRET: "{{ .vonage_api_secret }}"
        TWILIO_ACCOUNT_SID: "{{ .twilio_account_sid }}"
        TWILIO_AUTH_TOKEN: "{{ .twilio_auth_token }}"
        TWILIO_FROM_NUMBER: "{{ .twilio_from_number }}"
        ORANGE_CLIENT_ID: "{{ .orange_client_id }}"
        ORANGE_CLIENT_SECRET: "{{ .orange_client_secret }}"
        ORANGE_SENDER_NUMBER: "{{ .orange_sender_number }}"
        RESEND_API_KEY: "{{ .resend_api_key }}"
        SMTP_PASSWORD: "{{ .smtp_password }}"
  data:
    - remoteRef:
        key: iviss/production/provider-keys
        property: vonage_api_key
      secretKey: vonage_api_key
    - remoteRef:
        key: iviss/production/provider-keys
        property: vonage_api_secret
      secretKey: vonage_api_secret
    - remoteRef:
        key: iviss/production/provider-keys
        property: twilio_account_sid
      secretKey: twilio_account_sid
    - remoteRef:
        key: iviss/production/provider-keys
        property: twilio_auth_token
      secretKey: twilio_auth_token
    - remoteRef:
        key: iviss/production/provider-keys
        property: twilio_from_number
      secretKey: twilio_from_number
    - remoteRef:
        key: iviss/production/provider-keys
        property: orange_client_id
      secretKey: orange_client_id
    - remoteRef:
        key: iviss/production/provider-keys
        property: orange_client_secret
      secretKey: orange_client_secret
    - remoteRef:
        key: iviss/production/provider-keys
        property: orange_sender_number
      secretKey: orange_sender_number
    - remoteRef:
        key: iviss/production/provider-keys
        property: resend_api_key
      secretKey: resend_api_key
    - remoteRef:
        key: iviss/production/provider-keys
        property: smtp_password
      secretKey: smtp_password
```

---

### 4. ExternalSecret — Cloudfront Origin Secret

Pulls from `iviss/production/cloudfront-origin-secret` (already in AWS). The Nginx Ingress uses this to validate the `X-Origin-Verify` header from CloudFront.

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: iviss-cloudfront-origin-secret
  namespace: iviss
  labels:
    app.kubernetes.io/part-of: iviss
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secretsmanager
    kind: ClusterSecretStore
  target:
    name: iviss-cloudfront-origin
    creationPolicy: Owner
  data:
    - remoteRef:
        key: iviss/production/cloudfront-origin-secret
      secretKey: origin-secret
```

---

### 5. CNPG Database Secrets

These are **not** pulled from AWS — the DB credentials are generated and managed by CNPG. Create them directly:

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
  password: REPLACE_ME    # generate: openssl rand -base64 32
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
  password: REPLACE_ME    # generate: openssl rand -base64 32
```

---

### 6. CNPG Cluster

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

---

### 7. ConfigMap — Non-sensitive App Config

These values are not secrets and are managed directly in the cluster. The dev team will update them via the Helm chart, but if you need a base ConfigMap before ArgoCD is set up:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: iviss-config
  namespace: iviss
  labels:
    app.kubernetes.io/part-of: iviss
data:
  SERVER_HOST: "0.0.0.0"
  SERVER_PORT: "3000"
  LOG_LEVEL: "info"
  ENVIRONMENT: "production"
  SHIFT_START_HOUR: "6"
  SHIFT_END_HOUR: "18"
  SMS_PROVIDER: "vonage"
  EMAIL_PROVIDER: "mock"
  OTP_VIA_EMAIL: "false"
  RESEND_FROM_EMAIL: "noreply@iviss.cloud"
  SMTP_HOST: "localhost"
  SMTP_PORT: "587"
  SMTP_USERNAME: "user"
  SQLX_OFFLINE: "true"
```

> **Note:** Once ArgoCD is running, this ConfigMap will be managed by the Helm chart.
> The values above are a baseline — the dev team can override them in `values-production.yaml`.

---

### 8. Static Secrets — Admin Bootstrap & External API

These contain non-cloud secrets that aren't in AWS Secrets Manager. Create them once:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: iviss-static-secrets
  namespace: iviss
  labels:
    app.kubernetes.io/part-of: iviss
type: Opaque
stringData:
  ADMIN_BOOTSTRAP_EMAIL: "admin@iviss.cloud"
  ADMIN_BOOTSTRAP_PHONE: "+237600000000"
  ADMIN_BOOTSTRAP_USERNAME: "admin"
  SMS_PROVIDER: "vonage"
  EMAIL_PROVIDER: "mock"
  EXTERNAL_API_BASE_URL: REPLACE_ME
  EXTERNAL_API_USERNAME: REPLACE_ME
  EXTERNAL_API_PASSWORD: REPLACE_ME
  EXTERNAL_API_LOCK_NDIA: REPLACE_ME
  EXTERNAL_API_KINDIA: REPLACE_ME
  EXTERNAL_API_USER: REPLACE_ME
  EXTERNAL_API_CLIENT: REPLACE_ME
  EXTERNAL_API_CTR: REPLACE_ME
  EXTERNAL_API_TLS_CERT_B64: REPLACE_ME
```

> **Note:** The dev team's backend Deployment references `iviss-secrets` for AWS-pulled secrets. These static values will be merged into the same secret or referenced separately depending on preference. The dev team will confirm.

---

## Verification

After applying **all** resources above, verify everything is ready:

```bash
# 1. Namespace exists
kubectl get namespace iviss

# 2. External Secrets are synced
kubectl get externalsecrets -n iviss
kubectl get secret iviss-secrets -n iviss -o yaml  # should show all keys populated
kubectl get secret iviss-cloudfront-origin -n iviss

# 3. CNPG cluster is ready
kubectl get cluster -n iviss iviss-postgres
kubectl wait cluster/iviss-postgres -n iviss --for=condition=Ready --timeout=300s
kubectl get pods -n iviss -l cnpg.io/cluster=iviss-postgres

# 4. Database is accessible (use password from iviss-postgres-app secret)
DB_PASS=$(kubectl get secret iviss-postgres-app -n iviss -o jsonpath='{.data.password}' | base64 -d)
kubectl exec -n iviss iviss-postgres-1 -- psql "postgresql://iviss_user:${DB_PASS}@localhost:5432/iviss_dev" -c "SELECT 1"

# 5. ServiceAccount exists
kubectl get sa iviss -n iviss

# 6. Nginx Ingress Controller is running
kubectl get pods -n ingress-nginx

# 7. ESO is syncing
kubectl get clustersecretstore aws-secretsmanager
```

---

## What This Enables

Once all resources above are created, the dev team can deploy **Wave 2** (the application) via ArgoCD without any admin permissions. The application pods will:

1. Read config from ConfigMap `iviss-config` (ArgoCD-managed)
2. Read secrets from `iviss-secrets` (ESO-pulled from AWS Secrets Manager)
3. Read static config from `iviss-static-secrets` (admin-created)
4. Connect to `iviss-postgres-rw:5432` (CNPG-managed)
5. Validate CloudFront origin via `iviss-cloudfront-origin` secret

Secret rotation is automatic — ESO refreshes every hour from AWS Secrets Manager.