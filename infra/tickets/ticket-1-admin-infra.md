# Ticket 1 — Infrastructure Bootstrap (Admin)

**Requester:** IVISS Dev Team  
**Cluster:** Hetzner K8s (k3s)  
**Namespace:** `iviss`  

> **After completing this ticket, the admin deploys the `iviss-database` ArgoCD app, which creates all secrets, ExternalSecrets, CNPG Cluster, and ServiceAccount automatically.**  
> **The dev then deploys `iviss-backend` and `iviss-frontend`.**

---

## Prerequisites — Manual Steps (One-Time)

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

# 5. ArgoCD
helm upgrade --install argocd argo-cd \
  --repo https://argoproj.github.io/argo-helm \
  --version 7.8.0 \
  --namespace argocd --create-namespace \
  --set server.service.type=LoadBalancer \
  --wait
```

### C. AWS Secrets Manager — Verify Content

These secrets already exist in AWS Secrets Manager (managed by Terraform with `ignore_changes`).  
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
  "smtp_password": "your_smtp_app_password_here",
  "smtp_from_email": "noreply@iviss.cloud",
  "resend_from_email": "noreply@iviss.cloud"
}
```

Verify:
```bash
aws secretsmanager get-secret-value \
  --secret-id iviss/prod/provider-keys \
  --region eu-west-1 \
  --query SecretString --output text | python3 -m json.tool
```

#### Secret 3: `iviss/prod/vehicle-api-keys`

```json
{
  "external_api_base_url": "https://api.example.com",
  "external_api_username": "auth_api_username",
  "external_api_password": "auth_api_password",
  "external_api_lock_ndia": "header_lock_ndia",
  "external_api_kindia": "header_kindia",
  "external_api_user": "header_user",
  "external_api_client": "header_client",
  "external_api_ctr": "header_ctr",
  "external_api_tls_cert_b64": "base64-encoded-pem-certificate"
}
```

Verify:
```bash
aws secretsmanager get-secret-value \
  --secret-id iviss/prod/vehicle-api-keys \
  --region eu-west-1 \
  --query SecretString --output text | python3 -m json.tool
```

### D. ClusterSecretStore — Connect ESO to AWS

The ESO pods need an IAM user with `secretsmanager:GetSecretValue` on:
- `arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/prod/app-secrets-*`
- `arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/prod/provider-keys-*`
- `arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/prod/vehicle-api-keys-*`

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

### E. Generate CNPG Passwords

The `iviss-database` Helm chart uses `randAlphaNum` to auto-generate passwords, but if you want to set specific values, override them in `values-production.yaml`:

```bash
# Optional: generate and note passwords for CNPG
POSTGRES_SUPER_PASSWORD=$(openssl rand -base64 32)
POSTGRES_APP_PASSWORD=$(openssl rand -base64 32)
echo "superuser: ${POSTGRES_SUPER_PASSWORD}"
echo "app: ${POSTGRES_APP_PASSWORD}"
```

Set them via Helm values:
```yaml
cnpg:
  superuserPassword: "<your-super-password>"
  appPassword: "<your-app-password>"
```

---

## Deploy ArgoCD Apps

### 1. Register the project and apps

```bash
kubectl apply -k argocd/
```

This creates:
- **AppProject** `iviss`
- **Application** `iviss-database` (sync-wave 0) — CNPG, secrets, ExternalSecrets
- **Application** `iviss-backend` (sync-wave 1) — API server, ConfigMap, Ingress
- **Application** `iviss-frontend` (sync-wave 1) — Dashboard, Ingress

### 2. Verify sync

```bash
argocd app sync iviss-database
argocd app sync iviss-backend
argocd app sync iviss-frontend
```

Or let ArgoCD auto-sync (prune + self-heal enabled).

---

## What the `iviss-database` ArgoCD App Creates

All resources below are in `charts/database/templates/` and managed by ArgoCD:

