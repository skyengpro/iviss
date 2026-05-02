#!/bin/bash
set -e

# Cleanup sensitive files on exit
cleanup() {
  local exit_code=$?
  echo "🧹 Cleaning up sensitive files..."
  rm -f \
    "${VARS_FILE:-}" \
    "${ANSIBLE_DIR:-}/iviss-key.pem" \
    "${ANSIBLE_DIR:-}/ssh_config"
  exit $exit_code
}
trap cleanup EXIT

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TERRAFORM_DIR="$PROJECT_ROOT/infra/terraform"
ANSIBLE_DIR="$PROJECT_ROOT/infra/ansible"
VARS_FILE="$ANSIBLE_DIR/.deploy-vars.json"

# Auto-detect DOCKER_ORG from git remote if not set
if [ -z "$DOCKER_ORG" ]; then
    # Try to get the remote URL (handles https and ssh)
    GIT_REMOTE=$(git -C "$PROJECT_ROOT" remote get-url origin 2>/dev/null || echo "")
    if [[ $GIT_REMOTE =~ github.com[:/]([^/]+)/ ]]; then
        export DOCKER_ORG="${BASH_REMATCH[1]}"
        echo "🔍 Auto-detected DOCKER_ORG from Git: $DOCKER_ORG"
    fi
fi

# Validation: DOCKER_ORG is now required for the new docker-compose.yml
if [ -z "$DOCKER_ORG" ]; then
    echo "❌ FATAL ERROR: DOCKER_ORG is not set and could not be auto-detected."
    echo "Please set it manually: export DOCKER_ORG=your-org-name"
    exit 1
fi

# Load local .env if it exists
if [ -f "$PROJECT_ROOT/.env" ]; then
    echo "📄 Loading variables from local .env..."
    set -o allexport
    source "$PROJECT_ROOT/.env"
    set +o allexport
    
    # Show loaded critical variables (mask sensitive values)
    echo "✅ Loaded from .env:"
    echo "   - DOMAIN_NAME: ${DOMAIN_NAME:-'(not set)'}"
    echo "   - AWS_REGION: ${AWS_REGION:-'(not set, will use default)'}"
    echo "   - ENVIRONMENT: ${ENVIRONMENT:-'(not set, will default to production)'}"
    echo "   - USE_SECRETS_MANAGER: ${USE_SECRETS_MANAGER:-'(not set, will default to false)'}"
else
    echo "⚠️  No .env file found at $PROJECT_ROOT/.env"
    echo "   Using environment variables only"
fi

# Override with command line arguments if provided
export DOMAIN_NAME="${1:-$DOMAIN_NAME}"
export CERTBOT_EMAIL="${2:-$CERTBOT_EMAIL}"

# Automatic Version Detection (find latest tag if not provided)
if [ -z "$IMAGE_TAG" ]; then
    LATEST_TAG=$(git -C "$PROJECT_ROOT" describe --tags --abbrev=0 2>/dev/null || echo "")
    export IMAGE_TAG="${LATEST_TAG#v}"
fi
echo "📍 Deployment Version: ${IMAGE_TAG:-latest}"

echo "🚀 Starting IVISS Production Deployment..."

# Safety guard: prevent accidental custom-domain teardown
if [ -n "${DOMAIN_NAME:-}" ] && [ -z "${ROUTE53_ZONE_ID:-}" ]; then
    echo "❌ FATAL ERROR: DOMAIN_NAME is set but ROUTE53_ZONE_ID is empty."
    echo "   Refusing to run Terraform because this can destroy Route53/ACM custom-domain resources."
    echo "   Set ROUTE53_ZONE_ID and rerun."
    exit 1
fi

# 1. Terraform Initialization & Application
echo "📦 Provisioning infrastructure..."
cd "$TERRAFORM_DIR"
terraform init -reconfigure

# 1.5 Auto-healing: Import existing resources if missing from state
PROJECT="iviss"
ENV="production"
REGION="${AWS_REGION:-eu-west-1}"
LIGHTSAIL_INSTANCE_NAME="${PROJECT}-${ENV}-app-v2"

# Helper for conditional import
auto_import() {
    local resource=$1
    local name=$2
    local query=$3
    local check_cmd=$4

    echo "🔍 Checking for $name..."
    if eval "$check_cmd" >/dev/null 2>&1; then
        if ! terraform state list | grep -q "$resource"; then
            echo "⚠️  $name exists but is not in state. Importing..."
            terraform import -var="domain_name=${DOMAIN_NAME:-}" -var="certbot_email=${CERTBOT_EMAIL:-}" "$resource" "$name" || echo "Import of $name failed, proceeding..."
        fi
    fi
}

