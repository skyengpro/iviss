terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.6"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

# AWS resources retained: Secrets Manager + IAM (OIDC for GitHub Actions CI)
# CloudFront, WAF, ACM, Route53, and Lightsail have been removed.