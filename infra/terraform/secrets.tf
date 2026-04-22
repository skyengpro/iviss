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
  # Initialize with empty schema — seed manually using AWS Console or CLI
  secret_string = jsonencode({
    jwt_private_key_pem      = ""
    jwt_public_key_pem       = ""
    activation_code_pepper   = ""
    db_password              = ""
    admin_bootstrap_password = ""
    docker_password          = ""
  })

  lifecycle {
    # CRITICAL: Prevent terraform from overwriting manually seeded secrets
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
  # Initialize with empty schema including ORANGE credentials
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
  })

  lifecycle {
    ignore_changes = [secret_string]
  }
}
