variable "aws_region" {
  description = "AWS region to deploy to"
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

variable "lightsail_blueprint_id" {
  description = "Lightsail blueprint ID (OS)"
  type        = string
  default     = "ubuntu_22_04"
}

variable "lightsail_bundle_id" {
  description = "Lightsail bundle ID (Instance size)"
  type        = string
  default     = "small_3_0" # 2 GB RAM, 2 vCPUs, 60 GB SSD
}

variable "auto_deploy" {
  description = "Whether to automatically trigger Ansible deployment after provisioning"
  type        = bool
  default     = true
}

# Application Secrets (Used if auto_deploy is true)
variable "jwt_private_key_pem" { 
  type      = string 
  default   = "" 
  sensitive = true 
}

variable "jwt_public_key_pem" { 
  type      = string 
  default   = "" 
  sensitive = true 
}

variable "activation_code_pepper" { 
  type      = string 
  default   = "" 
  sensitive = true 
}

variable "admin_bootstrap_email" { 
  type      = string 
  default   = "admin@iviss.local" 
}

variable "admin_bootstrap_password" { 
  type      = string 
  default   = "" 
  sensitive = true 
}

variable "admin_bootstrap_phone" { 
  type      = string 
  default   = "" 
}

variable "admin_bootstrap_username" { 
  type      = string 
  default   = "admin" 
}

variable "twilio_account_sid" { 
  type      = string 
  default   = "mock" 
}

variable "twilio_auth_token" { 
  type      = string 
  default   = "mock" 
  sensitive = true 
}

variable "twilio_from_number" { 
  type      = string 
  default   = "mock" 
}

variable "github_username" { 
  type      = string 
  default   = "" 
}

variable "github_token" { 
  type      = string 
  default   = "" 
  sensitive = true 
}

variable "domain_name" { 
  type      = string 
  default   = "" 
}

variable "certbot_email" { 
  type      = string 
  default   = "admin@iviss.local" 
}

# SMS Provider Configuration
variable "sms_provider" {
  type    = string
  default = "mock"
}

variable "vonage_api_key" {
  type    = string
  default = ""
}

variable "vonage_api_secret" {
  type      = string
  default   = ""
  sensitive = true
}

# Email Provider Configuration
variable "email_provider" {
  type    = string
  default = "mock"
}

variable "resend_api_key" {
  type      = string
  default   = ""
  sensitive = true
}

variable "resend_from_email" {
  type    = string
  default = ""
}

variable "smtp_host" {
  type    = string
  default = ""
}

variable "smtp_port" {
  type    = string
  default = "587"
}

variable "smtp_username" {
  type    = string
  default = ""
}

variable "smtp_password" {
  type      = string
  default   = ""
  sensitive = true
}

variable "smtp_from_email" {
  type    = string
  default = ""
}

# Shift Configuration
variable "shift_start_hour" {
  type    = string
  default = "6"
}

variable "shift_end_hour" {
  type    = string
  default = "18"
}

