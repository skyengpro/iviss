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

variable "image_tag" {
  description = "The Docker image tag to deploy (e.g. 1.0.0-rc.8 or latest)"
  type        = string
  default     = "latest"
}

variable "k8s_origin_hostname" {
  description = "Hostname of the K8s Ingress LoadBalancer (no longer used — traffic goes direct to Hetzner)"
  type        = string
  default     = ""
}