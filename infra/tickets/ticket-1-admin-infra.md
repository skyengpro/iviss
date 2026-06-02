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

### B. Operators (install in this order)

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

### C. AWS Secrets Manager — Exact Content Required

These three secrets already exist in AWS Secrets Manager (managed by Terraform with `ignore_changes`).  
**Verify each one has the correct keys populated.** If any value is empty, seed it using the AWS Console or CLI.

#### Secret 1: `iviss/prod/app-secrets`

```json
{
}
```

Verify:
```bash
aws secretsmanager get-secret-value \
  --secret-id iviss/prod/app-secrets \
  --region eu-west-1 \
  --query SecretString --output text | python3 -m json.tool
```

#### Secret 2: `iviss/prod/provider-keys`

```json
{
  "twilio_account_sid": "ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
  "twilio_auth_token": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
  "twilio_from_number": "+1234567890",
  "vonage_api_key": "xxxxxxxx",
  "vonage_api_secret": "xxxxxxxxxxxxxxxxxxxxxxxx",
  "orange_client_id": "xxxxxxxxxxxxxxxxxxxxxxxx",
  "orange_client_secret": "xxxxxxxxxxxxxxxxxxxxxxxx",
  "orange_sender_number": "+237000000000",
  "resend_api_key": "re_xxxxxxxxxxxxxxxxxxxxxxxx",
  "smtp_password": "your_smtp_app_password_here"
}
```

Verify:
```bash
aws secretsmanager get-secret-value \
  --secret-id iviss/prod/provider-keys \
  --region eu-west-1 \
  --query SecretString --output text | python3 -m json.tool
```

#### Secret 3: `iviss/prod/cloudfront-origin-secret`

This is a plain string (not JSON) — the random 32-char password that CloudFront sends as the `X-Origin-Verify` header.

```bash
aws secretsmanager get-secret-value \
  --secret-id iviss/prod/cloudfront-origin-secret \
  --region eu-west-1 \
  --query SecretString --output text
```

### D. ClusterSecretStore — Connect ESO to AWS

The ESO pods need an IAM user with `secretsmanager:GetSecretValue` on:
- `arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/prod/app-secrets-*`
- `arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/prod/provider-keys-*`
- `arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/prod/cloudfront-origin-secret-*`

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: aws-eso-credentials
  namespace: external-secrets-system
type: Opaque
stringData:
  access-key-id: REPLACE_ME          # AWS IAM access key with SecretsManager read access
  secret-access-key: REPLACE_ME      # corresponding secret key
---
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

Pulls from AWS `iviss/prod/app-secrets` → creates K8s Secret `iviss-secrets` (with Owner policy).

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
  data:
    - remoteRef:
        key: iviss/prod/app-secrets
    - remoteRef:
        key: iviss/prod/app-secrets
    - remoteRef:
        key: iviss/prod/app-secrets
    - remoteRef:
        key: iviss/prod/app-secrets
```

**Resulting keys in `iviss-secrets`:**
| Key | Source |
|---|---|

---

### 3. ExternalSecret — Provider Keys

Pulls from AWS `iviss/prod/provider-keys` → merges into K8s Secret `iviss-secrets` (with Merge policy).

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
        key: iviss/prod/provider-keys
        property: vonage_api_key
      secretKey: vonage_api_key
    - remoteRef:
        key: iviss/prod/provider-keys
        property: vonage_api_secret
      secretKey: vonage_api_secret
    - remoteRef:
        key: iviss/prod/provider-keys
        property: twilio_account_sid
      secretKey: twilio_account_sid
    - remoteRef:
        key: iviss/prod/provider-keys
        property: twilio_auth_token
      secretKey: twilio_auth_token
    - remoteRef:
        key: iviss/prod/provider-keys
        property: twilio_from_number
      secretKey: twilio_from_number
    - remoteRef:
        key: iviss/prod/provider-keys
        property: orange_client_id
      secretKey: orange_client_id
    - remoteRef:
        key: iviss/prod/provider-keys
        property: orange_client_secret
      secretKey: orange_client_secret
    - remoteRef:
        key: iviss/prod/provider-keys
        property: orange_sender_number
      secretKey: orange_sender_number
    - remoteRef:
        key: iviss/prod/provider-keys
        property: resend_api_key
      secretKey: resend_api_key
    - remoteRef:
        key: iviss/prod/provider-keys
        property: smtp_password
      secretKey: smtp_password
```

