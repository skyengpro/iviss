# Ticket 1 — Infrastructure Bootstrap (Admin)

**Requester:** IVISS Dev Team  
**Cluster:** Hetzner K8s (k3s)  
**Namespace:** `iviss`  

> **The admin creates all infrastructure manually. The dev team only manages the application via ArgoCD (`iviss-backend` + `iviss-frontend`).**

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

## Step 2 — AWS Secrets Manager — Verify Content

These secrets exist in AWS (created by Terraform). **Verify they have the correct keys populated.**

### Secret 1: `iviss/prod/app-secrets`

```bash
aws secretsmanager get-secret-value \
  --secret-id iviss/prod/app-secrets \
  --region eu-west-1 \
  --query SecretString --output text | python3 -m json.tool
```


### Secret 2: `iviss/prod/provider-keys`

```bash
aws secretsmanager get-secret-value \
  --secret-id iviss/prod/provider-keys \
  --region eu-west-1 \
  --query SecretString --output text | python3 -m json.tool
```

Expected keys: `twilio_account_sid`, `twilio_auth_token`, `twilio_from_number`, `vonage_api_key`, `vonage_api_secret`, `orange_client_id`, `orange_client_secret`, `orange_sender_number`, `resend_api_key`, `smtp_password`, `smtp_from_email`, `resend_from_email`

### Secret 3: `iviss/prod/vehicle-api-keys`

```bash
aws secretsmanager get-secret-value \
  --secret-id iviss/prod/vehicle-api-keys \
  --region eu-west-1 \
  --query SecretString --output text | python3 -m json.tool
```

Expected keys: `external_api_base_url`, `external_api_username`, `external_api_password`, `external_api_lock_ndia`, `external_api_kindia`, `external_api_user`, `external_api_client`, `external_api_ctr`, `external_api_tls_cert_b64`

---

## Step 3 — ClusterSecretStore (connect ESO to AWS)

The ESO needs IAM credentials with `secretsmanager:GetSecretValue` on:
- `arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/prod/app-secrets-*`
- `arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/prod/provider-keys-*`
- `arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/prod/vehicle-api-keys-*`

```bash
kubectl apply -f infra/manifests/cluster-secret-store.yaml
```

---

## Step 4 — Apply Infrastructure Manifests

All files are in `infra/manifests/`. Apply them in order:

```bash
# 1. ServiceAccount
kubectl apply -f infra/manifests/serviceaccount.yaml

# 2. CNPG passwords (edit first — set real passwords!)
#    Generate: openssl rand -base64 32
#    Replace REPLACE_ME_GENERATED_SUPERUSER_PASSWORD and REPLACE_ME_GENERATED_APP_PASSWORD
kubectl apply -f infra/manifests/cnpg-secrets.yaml

# 3. CNPG Cluster (depends on secrets above)
kubectl apply -f infra/manifests/cnpg-cluster.yaml

# 4. ExternalSecrets (depends on ClusterSecretStore)
kubectl apply -f infra/manifests/externalsecret-app.yaml
kubectl apply -f infra/manifests/externalsecret-provider.yaml
kubectl apply -f infra/manifests/externalsecret-vehicle-api.yaml

# 5. Static secrets
kubectl apply -f infra/manifests/static-secrets.yaml
```

---

## Step 5 — Verify

```bash
# CNPG cluster ready
kubectl get cluster -n iviss iviss-postgres
kubectl wait cluster/iviss-postgres -n iviss --for=condition=Ready --timeout=300s

# External Secrets synced
kubectl get externalsecrets -n iviss                 # all should show Ready=True
kubectl get secret iviss-secrets -n iviss -o jsonpath='{.data}' | jq 'keys'

# Static secrets exist
kubectl get secret iviss-static-secrets -n iviss

# ServiceAccount exists
kubectl get sa iviss -n iviss

# Database accessible
DB_PASS=$(kubectl get secret iviss-postgres-app -n iviss -o jsonpath='{.data.password}' | base64 -d)
kubectl exec -n iviss iviss-postgres-1 -- psql "postgresql://iviss_user:${DB_PASS}@localhost:5432/iviss_dev" -c "SELECT 1"
```

---

## Step 6 — Register ArgoCD Apps

```bash
kubectl apply -k argocd/
```

This creates:
- **AppProject** `iviss`
- **Application** `iviss-backend` (sync-wave 1)
- **Application** `iviss-frontend` (sync-wave 1)

ArgoCD will auto-sync the application deployments.

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