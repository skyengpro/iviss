# =============================================================================
# GitHub Actions OIDC — Federated Authentication (No Static AWS Keys)
# =============================================================================
# Short-lived credentials tied to the skyengpro/iviss repo.
# Only needed for: Terraform state (S3 + DynamoDB) and Secrets Manager read/write.
# =============================================================================
 
resource "aws_iam_openid_connect_provider" "github_actions" {
  url             = "https://token.actions.githubusercontent.com"
  client_id_list  = ["sts.amazonaws.com"]
  thumbprint_list = ["6938fd4d98bab03faadb97b34396831e3780aea1", "1c58a3a8511116c49cc3984e72a44b58532f83a2"]
  tags = {
    Project     = var.project_name
    Description = "GitHub Actions OIDC for CI/CD"
  }
}

resource "aws_iam_role" "github_actions_deploy" {
  name = "${var.project_name}-github-actions-deploy"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Principal = {
          Federated = aws_iam_openid_connect_provider.github_actions.arn
        }
        Action = "sts:AssumeRoleWithWebIdentity"
        Condition = {
          StringLike = {
            "token.actions.githubusercontent.com:aud" = "sts.amazonaws.com"
            "token.actions.githubusercontent.com:sub" = "repo:skyengpro/iviss:*"
          }
        }
      }
    ]
  })

  max_session_duration = 3600

  tags = {
    Project     = var.project_name
    Environment = var.environment
  }
}

resource "aws_iam_role_policy" "deploy_permissions" {
  name = "${var.project_name}-deploy-policy"
  role = aws_iam_role.github_actions_deploy.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "TerraformStateBucket"
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject",
          "s3:ListBucket"
        ]
        Resource = [
          "arn:aws:s3:::iviss-terraform-state-577638362880",
          "arn:aws:s3:::iviss-terraform-state-577638362880/*"
        ]
      },
      {
        Sid    = "TerraformStateLock"
        Effect = "Allow"
        Action = [
          "dynamodb:GetItem",
          "dynamodb:PutItem",
          "dynamodb:DeleteItem"
        ]
        Resource = "arn:aws:dynamodb:eu-central-1:577638362880:table/iviss-terraform-lock"
      },
      {
        Sid    = "SecretsManagerRead"
        Effect = "Allow"
        Action = [
          "secretsmanager:GetSecretValue",
          "secretsmanager:DescribeSecret",
          "secretsmanager:GetResourcePolicy"
        ]
        Resource = "arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/*"
      },
      {
        Sid    = "SecretsManagerWrite"
        Effect = "Allow"
        Action = [
          "secretsmanager:CreateSecret",
          "secretsmanager:UpdateSecret",
          "secretsmanager:PutSecretValue",
          "secretsmanager:TagResource"
        ]
        Resource = "arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/*"
      },
      {
        Sid    = "IAMRead"
        Effect = "Allow"
        Action = [
          "iam:GetRole",
          "iam:GetRolePolicy",
          "iam:ListAttachedRolePolicies",
          "iam:ListRolePolicies",
          "iam:GetOpenIDConnectProvider"
        ]
        Resource = [
          "arn:aws:iam::577638362880:role/iviss-*",
          "arn:aws:iam::577638362880:oidc-provider/token.actions.githubusercontent.com"
        ]
      }
    ]
  })
}

output "github_actions_role_arn" {
  value       = aws_iam_role.github_actions_deploy.arn
  description = "Use this ARN in CI: role-to-assume"
}