**Resulting keys added to `iviss-secrets`:**
| Key | Source |
|---|---|
| `VONAGE_API_KEY` | AWS `provider-keys.vonage_api_key` |
| `VONAGE_API_SECRET` | AWS `provider-keys.vonage_api_secret` |
| `TWILIO_ACCOUNT_SID` | AWS `provider-keys.twilio_account_sid` |
| `TWILIO_AUTH_TOKEN` | AWS `provider-keys.twilio_auth_token` |
| `TWILIO_FROM_NUMBER` | AWS `provider-keys.twilio_from_number` |
| `ORANGE_CLIENT_ID` | AWS `provider-keys.orange_client_id` |
| `ORANGE_CLIENT_SECRET` | AWS `provider-keys.orange_client_secret` |
| `ORANGE_SENDER_NUMBER` | AWS `provider-keys.orange_sender_number` |
| `RESEND_API_KEY` | AWS `provider-keys.resend_api_key` |
| `SMTP_PASSWORD` | AWS `provider-keys.smtp_password` |

---

### 4. ExternalSecret — CloudFront Origin Secret

Pulls from `iviss/prod/cloudfront-origin-secret` → creates K8s Secret `iviss-cloudfront-origin`.

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
        key: iviss/prod/cloudfront-origin-secret
      secretKey: origin-secret
```

---

### 5. CNPG Database Secrets

Not from AWS — generated locally:

```bash
# Generate passwords
POSTGRES_SUPER_PASSWORD=$(openssl rand -base64 32)
POSTGRES_APP_PASSWORD=$(openssl rand -base64 32)

echo "Generated passwords:"
echo "  POSTGRES_SUPER_PASSWORD: ${POSTGRES_SUPER_PASSWORD}"
echo "  POSTGRES_APP_PASSWORD: ${POSTGRES_APP_PASSWORD}"
echo "  SAVE THESE — they won't be shown again."
```

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
  password: PASTE_GENERATED_POSTGRES_SUPER_PASSWORD_HERE
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
  password: PASTE_GENERATED_POSTGRES_APP_PASSWORD_HERE
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

> This will be managed by ArgoCD via the Helm chart once it's running.  
> Shown here for reference if you need to bootstrap before ArgoCD is configured.

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
  OTP_VIA_EMAIL: "true"
  RESEND_FROM_EMAIL: "noreply@iviss.cloud"
  SMTP_HOST: "localhost"
  SMTP_PORT: "587"
  SMTP_USERNAME: "user"
  SQLX_OFFLINE: "true"
```

---

### 8. Static Secrets — Admin Bootstrap & External API

These are values that are not stored in AWS Secrets Manager. Seed them once:

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
  OTP_VIA_EMAIL: "true"
  EXTERNAL_API_BASE_URL: "REPLACE_ME"
  EXTERNAL_API_USERNAME: "REPLACE_ME"
  EXTERNAL_API_PASSWORD: "REPLACE_ME"
  EXTERNAL_API_LOCK_NDIA: "REPLACE_ME"
  EXTERNAL_API_KINDIA: "REPLACE_ME"
  EXTERNAL_API_USER: "REPLACE_ME"
  EXTERNAL_API_CLIENT: "REPLACE_ME"
  EXTERNAL_API_CTR: "REPLACE_ME"
  EXTERNAL_API_TLS_CERT_B64: "REPLACE_ME"
