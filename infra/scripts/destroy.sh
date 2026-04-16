#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TERRAFORM_DIR="$PROJECT_ROOT/infra/terraform"
ANSIBLE_DIR="$PROJECT_ROOT/infra/ansible"

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
        cd "$ANSIBLE_DIR"
        ansible-playbook -i inventory.ini cleanup.yml || echo "Ansible cleanup failed, proceeding with infrastructure destruction."
    fi

    cd "$TERRAFORM_DIR"
    terraform init -reconfigure
    terraform destroy -auto-approve
    rm -f "$ANSIBLE_DIR/iviss-key.pem" "$ANSIBLE_DIR/inventory.ini"
    echo "💥 Infrastructure destroyed."
else
    echo "Operation cancelled."
fi
