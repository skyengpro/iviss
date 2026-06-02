# =============================================================================
# AWS Secrets Manager — Centralized Secret Storage for IVISS
# =============================================================================
# Secrets are grouped by domain for independent rotation and least-privilege access.
#
# Group 1: iviss/<env>/app-secrets     — Core auth & database credentials
# Group 2: iviss/<env>/provider-keys   — Third-party API keys (SMS, Email, etc.)
# =============================================================================

# -------------------------
# Group 1: Core App Secrets
# -------------------------
resource "aws_secretsmanager_secret" "app_secrets" {
  name                    = "${var.project_name}/${var.environment}/app-secrets"
  description             = "Core application secrets — JWT keys, DB password, admin credentials"
  recovery_window_in_days = 7

  tags = {
    Project     = var.project_name
    Environment = var.environment
    Group       = "app-core"
  }
}

resource "aws_secretsmanager_secret_version" "app_secrets" {
  secret_id = aws_secretsmanager_secret.app_secrets.id
  secret_string = jsonencode({
    jwt_private_key_pem      = ""
    jwt_public_key_pem       = ""
    activation_code_pepper   = ""
    db_password              = ""
    admin_bootstrap_password = ""
    docker_password          = ""
  })

  lifecycle {
    ignore_changes = [secret_string]
  }
}

# -------------------------
# Group 2: Provider API Keys
# -------------------------
resource "aws_secretsmanager_secret" "provider_keys" {
  name                    = "${var.project_name}/${var.environment}/provider-keys"
  description             = "Third-party provider API keys — Twilio, Vonage, SMTP, Resend, Orange"
  recovery_window_in_days = 7

  tags = {
    Project     = var.project_name
    Environment = var.environment
    Group       = "providers"
  }
}

resource "aws_secretsmanager_secret_version" "provider_keys" {
  secret_id = aws_secretsmanager_secret.provider_keys.id
  secret_string = jsonencode({
    twilio_account_sid   = ""
    twilio_auth_token    = ""
    twilio_from_number   = ""
    vonage_api_key       = ""
    vonage_api_secret    = ""
    orange_client_id     = ""
    orange_client_secret = ""
    orange_sender_number = ""
    resend_api_key       = ""
    smtp_password        = ""
    smtp_from_email      = ""
    resend_from_email    = ""
  })

  lifecycle {
    ignore_changes = [secret_string]
  }
}

# -------------------------
# Group 3: Vehicle API Keys
# -------------------------
resource "aws_secretsmanager_secret" "vehicle_api_keys" {
  name                    = "${var.project_name}/${var.environment}/vehicle-api-keys"
  description             = "Vehicle identification API credentials and endpoint configuration"
  recovery_window_in_days = 7

  tags = {
    Project     = var.project_name
    Environment = var.environment
    Group       = "vehicle-api"
  }
}

resource "aws_secretsmanager_secret_version" "vehicle_api_keys" {
  secret_id = aws_secretsmanager_secret.vehicle_api_keys.id
  secret_string = jsonencode({
    external_api_base_url    = ""
    external_api_username    = ""
    external_api_password    = ""
    external_api_lock_ndia  = ""
    external_api_kindia     = ""
    external_api_user       = ""
    external_api_client    = ""
    external_api_ctr        = ""
    external_api_tls_cert_b64 = ""
  })

  lifecycle {
    ignore_changes = [secret_string]
  }
}