```

> **Only `EXTERNAL_API_*` fields need actual values.** Everything else has defaults set.

---

### Complete Secret Mapping Reference

This table shows exactly where every env var the backend reads comes from:

| Backend Env Var | K8s Secret | Key | AWS Source |
|---|---|---|---|
| `VONAGE_API_KEY` | `iviss-secrets` | `VONAGE_API_KEY` | `provider-keys.vonage_api_key` |
| `VONAGE_API_SECRET` | `iviss-secrets` | `VONAGE_API_SECRET` | `provider-keys.vonage_api_secret` |
| `TWILIO_ACCOUNT_SID` | `iviss-secrets` | `TWILIO_ACCOUNT_SID` | `provider-keys.twilio_account_sid` |
| `TWILIO_AUTH_TOKEN` | `iviss-secrets` | `TWILIO_AUTH_TOKEN` | `provider-keys.twilio_auth_token` |
| `TWILIO_FROM_NUMBER` | `iviss-secrets` | `TWILIO_FROM_NUMBER` | `provider-keys.twilio_from_number` |
| `ORANGE_CLIENT_ID` | `iviss-secrets` | `ORANGE_CLIENT_ID` | `provider-keys.orange_client_id` |
| `ORANGE_CLIENT_SECRET` | `iviss-secrets` | `ORANGE_CLIENT_SECRET` | `provider-keys.orange_client_secret` |
| `ORANGE_SENDER_NUMBER` | `iviss-secrets` | `ORANGE_SENDER_NUMBER` | `provider-keys.orange_sender_number` |
| `RESEND_API_KEY` | `iviss-secrets` | `RESEND_API_KEY` | `provider-keys.resend_api_key` |
| `SMTP_PASSWORD` | `iviss-secrets` | `SMTP_PASSWORD` | `provider-keys.smtp_password` |
| `ADMIN_BOOTSTRAP_EMAIL` | `iviss-static-secrets` | `ADMIN_BOOTSTRAP_EMAIL` | — |
| `ADMIN_BOOTSTRAP_PHONE` | `iviss-static-secrets` | `ADMIN_BOOTSTRAP_PHONE` | — |
| `ADMIN_BOOTSTRAP_USERNAME` | `iviss-static-secrets` | `ADMIN_BOOTSTRAP_USERNAME` | — |
| `SMS_PROVIDER` | `iviss-static-secrets` | `SMS_PROVIDER` | — |
| `EMAIL_PROVIDER` | `iviss-static-secrets` | `EMAIL_PROVIDER` | — |
| `OTP_VIA_EMAIL` | `iviss-static-secrets` | `OTP_VIA_EMAIL` | — |
| `EXTERNAL_API_BASE_URL` | `iviss-static-secrets` | `EXTERNAL_API_BASE_URL` | — |
| `EXTERNAL_API_USERNAME` | `iviss-static-secrets` | `EXTERNAL_API_USERNAME` | — |
| `EXTERNAL_API_PASSWORD` | `iviss-static-secrets` | `EXTERNAL_API_PASSWORD` | — |
| `EXTERNAL_API_LOCK_NDIA` | `iviss-static-secrets` | `EXTERNAL_API_LOCK_NDIA` | — |
| `EXTERNAL_API_KINDIA` | `iviss-static-secrets` | `EXTERNAL_API_KINDIA` | — |
| `EXTERNAL_API_USER` | `iviss-static-secrets` | `EXTERNAL_API_USER` | — |
| `EXTERNAL_API_CLIENT` | `iviss-static-secrets` | `EXTERNAL_API_CLIENT` | — |
| `EXTERNAL_API_CTR` | `iviss-static-secrets` | `EXTERNAL_API_CTR` | — |
| `EXTERNAL_API_TLS_CERT_B64` | `iviss-static-secrets` | `EXTERNAL_API_TLS_CERT_B64` | — |
| `DATABASE_URL` | — | Constructed by Helm | `postgres://iviss_user:<password>@iviss-postgres-rw:5432/iviss_dev` |
| CloudFront origin | `iviss-cloudfront-origin` | `origin-secret` | `cloudfront-origin-secret` |

---

## Verification

After applying **all** resources above, verify everything is ready:

```bash
# 1. Namespace exists
kubectl get namespace iviss

# 2. External Secrets are synced (all should show Ready=True)
kubectl get externalsecrets -n iviss
kubectl get secret iviss-secrets -n iviss -o jsonpath='{.data}' | jq 'keys'
kubectl get secret iviss-cloudfront-origin -n iviss

# 3. CNPG cluster is ready
kubectl get cluster -n iviss iviss-postgres
kubectl wait cluster/iviss-postgres -n iviss --for=condition=Ready --timeout=300s
kubectl get pods -n iviss -l cnpg.io/cluster=iviss-postgres

# 4. Database is accessible
DB_PASS=$(kubectl get secret iviss-postgres-app -n iviss -o jsonpath='{.data.password}' | base64 -d)
kubectl exec -n iviss iviss-postgres-1 -- psql "postgresql://iviss_user:${DB_PASS}@localhost:5432/iviss_dev" -c "SELECT 1"

# 5. ServiceAccount exists
kubectl get sa iviss -n iviss

# 6. Static secrets exist
kubectl get secret iviss-static-secrets -n iviss

# 7. Nginx Ingress Controller is running
kubectl get pods -n ingress-nginx

# 8. ESO ClusterSecretStore is ready
kubectl get clustersecretstore aws-secretsmanager
```

---

## What This Enables

Once all resources above are created, the dev team can deploy **Wave 2** (the application) via ArgoCD. Secret rotation is automatic — ESO refreshes every hour from AWS Secrets Manager.