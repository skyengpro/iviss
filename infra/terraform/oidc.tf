# =============================================================================
# GitHub Actions OIDC — Federated Authentication (No Static AWS Keys)
# =============================================================================
# This replaces long-lived AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY with
# short-lived, auto-rotating credentials tied to specific repo/branch.
#
# NOTE: The OIDC provider for GitHub Actions already exists in this AWS account
# (created by another project). We reference it via a data source instead of
# creating a duplicate. Only the IAM role + policy are new.
# =============================================================================

# --- Create the OIDC Identity Provider ---
resource "aws_iam_openid_connect_provider" "github_actions" {
  url             = "https://token.actions.githubusercontent.com"
  client_id_list  = ["sts.amazonaws.com"]
  thumbprint_list = ["6938fd4d98bab03faadb97b34396831e3780aea1", "1c58a3a8511116c49cc3984e72a44b58532f83a2"] # Current GH thumbprints
  tags = {
    Project     = var.project_name
    Description = "GitHub Actions OIDC for CI/CD deployments"
  }
}

# --- IAM Role assumed by GitHub Actions ---
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

  max_session_duration = 3600 # 1 hour max

  tags = {
    Project     = var.project_name
    Environment = var.environment
  }
}

# --- Permissions Policy: Least-Privilege for Deploy ---
resource "aws_iam_role_policy" "deploy_permissions" {
  name = "${var.project_name}-deploy-policy"
  role = aws_iam_role.github_actions_deploy.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [

      {
        Sid    = "CloudFrontAndAcm"
        Effect = "Allow"
        Action = [
          "cloudfront:CreateDistribution",
          "cloudfront:UpdateDistribution",
          "cloudfront:GetDistribution",
          "cloudfront:GetDistributionConfig",
          "cloudfront:DeleteDistribution",
          "cloudfront:ListDistributions",
          "cloudfront:CreateInvalidation",
          "cloudfront:ListTagsForResource",
          "cloudfront:TagResource",
          "cloudfront:UntagResource",
          "acm:RequestCertificate",
          "acm:DescribeCertificate",
          "acm:DeleteCertificate",
          "acm:AddTagsToCertificate",
          "acm:ListTagsForCertificate",
          "acm:ListCertificates"
        ]
        Resource = "*"
      },
      {
        Sid    = "WafManage"
        Effect = "Allow"
        Action = [
          "wafv2:CreateWebACL",
          "wafv2:UpdateWebACL",
          "wafv2:DeleteWebACL",
          "wafv2:GetWebACL",
          "wafv2:ListWebACLs",
          "wafv2:ListTagsForResource",
          "wafv2:TagResource",
          "wafv2:UntagResource"
        ]
        Resource = "*"
      },
      {
        Sid    = "Route53ForCloudFrontDns"
        Effect = "Allow"
        Action = [
          "route53:ChangeResourceRecordSets",
          "route53:GetChange",
          "route53:GetHostedZone",
          "route53:ListHostedZonesByName",
          "route53:ListResourceRecordSets"
        ]
        Resource = "*"
      },
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
        # ARN pattern matches secrets like:
        # arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/production/app-secrets-AbCdEf
        # The wildcard after 'iviss/' matches the full path including AWS's random suffix
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
        # ARN pattern matches secrets like:
        # arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/production/cloudfront-origin-secret-XyZ123
        # The wildcard after 'iviss/' matches the full path including AWS's random suffix
        Resource = "arn:aws:secretsmanager:eu-west-1:577638362880:secret:iviss/*"
      }
    ]
  })
}

# --- Outputs ---
output "github_actions_role_arn" {
  value       = aws_iam_role.github_actions_deploy.arn
  description = "Use this ARN in deploy-aws.yml: role-to-assume"
}
