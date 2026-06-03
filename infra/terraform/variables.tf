variable "aws_region" {
  description = "AWS region for Secrets Manager and IAM"
  type        = string
  default     = "eu-west-1"
}

variable "project_name" {
  description = "Project name for resource tagging"
  type        = string
  default     = "iviss"
}

variable "environment" {
  description = "Deployment environment"
  type        = string
  default     = "production"
}