output "instance_ip" {
  value = aws_lightsail_static_ip.iviss_ip.ip_address
}

output "lightsail_private_ip" {
  value = aws_lightsail_instance.iviss_app.private_ip_address
}

output "private_key" {
  value     = aws_lightsail_key_pair.iviss_key.private_key
  sensitive = true
}

output "cloudfront_distribution_id" {
  value = aws_cloudfront_distribution.iviss.id
}

output "cloudfront_distribution_domain_name" {
  value = aws_cloudfront_distribution.iviss.domain_name
}

output "image_tag" {
  value = var.image_tag
}
