#!/bin/bash
set -e

# Cleanup sensitive files on exit
cleanup() {
  local exit_code=$?
  echo "🧹 Cleaning up sensitive files..."
  rm -f "${VARS_FILE:-}" "${ANSIBLE_DIR:-}/iviss-key.pem"
  exit $exit_code
}
trap cleanup EXIT

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TERRAFORM_DIR="$PROJECT_ROOT/infra/terraform"
ANSIBLE_DIR="$PROJECT_ROOT/infra/ansible"
VARS_FILE="$ANSIBLE_DIR/.deploy-vars.json"

# Load local .env if it exists
if [ -f "$PROJECT_ROOT/.env" ]; then
    echo "📄 Loading variables from local .env..."
    set -o allexport
    source "$PROJECT_ROOT/.env"
    set +o allexport
fi

# Override with command line arguments if provided
export DOMAIN_NAME="${1:-$DOMAIN_NAME}"
export CERTBOT_EMAIL="${2:-$CERTBOT_EMAIL}"

# Automatic Version Detection (find latest tag if not provided)
if [ -z "$IMAGE_TAG" ]; then
    LATEST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    # Remove 'v' prefix if it exists (e.g., v1.0.0 -> 1.0.0)
    export IMAGE_TAG="${LATEST_TAG#v}"
fi
echo "📍 Deployment Version: ${IMAGE_TAG:-latest}"

echo "🚀 Starting IVISS Production Deployment..."

# 1. Terraform Initialization & Application
echo "📦 Provisioning infrastructure..."
cd "$TERRAFORM_DIR"
terraform init -reconfigure

# 1.5 Auto-healing: Import existing resources if missing from state
PROJECT="iviss"
ENV="production"
REGION="eu-west-1"

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

terraform apply -auto-approve \
  -var="auto_deploy=false" \
  -var="domain_name=${DOMAIN_NAME:-}" \
  -var="certbot_email=${CERTBOT_EMAIL:-}"

# 2. Extract Infrastructure Details
INSTANCE_IP=$(terraform output -raw instance_ip)
PRIVATE_KEY=$(terraform output -raw private_key)
TF_IMAGE_TAG=$(terraform output -raw image_tag 2>/dev/null || echo "latest")

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
cat <<EOF > "$ANSIBLE_DIR/inventory.ini"
[iviss_prod]
$INSTANCE_IP ansible_user=ubuntu ansible_ssh_private_key_file=./iviss-key.pem ansible_ssh_common_args='-o StrictHostKeyChecking=no -o IdentitiesOnly=yes'
EOF

# 5. Wait for SSH
echo "⚙️ Configuring server and deploying application..."
cd "$ANSIBLE_DIR"
echo "Waiting for SSH to be ready on $INSTANCE_IP..."
until nc -zvw5 $INSTANCE_IP 22; do
  sleep 5
done

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
docker_user = os.environ.get('DOCKER_USERNAME', '')
docker_pass = app.get('docker_password', '')

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
    'vite_api_url': f'https://{domain}' if domain else f'http://{instance_ip}:3000',
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

vars = {
    'domain_name': domain,
    'certbot_email': os.environ.get('CERTBOT_EMAIL', ''),
    'db_password': os.environ.get('POSTGRES_PASSWORD', ''),
    'db_user': os.environ.get('POSTGRES_USER', ''),
    'db_name': os.environ.get('POSTGRES_DB', ''),
    'vite_api_url': f'https://{domain}' if domain else f'http://{instance_ip}:3000',
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

# Clean up sensitive vars file (Disabled for debugging)
# rm -f "$VARS_FILE"

echo "✅ Deployment complete!"
