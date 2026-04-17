#!/bin/bash
set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PROJECT_NAME="iviss"

# Load variables from .env
if [ -f "$PROJECT_ROOT/.env" ]; then
    set -o allexport
    source "$PROJECT_ROOT/.env"
    set +o allexport
fi

REGION=${AWS_DEFAULT_REGION:-"eu-west-1"}
BUCKET_NAME="${PROJECT_NAME}-terraform-state-$(aws sts get-caller-identity --query Account --output text)"
TABLE_NAME="${PROJECT_NAME}-terraform-lock"

echo "☁️ Setting up Remote Terraform State in $REGION..."

# 1. Create S3 Bucket
if aws s3api head-bucket --bucket "$BUCKET_NAME" 2>/dev/null; then
    echo "✅ S3 Bucket $BUCKET_NAME already exists."
else
    echo "📦 Creating S3 Bucket $BUCKET_NAME..."
    if [ "$REGION" == "us-east-1" ]; then
        aws s3api create-bucket --bucket "$BUCKET_NAME" --region "$REGION"
    else
        aws s3api create-bucket --bucket "$BUCKET_NAME" --region "$REGION" --create-bucket-configuration LocationConstraint="$REGION"
    fi
    
    # Enable versioning (Best Practice!)
    aws s3api put-bucket-versioning --bucket "$BUCKET_NAME" --versioning-configuration Status=Enabled
    
    # Enable Server-Side Encryption (Best Practice!)
    aws s3api put-bucket-encryption --bucket "$BUCKET_NAME" --server-side-encryption-configuration '{
        "Rules": [
            {
                "ApplyServerSideEncryptionByDefault": {
                    "SSEAlgorithm": "AES256"
                }
            }
        ]
    }'
fi

# 2. Create DynamoDB Table for Locking
if aws dynamodb describe-table --table-name "$TABLE_NAME" --region "$REGION" >/dev/null 2>&1; then
    echo "✅ DynamoDB Table $TABLE_NAME already exists."
else
    echo "🔒 Creating DynamoDB Table $TABLE_NAME for state locking..."
    aws dynamodb create-table \
        --table-name "$TABLE_NAME" \
        --region "$REGION" \
        --attribute-definitions AttributeName=LockID,AttributeType=S \
        --key-schema AttributeName=LockID,KeyType=HASH \
        --provisioned-throughput ReadCapacityUnits=5,WriteCapacityUnits=5
fi

# 3. Update Terraform Backend Configuration
BACKEND_FILE="$PROJECT_ROOT/infra/terraform/backend.tf"
echo "📝 Generating backend.tf..."
cat <<EOF > "$BACKEND_FILE"
terraform {
  backend "s3" {
    bucket         = "$BUCKET_NAME"
    key            = "production/terraform.tfstate"
    region         = "$REGION"
    dynamodb_table = "$TABLE_NAME"
    encrypt        = true
  }
}
EOF

echo "✨ Remote state infrastructure is ready!"
echo "📍 Bucket: $BUCKET_NAME"
echo "📍 Table: $TABLE_NAME"
echo "👉 You can now run ./infra/scripts/deploy.sh"
