#!/bin/bash
set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TERRAFORM_DIR="$PROJECT_ROOT/infra/terraform"
ANSIBLE_DIR="$PROJECT_ROOT/infra/ansible"
SCRIPTS_DIR="$PROJECT_ROOT/infra/scripts"
DOMAIN=${1:-""}
EMAIL=${2:-"admin@iviss.local"}

# Load local .env if it exists
if [ -f "$PROJECT_ROOT/.env" ]; then
    echo "📄 Loading variables from .env..."
    set -o allexport
    source "$PROJECT_ROOT/.env"
    set +o allexport
fi

echo "🚀 Starting IVISS Production Deployment..."

# 1. Terraform Initialization & Application
echo "📦 Provisioning infrastructure..."
cd "$TERRAFORM_DIR"
terraform init -reconfigure
terraform apply -auto-approve \
  -var="auto_deploy=false" \
  -var="domain_name=$DOMAIN" \
  -var="certbot_email=$EMAIL"

# 2. Extract Infrastructure Details
INSTANCE_IP=$(terraform output -raw instance_ip)
PRIVATE_KEY=$(terraform output -raw private_key)

# Generate a DB password if not provided
DB_PASSWORD=${DB_PASSWORD:-$(openssl rand -base64 16)}

# 3. Save SSH Key
echo "$PRIVATE_KEY" > "$ANSIBLE_DIR/iviss-key.pem"
chmod 600 "$ANSIBLE_DIR/iviss-key.pem"

# 4. Generate Ansible Inventory
echo "📝 Generating Ansible inventory..."
cat <<INVENTORY > "$ANSIBLE_DIR/inventory.ini"
[iviss_prod]
$INSTANCE_IP ansible_user=ubuntu ansible_ssh_private_key_file=./iviss-key.pem ansible_ssh_common_args='-o StrictHostKeyChecking=no'
INVENTORY

# 5. Run Ansible Playbook
echo "⚙️ Configuring server and deploying application..."
cd "$ANSIBLE_DIR"

# Wait for SSH to be ready
echo "Waiting for SSH to be ready on $INSTANCE_IP..."
until nc -zvw5 $INSTANCE_IP 22; do
  sleep 5
done

# 6. Build JSON vars file (handles multi-line PEM keys safely)
VARS_FILE="$ANSIBLE_DIR/.deploy-vars.json"
python3 -c "
import json, os, base64

def get_pem(env_var, file_path):
    val = os.environ.get(env_var, '')
    if not val and os.path.exists(file_path):
        try:
            with open(file_path, 'r') as f:
                val = f.read()
        except:
            pass
    if not val:
        return ''
    # Normalize all possible escapes (e.g. \\n, \\r, etc.)
    try:
        # This handles quadruple escapes from CI/CD environments
        cleaned = val.encode('utf-8').decode('unicode_escape').strip()
    except:
        cleaned = val.strip().replace('\\n', '\n')
    
    return base64.b64encode(cleaned.encode('utf-8')).decode('utf-8')

priv_key = get_pem('JWT_PRIVATE_KEY_PEM', '$PROJECT_ROOT/jwt-private.pem')
pub_key = get_pem('JWT_PUBLIC_KEY_PEM', '$PROJECT_ROOT/jwt-public.pem')

print(f'DEBUG: JWT Private Key detected (length: {len(priv_key)})')
print(f'DEBUG: JWT Public Key detected (length: {len(pub_key)})')

if not priv_key or len(priv_key) < 20:
    print('❌ ERROR: JWT_PRIVATE_KEY_PEM is missing or invalid!')
    exit(1)

vars = {
    'db_password': os.environ.get('POSTGRES_PASSWORD', os.environ.get('DB_PASSWORD', '$DB_PASSWORD')),
    'db_user': os.environ.get('POSTGRES_USER', 'iviss_user'),
    'db_name': os.environ.get('POSTGRES_DB', 'iviss_dev'),
    'vite_api_url': f'https://${DOMAIN}/api' if '${DOMAIN}' else f'http://${INSTANCE_IP}:3000',
    'jwt_secret': os.environ.get('JWT_SECRET', ''),
    'iviss_jwt_private_key_base64': priv_key,
    'iviss_jwt_public_key_base64': pub_key,
    'activation_code_pepper': os.environ.get('ACTIVATION_CODE_PEPPER', ''),
    'environment': os.environ.get('ENVIRONMENT', 'production'),
    'log_level': os.environ.get('LOG_LEVEL', 'info'),
    'shift_start_hour': os.environ.get('SHIFT_START_HOUR', '6'),
    'shift_end_hour': os.environ.get('SHIFT_END_HOUR', '18'),
    'admin_bootstrap_email': os.environ.get('ADMIN_BOOTSTRAP_EMAIL', 'admin@iviss.local'),
    'admin_bootstrap_password': os.environ.get('ADMIN_BOOTSTRAP_PASSWORD', ''),
    'admin_bootstrap_phone': os.environ.get('ADMIN_BOOTSTRAP_PHONE', ''),
    'admin_bootstrap_username': os.environ.get('ADMIN_BOOTSTRAP_USERNAME', 'admin'),
    'twilio_account_sid': os.environ.get('TWILIO_ACCOUNT_SID', 'mock'),
    'twilio_auth_token': os.environ.get('TWILIO_AUTH_TOKEN', 'mock'),
    'twilio_from_number': os.environ.get('TWILIO_FROM_NUMBER', 'mock'),
    'sms_provider': os.environ.get('SMS_PROVIDER', 'mock'),
    'vonage_api_key': os.environ.get('VONAGE_API_KEY', ''),
    'vonage_api_secret': os.environ.get('VONAGE_API_SECRET', ''),
    'email_provider': os.environ.get('EMAIL_PROVIDER', 'mock'),
    'resend_api_key': os.environ.get('RESEND_API_KEY', ''),
    'resend_from_email': os.environ.get('RESEND_FROM_EMAIL', ''),
    'smtp_host': os.environ.get('SMTP_HOST', 'smtp.gmail.com'),
    'smtp_port': os.environ.get('SMTP_PORT', '587'),
    'smtp_username': os.environ.get('SMTP_USERNAME', ''),
    'smtp_password': os.environ.get('SMTP_PASSWORD', ''),
    'smtp_from_email': os.environ.get('SMTP_FROM_EMAIL', ''),
    'docker_username': os.environ.get('DOCKER_USERNAME', os.environ.get('GITHUB_ACTOR', '')),
    'docker_password': os.environ.get('DOCKER_PASSWORD', os.environ.get('GITHUB_TOKEN', '')),
}

if '${DOMAIN}':
    vars['domain_name'] = '${DOMAIN}'
    vars['certbot_email'] = '${EMAIL}'

with open('$VARS_FILE', 'w') as f:
    json.dump(vars, f)

# Self-verification
with open('$VARS_FILE', 'r') as f:
    verify = json.load(f)
    print(f\"CONFIRM: JSON file has Private Key? {'YES' if verify.get('jwt_private_key_pem') else 'NO'}\")
    print(f\"CONFIRM: JSON Private Key length in file: {len(verify.get('jwt_private_key_pem', ''))}\")
"

ansible-playbook -i inventory.ini playbook.yml --extra-vars "@$VARS_FILE"

# Clean up sensitive vars file (Disabled for debugging)
# rm -f "$VARS_FILE"

echo "✅ Deployment complete!"