auto_import "aws_lightsail_key_pair.iviss_key" "${PROJECT}-${ENV}-key-v2" "Key Pair" "aws lightsail get-key-pair --key-pair-name ${PROJECT}-${ENV}-key-v2 --region $REGION"
auto_import "aws_lightsail_static_ip.iviss_ip" "${PROJECT}-${ENV}-ip-v2" "Static IP" "aws lightsail get-static-ip --static-ip-name ${PROJECT}-${ENV}-ip-v2 --region $REGION"
auto_import "aws_lightsail_instance.iviss_app" "${PROJECT}-${ENV}-app-v2" "Instance" "aws lightsail get-instance --instance-name ${PROJECT}-${ENV}-app-v2 --region $REGION"

echo ""
echo "📋 Terraform Configuration:"
echo "   - Domain Name: ${DOMAIN_NAME:-'(not set - will use CloudFront default)'}"
echo "   - Edge Lockdown: ${EDGE_LOCKDOWN_ENABLED:-true}"
echo "   - Route53 Zone ID: ${ROUTE53_ZONE_ID:-'(not set)'}"
echo ""

terraform apply -auto-approve \
  -var="auto_deploy=false" \
  -var="domain_name=${DOMAIN_NAME:-}" \
  -var="certbot_email=${CERTBOT_EMAIL:-}" \
  -var="route53_zone_id=${ROUTE53_ZONE_ID:-}" \
  -var="edge_lockdown_enabled=${EDGE_LOCKDOWN_ENABLED:-true}"

# Warn if edge lockdown is explicitly disabled in production
if [ "${EDGE_LOCKDOWN_ENABLED:-true}" = "false" ]; then
  echo "⚠️  WARNING: Edge lockdown is DISABLED. Lightsail will have public SSH and unrestricted HTTP access."
  echo "⚠️  This is NOT recommended for production environments."
fi

# 2. Extract Infrastructure Details
INSTANCE_IP=$(terraform output -raw instance_ip)
PRIVATE_KEY=$(terraform output -raw private_key)
CF_DISTRIBUTION_DOMAIN=$(terraform output -raw cloudfront_distribution_domain_name)
TF_IMAGE_TAG=$(terraform output -raw image_tag 2>/dev/null || echo "latest")

# Temporarily relaxed mode: keep SSH publicly reachable during setup/testing.
echo "⚠️  Opening Lightsail SSH publicly (0.0.0.0/0) for debugging/testing..."
aws lightsail open-instance-public-ports \
  --region "${REGION}" \
  --instance-name "${LIGHTSAIL_INSTANCE_NAME}" \
  --port-info fromPort=22,toPort=22,protocol=tcp,cidrs=0.0.0.0/0 >/dev/null

# 2.5 Resolve Deployment Version (Override order: ENV > Terraform > Git Tag)
if [ -n "$IMAGE_TAG" ]; then
    echo "📍 Using IMAGE_TAG from environment: $IMAGE_TAG"
elif [ "$TF_IMAGE_TAG" != "latest" ]; then
    export IMAGE_TAG="$TF_IMAGE_TAG"
    echo "📍 Using IMAGE_TAG from Terraform: $IMAGE_TAG"
else
    LATEST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    export IMAGE_TAG="${LATEST_TAG#v}"
    echo "📍 Using IMAGE_TAG from Git: ${IMAGE_TAG:-latest}"
fi
echo "$PRIVATE_KEY" > "$ANSIBLE_DIR/iviss-key.pem"
chmod 600 "$ANSIBLE_DIR/iviss-key.pem"

# 4. Generate Ansible Inventory
echo "📝 Generating Ansible inventory..."
cat <<EOF > "$ANSIBLE_DIR/ssh_config"
Host lightsail-public
  HostName $INSTANCE_IP
  User ubuntu
  IdentityFile $ANSIBLE_DIR/iviss-key.pem
  IdentitiesOnly yes
  StrictHostKeyChecking no
  UserKnownHostsFile /dev/null
EOF

cat <<EOF > "$ANSIBLE_DIR/inventory.ini"
[iviss_prod]
lightsail-public ansible_host=$INSTANCE_IP ansible_user=ubuntu ansible_ssh_private_key_file=$ANSIBLE_DIR/iviss-key.pem ansible_ssh_common_args='-F $ANSIBLE_DIR/ssh_config'
EOF

# 5. Wait for SSH
echo "⚙️ Configuring server and deploying application..."
cd "$ANSIBLE_DIR"
echo "Waiting for direct SSH to be ready on $INSTANCE_IP..."

