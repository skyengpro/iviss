#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TERRAFORM_DIR="$PROJECT_ROOT/infra/terraform"
ANSIBLE_DIR="$PROJECT_ROOT/infra/ansible"

prepare_ansible_ssh() {
    cd "$TERRAFORM_DIR"
    terraform init -reconfigure >/dev/null

    local instance_ip
    local private_key
    instance_ip="$(terraform output -raw instance_ip 2>/dev/null || true)"
    private_key="$(terraform output -raw private_key 2>/dev/null || true)"

    if [ -z "$instance_ip" ] || [ -z "$private_key" ]; then
        echo "⚠️  Could not resolve instance SSH details from Terraform outputs."
        return 1
    fi

    echo "$private_key" > "$ANSIBLE_DIR/iviss-key.pem"
    chmod 600 "$ANSIBLE_DIR/iviss-key.pem"

    cat <<EOF > "$ANSIBLE_DIR/ssh_config"
Host lightsail-public
  HostName $instance_ip
  User ubuntu
  IdentityFile $ANSIBLE_DIR/iviss-key.pem
  IdentitiesOnly yes
  StrictHostKeyChecking no
  UserKnownHostsFile /dev/null
EOF

    cat <<EOF > "$ANSIBLE_DIR/inventory.ini"
[iviss_prod]
lightsail-public ansible_host=$instance_ip ansible_user=ubuntu ansible_ssh_private_key_file=$ANSIBLE_DIR/iviss-key.pem ansible_ssh_common_args='-F $ANSIBLE_DIR/ssh_config'
EOF

    return 0
}

# Load local .env if it exists
if [ -f "$PROJECT_ROOT/.env" ]; then
    echo "📄 Loading variables from .env..."
    # allexport mode automatically exports all variables defined in the sourced file
    set -o allexport
    source "$PROJECT_ROOT/.env"
    set +o allexport
fi

echo "⚠️  WARNING: This will destroy all IVISS production infrastructure!"
read -p "Are you sure? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Optional Ansible Cleanup
    read -p "Run app-level cleanup (docker down, etc.) before destroying instance? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if prepare_ansible_ssh; then
            cd "$ANSIBLE_DIR"
            ansible-playbook -i inventory.ini cleanup.yml || echo "Ansible cleanup failed, proceeding with infrastructure destruction."
        else
            echo "⚠️  Skipping Ansible cleanup because SSH artifacts could not be prepared."
        fi
    fi

    cd "$TERRAFORM_DIR"
    terraform init -reconfigure
    # We target the instance; Terraform's dependency tracking will automatically 
    # destroy ports, attachments, and keys that depend on it, leaving the Static IP.
    terraform destroy -auto-approve -target=aws_lightsail_instance.iviss_app
    rm -f "$ANSIBLE_DIR/iviss-key.pem" "$ANSIBLE_DIR/inventory.ini" "$ANSIBLE_DIR/ssh_config"
    echo "💥 Infrastructure destroyed."
else
    echo "Operation cancelled."
fi
