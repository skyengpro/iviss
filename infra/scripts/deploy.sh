#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# IVISS Deploy Script (K8s + ArgoCD version)
#
# Manages the AWS edge layer (CloudFront, WAF, ACM, Route53).
# Application deployment is handled by ArgoCD via GitOps.
#
# Prerequisites:
#   - K8s cluster provisioned on Hetzner (infra/terraform/hetzner/)
#   - ArgoCD installed on the cluster
#   - Nginx Ingress Controller running with a LoadBalancer IP
#   - DNS record pointing to the LoadBalancer IP (K8S_ORIGIN_HOSTNAME)
# ============================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TERRAFORM_DIR="$SCRIPT_DIR/../terraform"

# --- Load .env if present ---
if [ -f "$SCRIPT_DIR/../../.env" ]; then
  set -a
  source "$SCRIPT_DIR/../../.env"
  set +a
fi

# --- Required vars ---
: "${DOMAIN_NAME:=}"
: "${ROUTE53_ZONE_ID:=}"
: "${K8S_ORIGIN_HOSTNAME:=}"
: "${CERTBOT_EMAIL:=admin@iviss.local}"
: "${AWS_REGION:=eu-west-1}"

# --- Safety guard ---
if [ -n "$DOMAIN_NAME" ] && [ -z "${ROUTE53_ZONE_ID:-}" ]; then
  echo "ERROR: DOMAIN_NAME is set but ROUTE53_ZONE_ID is empty."
  echo "Set ROUTE53_ZONE_ID to avoid accidental DNS/ACM teardown."
  exit 1
fi

# --- Terraform Apply (AWS Edge Layer) ---
echo "==> Deploying AWS Edge layer (CloudFront + WAF + ACM + Route53)..."
cd "$TERRAFORM_DIR"
terraform init
terraform apply \
  -auto-approve \
  -var="domain_name=$DOMAIN_NAME" \
  -var="route53_zone_id=$ROUTE53_ZONE_ID" \
  -var="k8s_origin_hostname=$K8S_ORIGIN_HOSTNAME" \
  -var="certbot_email=$CERTBOT_EMAIL" \
  -var="aws_region=$AWS_REGION"

echo ""
echo "==> Deployment complete!"
echo "    CloudFront: $(terraform output -raw cloudfront_distribution_domain_name 2>/dev/null || echo 'N/A')"
echo ""
echo "Note: Application rollout is handled by ArgoCD."
echo "Push to the main branch to trigger a new sync."