# Add timeout to prevent infinite waiting
SSH_TIMEOUT=300  # 5 minutes
SSH_START_TIME=$(date +%s)
SSH_ATTEMPT=0

while ! ssh -F "$ANSIBLE_DIR/ssh_config" -o BatchMode=yes -o ConnectTimeout=10 lightsail-public "true" >/dev/null 2>&1; do
  SSH_ELAPSED=$(($(date +%s) - SSH_START_TIME))
  SSH_ATTEMPT=$((SSH_ATTEMPT + 1))
  
  if [ $SSH_ELAPSED -ge $SSH_TIMEOUT ]; then
    echo ""
    echo "❌ ERROR: SSH connection timed out after ${SSH_TIMEOUT} seconds (${SSH_ATTEMPT} attempts)"
    echo ""
    echo "Troubleshooting steps:"
    echo "1. Check if Lightsail instance is running:"
    echo "   aws lightsail get-instances --region ${AWS_REGION:-eu-west-1}"
    echo ""
    echo "3. Test SSH manually:"
    echo "   ssh -F $ANSIBLE_DIR/ssh_config -v lightsail-public"
    echo ""
    exit 1
  fi
  
  if [ $((SSH_ATTEMPT % 12)) -eq 0 ]; then
    echo "Still waiting for SSH... (${SSH_ELAPSED}s elapsed)"
  fi
  
  sleep 5
done

echo "✓ SSH connection established successfully"

# 6. Build JSON vars file
if [ "${USE_SECRETS_MANAGER:-false}" = "true" ]; then
  echo "🔐 Fetching secrets from AWS Secrets Manager..."
  # Ensure we use 'production' for AWS paths even if .env says 'local'
  ENVIRONMENT_NAME="${ENVIRONMENT:-production}"
  if [ "$ENVIRONMENT_NAME" = "local" ]; then ENVIRONMENT_NAME="production"; fi
  REGION="${AWS_REGION:-eu-west-1}"
  if [ -z "$REGION" ]; then REGION="eu-west-1"; fi
  SECRET_ID="iviss/${ENVIRONMENT_NAME}/app-secrets"
  echo "📍 Region: $REGION"
  echo "📍 ID: $SECRET_ID"
  # Fetch secrets
  export APP_SECRETS=$(aws secretsmanager get-secret-value \
    --secret-id "iviss/${ENVIRONMENT_NAME}/app-secrets" \
    --query SecretString --output text --region $REGION)
  
  export PROVIDER_KEYS=$(aws secretsmanager get-secret-value \
    --secret-id "iviss/${ENVIRONMENT_NAME}/provider-keys" \
    --query SecretString --output text --region $REGION)

  export CLOUDFRONT_ORIGIN_SECRET=$(aws secretsmanager get-secret-value \
    --secret-id "iviss/${ENVIRONMENT_NAME}/cloudfront-origin-secret" \
    --query SecretString --output text --region $REGION)
  
  python3 -c "
import json, os, sys

try:
    # Read from environment to safely handle newlines/quotes
    app_raw = os.environ.get('APP_SECRETS', '{}')
    providers_raw = os.environ.get('PROVIDER_KEYS', '{}')
    
    app = json.loads(app_raw)
    providers = json.loads(providers_raw)
except Exception as e:
    print(f'❌ FATAL ERROR: Failed to parse JSON from AWS Secrets: {e}', file=sys.stderr)
    sys.exit(1)

domain = os.environ.get('DOMAIN_NAME', '')
instance_ip = '${INSTANCE_IP}'
cloudfront_domain = '${CF_DISTRIBUTION_DOMAIN}'
docker_user = os.environ.get('DOCKER_USERNAME', '')
docker_pass = app.get('docker_password', '')

print('Building Ansible vars with:')
print('   - Domain: ' + (domain or '(using CloudFront domain)'))
print('   - Instance IP: ' + instance_ip)
print('   - CloudFront Domain: ' + cloudfront_domain)

# Mandatory Validation
if not docker_pass:
    print('❌ FATAL ERROR: docker_password is empty in AWS Secrets Manager!', file=sys.stderr)
    sys.exit(1)
if not docker_user:
    print('❌ FATAL ERROR: DOCKER_USERNAME environment variable is not set!', file=sys.stderr)
    sys.exit(1)

