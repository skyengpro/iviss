output "secrets_manager_arns" {
  value = {
    app_secrets     = aws_secretsmanager_secret.app_secrets.arn
    provider_keys   = aws_secretsmanager_secret.provider_keys.arn
    vehicle_api_keys = aws_secretsmanager_secret.vehicle_api_keys.arn
  }
  description = "ARNs of the AWS Secrets Manager secrets"
}