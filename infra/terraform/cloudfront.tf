locals {
  cloudfront_custom_domain_enabled = var.domain_name != "" && var.route53_zone_id != ""
  # Origin hostname pointing to the Hetzner K8s Nginx Ingress LoadBalancer
  k8s_origin_hostname              = var.k8s_origin_hostname != "" ? var.k8s_origin_hostname : "placeholder.sslip.io"
}

resource "random_password" "cloudfront_origin_secret" {
  length           = 32
  special          = false
  override_special = ""
}

resource "aws_secretsmanager_secret" "cloudfront_origin_secret" {
  name                    = "${var.project_name}/${var.environment}/cloudfront-origin-secret"
  description             = "Shared secret header that CloudFront uses to prove origin requests to Lightsail"
  recovery_window_in_days = 7

  tags = {
    Project     = var.project_name
    Environment = var.environment
    Group       = "edge"
  }
}

resource "aws_secretsmanager_secret_version" "cloudfront_origin_secret" {
  secret_id     = aws_secretsmanager_secret.cloudfront_origin_secret.id
  secret_string = random_password.cloudfront_origin_secret.result

  lifecycle {
    ignore_changes = [secret_string]
  }
}

resource "aws_acm_certificate" "cloudfront" {
  provider          = aws.us_east_1
  count             = local.cloudfront_custom_domain_enabled ? 1 : 0
  domain_name       = var.domain_name
  validation_method = "DNS"

  lifecycle {
    create_before_destroy = true
  }

  tags = {
    Project     = var.project_name
    Environment = var.environment
  }
}

resource "aws_route53_record" "cloudfront_cert_validation" {
  count   = local.cloudfront_custom_domain_enabled ? length(aws_acm_certificate.cloudfront[0].domain_validation_options) : 0
  zone_id = var.route53_zone_id
  name    = tolist(aws_acm_certificate.cloudfront[0].domain_validation_options)[count.index].resource_record_name
  type    = tolist(aws_acm_certificate.cloudfront[0].domain_validation_options)[count.index].resource_record_type
  records = [tolist(aws_acm_certificate.cloudfront[0].domain_validation_options)[count.index].resource_record_value]
  ttl     = 60
}

resource "aws_acm_certificate_validation" "cloudfront" {
  provider                = aws.us_east_1
  count                   = local.cloudfront_custom_domain_enabled ? 1 : 0
  certificate_arn         = aws_acm_certificate.cloudfront[0].arn
  validation_record_fqdns = aws_route53_record.cloudfront_cert_validation[*].fqdn
}

resource "aws_wafv2_web_acl" "cloudfront" {
  provider    = aws.us_east_1
  name        = "${var.project_name}-${var.environment}-cloudfront"
  description = "Baseline managed protections for the IVISS CloudFront distribution"
  scope       = "CLOUDFRONT"

  default_action {
    allow {}
  }

  rule {
    name     = "AWSManagedRulesCommonRuleSet"
    priority = 1

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        vendor_name = "AWS"
        name        = "AWSManagedRulesCommonRuleSet"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "${var.project_name}-${var.environment}-common"
      sampled_requests_enabled   = true
    }
  }

  rule {
    name     = "AWSManagedRulesKnownBadInputsRuleSet"
    priority = 2

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        vendor_name = "AWS"
        name        = "AWSManagedRulesKnownBadInputsRuleSet"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "${var.project_name}-${var.environment}-known-bad"
      sampled_requests_enabled   = true
    }
  }

  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name                = "${var.project_name}-${var.environment}-cloudfront"
    sampled_requests_enabled   = true
  }

  tags = {
    Project     = var.project_name
    Environment = var.environment
  }
}

resource "aws_cloudfront_distribution" "iviss" {
  enabled             = true
  is_ipv6_enabled     = true
  comment             = "IVISS edge distribution"
  default_root_object = ""
  aliases             = local.cloudfront_custom_domain_enabled ? [var.domain_name] : []
  price_class         = "PriceClass_100"
  web_acl_id          = aws_wafv2_web_acl.cloudfront.arn

  origin {
    domain_name = local.k8s_origin_hostname
    origin_id   = "k8s-origin"

    custom_header {
      name  = "X-Origin-Verify"
      value = random_password.cloudfront_origin_secret.result
    }

    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }

  default_cache_behavior {
    allowed_methods        = ["DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "k8s-origin"
    viewer_protocol_policy = "redirect-to-https"
    compress               = true

    # Modern cache policy: no caching for dynamic app content
    # Using AWS-managed CachingDisabled policy
    cache_policy_id = "4135ea2d-6df8-44a3-9df3-4b5a84be39ad"

    # Origin request policy: forward all viewer headers, cookies, and query strings
    # Using AWS-managed AllViewer policy
    origin_request_policy_id = "216adef6-5c7f-47e4-b989-5492eafa07d3"
  }

  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }

  viewer_certificate {
    acm_certificate_arn            = local.cloudfront_custom_domain_enabled ? aws_acm_certificate_validation.cloudfront[0].certificate_arn : null
    cloudfront_default_certificate = local.cloudfront_custom_domain_enabled ? false : true
    ssl_support_method             = local.cloudfront_custom_domain_enabled ? "sni-only" : null
    minimum_protocol_version       = local.cloudfront_custom_domain_enabled ? "TLSv1.2_2021" : "TLSv1"
  }

  tags = {
    Project     = var.project_name
    Environment = var.environment
  }

  depends_on = [
    aws_wafv2_web_acl.cloudfront,
    aws_acm_certificate_validation.cloudfront
  ]
}

resource "aws_route53_record" "cloudfront_alias_ipv4" {
  count   = local.cloudfront_custom_domain_enabled ? 1 : 0
  allow_overwrite = true
  zone_id = var.route53_zone_id
  name    = var.domain_name
  type    = "A"

  alias {
    name                   = aws_cloudfront_distribution.iviss.domain_name
    zone_id                = aws_cloudfront_distribution.iviss.hosted_zone_id
    evaluate_target_health = false
  }
}

resource "aws_route53_record" "cloudfront_alias_ipv6" {
  count   = local.cloudfront_custom_domain_enabled ? 1 : 0
  allow_overwrite = true
  zone_id = var.route53_zone_id
  name    = var.domain_name
  type    = "AAAA"

  alias {
    name                   = aws_cloudfront_distribution.iviss.domain_name
    zone_id                = aws_cloudfront_distribution.iviss.hosted_zone_id
    evaluate_target_health = false
  }
}