vars = {
    # Secrets from AWS
    'jwt_private_key_pem': app.get('jwt_private_key_pem', ''),
    'jwt_public_key_pem': app.get('jwt_public_key_pem', ''),
    'activation_code_pepper': app.get('activation_code_pepper', ''),
    'db_password': app.get('db_password', ''),
    'admin_bootstrap_password': app.get('admin_bootstrap_password', ''),
    'docker_password': docker_pass,
    'docker_username': docker_user,
    'docker_org': os.environ.get('DOCKER_ORG', ''),
    
    'twilio_account_sid': providers.get('twilio_account_sid', ''),
    'twilio_auth_token': providers.get('twilio_auth_token', ''),
    'twilio_from_number': providers.get('twilio_from_number', ''),
    'vonage_api_key': providers.get('vonage_api_key', ''),
    'vonage_api_secret': providers.get('vonage_api_secret', ''),
    'orange_client_id': providers.get('orange_client_id', ''),
    'orange_client_secret': providers.get('orange_client_secret', ''),
    'orange_sender_number': providers.get('orange_sender_number', ''),
    'resend_api_key': providers.get('resend_api_key', ''),
    'smtp_password': providers.get('smtp_password', ''),
    
    # Non-secrets from Environment
    'domain_name': domain,
    'certbot_email': os.environ.get('CERTBOT_EMAIL', ''),
    'db_user': os.environ.get('POSTGRES_USER', ''),
    'db_name': os.environ.get('POSTGRES_DB', ''),
    'vite_api_url': f'https://{domain}' if domain else f'https://{cloudfront_domain}',
    'iviss_env': os.environ.get('ENVIRONMENT') or 'production',
    'log_level': os.environ.get('LOG_LEVEL', 'info'),
    'shift_start_hour': os.environ.get('SHIFT_START_HOUR', '6'),
    'shift_end_hour': os.environ.get('SHIFT_END_HOUR', '18'),
    'admin_bootstrap_email': os.environ.get('ADMIN_BOOTSTRAP_EMAIL', ''),
    'admin_bootstrap_phone': os.environ.get('ADMIN_BOOTSTRAP_PHONE', ''),
    'admin_bootstrap_username': os.environ.get('ADMIN_BOOTSTRAP_USERNAME', ''),
    'sms_provider': os.environ.get('SMS_PROVIDER') or 'mock',
    'email_provider': os.environ.get('EMAIL_PROVIDER') or 'mock',
    'resend_from_email': os.environ.get('RESEND_FROM_EMAIL', ''),
    'smtp_host': os.environ.get('SMTP_HOST', ''),
    'smtp_port': os.environ.get('SMTP_PORT', ''),
    'smtp_username': os.environ.get('SMTP_USERNAME', ''),
    'smtp_from_email': os.environ.get('SMTP_FROM_EMAIL', ''),
    'image_tag': os.environ.get('IMAGE_TAG', 'latest'),
    'cloudfront_origin_secret': os.environ.get('CLOUDFRONT_ORIGIN_SECRET', ''),
    'cloudfront_enabled': os.environ.get('EDGE_LOCKDOWN_ENABLED', 'false').lower(),
    'deploy_ssh_cidr': os.environ.get('DEPLOY_SSH_CIDR', ''),
}

with open('$VARS_FILE', 'w') as f:
    json.dump(vars, f)
" || exit 1

else
  echo "📄 Using local environment variables..."
  python3 -c "
import json, os

def get_pem(env_var, file_path):
    val = os.environ.get(env_var, '')
    if not val and os.path.exists(file_path):
        val = open(file_path).read()
    return val.strip().replace(chr(10), '\\\\n')

domain = os.environ.get('DOMAIN_NAME', '')
instance_ip = '${INSTANCE_IP}'
cloudfront_domain = '${CF_DISTRIBUTION_DOMAIN}'

print('Building Ansible vars with:')
print('   - Domain: ' + (domain or '(using CloudFront domain)'))
print('   - Instance IP: ' + instance_ip)
print('   - CloudFront Domain: ' + cloudfront_domain)

