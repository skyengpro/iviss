output "instance_ip" {
  value = aws_lightsail_static_ip.iviss_ip.ip_address
}

output "private_key" {
  value     = aws_lightsail_key_pair.iviss_key.private_key
  sensitive = true
}

output "image_tag" {
  value = var.image_tag
}
