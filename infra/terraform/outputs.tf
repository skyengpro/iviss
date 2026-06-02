output "cloudfront_distribution_id" {
  value = aws_cloudfront_distribution.iviss.id
}

output "cloudfront_distribution_domain_name" {
  value = aws_cloudfront_distribution.iviss.domain_name
}

output "image_tag" {
  value = var.image_tag
}