vars = {
    'domain_name': domain,
    'certbot_email': os.environ.get('CERTBOT_EMAIL', ''),
    'db_password': os.environ.get('POSTGRES_PASSWORD', ''),
    'db_user': os.environ.get('POSTGRES_USER', ''),
    'db_name': os.environ.get('POSTGRES_DB', ''),
    'vite_api_url': f'https://{domain}' if domain else f'https://{cloudfront_domain}',
    'jwt_private_key_pem': get_pem('JWT_PRIVATE_KEY_PEM', '$PROJECT_ROOT/jwt-private.pem'),
    'jwt_public_key_pem': get_pem('JWT_PUBLIC_KEY_PEM', '$PROJECT_ROOT/jwt-public.pem'),
    'activation_code_pepper': os.environ.get('ACTIVATION_CODE_PEPPER', ''),
    'iviss_env': os.environ.get('ENVIRONMENT') or 'production',
    'log_level': os.environ.get('LOG_LEVEL', 'info'),
    'shift_start_hour': os.environ.get('SHIFT_START_HOUR', '6'),
    'shift_end_hour': os.environ.get('SHIFT_END_HOUR', '18'),
    'admin_bootstrap_email': os.environ.get('ADMIN_BOOTSTRAP_EMAIL', ''),
    'admin_bootstrap_password': os.environ.get('ADMIN_BOOTSTRAP_PASSWORD', ''),
    'admin_bootstrap_phone': os.environ.get('ADMIN_BOOTSTRAP_PHONE', ''),
    'admin_bootstrap_username': os.environ.get('ADMIN_BOOTSTRAP_USERNAME', ''),
    'twilio_account_sid': os.environ.get('TWILIO_ACCOUNT_SID', ''),
    'twilio_auth_token': os.environ.get('TWILIO_AUTH_TOKEN', ''),
    'twilio_from_number': os.environ.get('TWILIO_FROM_NUMBER', ''),
    'orange_client_id': os.environ.get('ORANGE_CLIENT_ID', ''),
    'orange_client_secret': os.environ.get('ORANGE_CLIENT_SECRET', ''),
    'orange_sender_number': os.environ.get('ORANGE_SENDER_NUMBER', ''),
    'sms_provider': os.environ.get('SMS_PROVIDER', ''),
    'vonage_api_key': os.environ.get('VONAGE_API_KEY', ''),
    'vonage_api_secret': os.environ.get('VONAGE_API_SECRET', ''),
    'email_provider': os.environ.get('EMAIL_PROVIDER', ''),
    'resend_api_key': os.environ.get('RESEND_API_KEY', ''),
    'resend_from_email': os.environ.get('RESEND_FROM_EMAIL', ''),
    'smtp_host': os.environ.get('SMTP_HOST', ''),
    'smtp_port': os.environ.get('SMTP_PORT', ''),
    'smtp_username': os.environ.get('SMTP_USERNAME', ''),
    'smtp_password': os.environ.get('SMTP_PASSWORD', ''),
    'smtp_from_email': os.environ.get('SMTP_FROM_EMAIL', ''),
    'docker_username': os.environ.get('DOCKER_USERNAME', ''),
    'docker_password': os.environ.get('DOCKER_PASSWORD', ''),
    'docker_org': os.environ.get('DOCKER_ORG', ''),
    'image_tag': os.environ.get('IMAGE_TAG', 'latest'),
    'cloudfront_origin_secret': os.environ.get('CLOUDFRONT_ORIGIN_SECRET', ''),
    'cloudfront_enabled': os.environ.get('EDGE_LOCKDOWN_ENABLED', 'false').lower(),
    'deploy_ssh_cidr': os.environ.get('DEPLOY_SSH_CIDR', ''),
}

with open('$VARS_FILE', 'w') as f:
    json.dump(vars, f)

# Self-verification
with open('$VARS_FILE', 'r') as f:
    verify = json.load(f)
    print(f\"CONFIRM: JSON file has Private Key? {'YES' if verify.get('jwt_private_key_pem') else 'NO'}\")
    print(f\"CONFIRM: JSON Private Key length in file: {len(verify.get('jwt_private_key_pem', ''))}\")
"
fi

ansible-playbook -i inventory.ini playbook.yml --extra-vars "@$VARS_FILE"

if [ $? -eq 0 ]; then
  echo ""
  echo "=========================================="
  echo "  Deployment Completed Successfully!"
  echo "=========================================="
  echo ""
  echo "Infrastructure:"
  echo "  - CloudFront: https://${DOMAIN_NAME:-yourdomain.com}"
  echo "  - Lightsail:  $INSTANCE_IP"
  echo ""
  echo "Next Steps:"
  echo "  1. Test the application:"
  echo "     curl https://${DOMAIN_NAME:-yourdomain.com}/api/v1/health"
  echo ""
  echo "  2. SSH to the server:"
  echo "     ssh -F $ANSIBLE_DIR/ssh_config lightsail-public"
  echo ""
  echo "  3. Check application logs:"
  echo "     ssh -F $ANSIBLE_DIR/ssh_config lightsail-public 'cd /opt/iviss && docker compose logs -f'"
  echo ""
  echo "=========================================="
else
  echo ""
  echo "❌ ERROR: Ansible playbook failed!"
  echo "Check the output above for error details."
  exit 1
fi

# Clean up sensitive vars file (Disabled for debugging)
# rm -f "$VARS_FILE"

echo "✅ Deployment complete!"