| Resource | File | Notes |
|---|---|---|
| ServiceAccount `iviss` | `serviceaccount.yaml` | sync-wave 0 |
| CNPG Secrets | `cnpg-secrets.yaml` | superuser + app passwords (auto-generated) |
| CNPG Cluster `iviss-postgres` | `cnpg-cluster.yaml` | 3 instances, 10Gi storage |
| ExternalSecret `iviss-app-secrets` | `externalsecret-app.yaml` | → `iviss-secrets` (Owner) |
| ExternalSecret `iviss-provider-keys` | `externalsecret-provider.yaml` | → `iviss-secrets` (Merge) |
| ExternalSecret `iviss-vehicle-api-keys` | `externalsecret-vehicle-api.yaml` | → `iviss-secrets` (Merge) |
| Secret `iviss-static-secrets` | `static-secrets.yaml` | admin bootstrap, provider choices |

---

## Complete Secret Mapping Reference

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
| `SMTP_FROM_EMAIL` | `iviss-secrets` | `SMTP_FROM_EMAIL` | `provider-keys.smtp_from_email` |
| `RESEND_FROM_EMAIL` | `iviss-secrets` | `RESEND_FROM_EMAIL` | `provider-keys.resend_from_email` |
| `EXTERNAL_API_BASE_URL` | `iviss-secrets` | `EXTERNAL_API_BASE_URL` | `vehicle-api-keys.external_api_base_url` |
| `EXTERNAL_API_USERNAME` | `iviss-secrets` | `EXTERNAL_API_USERNAME` | `vehicle-api-keys.external_api_username` |
| `EXTERNAL_API_PASSWORD` | `iviss-secrets` | `EXTERNAL_API_PASSWORD` | `vehicle-api-keys.external_api_password` |
| `EXTERNAL_API_LOCK_NDIA` | `iviss-secrets` | `EXTERNAL_API_LOCK_NDIA` | `vehicle-api-keys.external_api_lock_ndia` |
| `EXTERNAL_API_KINDIA` | `iviss-secrets` | `EXTERNAL_API_KINDIA` | `vehicle-api-keys.external_api_kindia` |
| `EXTERNAL_API_USER` | `iviss-secrets` | `EXTERNAL_API_USER` | `vehicle-api-keys.external_api_user` |
| `EXTERNAL_API_CLIENT` | `iviss-secrets` | `EXTERNAL_API_CLIENT` | `vehicle-api-keys.external_api_client` |
| `EXTERNAL_API_CTR` | `iviss-secrets` | `EXTERNAL_API_CTR` | `vehicle-api-keys.external_api_ctr` |
| `EXTERNAL_API_TLS_CERT_B64` | `iviss-secrets` | `EXTERNAL_API_TLS_CERT_B64` | `vehicle-api-keys.external_api_tls_cert_b64` |
| `ADMIN_BOOTSTRAP_EMAIL` | `iviss-static-secrets` | `ADMIN_BOOTSTRAP_EMAIL` | — |
| `ADMIN_BOOTSTRAP_PHONE` | `iviss-static-secrets` | `ADMIN_BOOTSTRAP_PHONE` | — |
| `ADMIN_BOOTSTRAP_USERNAME` | `iviss-static-secrets` | `ADMIN_BOOTSTRAP_USERNAME` | — |
| `SMS_PROVIDER` | `iviss-static-secrets` | `SMS_PROVIDER` | — |
| `EMAIL_PROVIDER` | `iviss-static-secrets` | `EMAIL_PROVIDER` | — |
| `OTP_VIA_EMAIL` | `iviss-static-secrets` | `OTP_VIA_EMAIL` | — |
| `DATABASE_URL` | — | Constructed by Helm | `postgres://iviss_user:<password>@iviss-postgres-rw:5432/iviss_dev` |

---

## Verification

```bash
# 1. Namespace exists
kubectl get namespace iviss

# 2. ArgoCD apps are synced
argocd app get iviss-database
argocd app get iviss-backend
argocd app get iviss-frontend

# 3. External Secrets are synced
kubectl get externalsecrets -n iviss
kubectl get secret iviss-secrets -n iviss -o jsonpath='{.data}' | jq 'keys'

# 4. CNPG cluster is ready
kubectl get cluster -n iviss iviss-postgres
kubectl wait cluster/iviss-postgres -n iviss --for=condition=Ready --timeout=300s

# 5. ServiceAccount exists
kubectl get sa iviss -n iviss

# 6. Static secrets exist
kubectl get secret iviss-static-secrets -n iviss

# 7. Nginx Ingress Controller is running
kubectl get pods -n ingress-nginx

# 8. ESO ClusterSecretStore is ready
kubectl get clustersecretstore aws-secretsmanager
```