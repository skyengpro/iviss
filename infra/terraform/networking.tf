data "http" "aws_ip_ranges" {
  url = "https://ip-ranges.amazonaws.com/ip-ranges.json"
}

locals {
  # Use AWS-published CloudFront origin-facing ranges directly.
  cloudfront_origin_ipv4_cidrs = sort(distinct([
    for prefix in jsondecode(data.http.aws_ip_ranges.response_body).prefixes :
    prefix.ip_prefix
    if prefix.service == "CLOUDFRONT_ORIGIN_FACING"
  ]))

  # Lightsail allows max 10 CIDRs per port rule.
  cloudfront_cidr_chunks = chunklist(local.cloudfront_origin_ipv4_cidrs, 10)

  # Build port infos array with one entry per chunk
  lightsail_public_port_infos = jsonencode([
    for chunk in local.cloudfront_cidr_chunks : {
      fromPort = 80
      toPort   = 80
      protocol = "tcp"
      cidrs    = chunk
    }
  ])
}

resource "null_resource" "lightsail_firewall" {
  count = var.edge_lockdown_enabled ? 1 : 0

  triggers = {
    instance_name        = aws_lightsail_instance.iviss_app.name
    cloudfront_cidrs_sha = sha256(jsonencode(local.cloudfront_origin_ipv4_cidrs))
  }

  provisioner "local-exec" {
    command = <<-EOT
      set -e
      echo "Applying CloudFront CIDR restrictions to Lightsail firewall..."
      echo "Total CIDRs: ${length(local.cloudfront_origin_ipv4_cidrs)}"
      echo "Number of rules: ${length(local.cloudfront_cidr_chunks)} (max 10 CIDRs per rule)"
      
      cat <<'PORTINFOS' > /tmp/iviss-lightsail-port-infos.json
${local.lightsail_public_port_infos}
PORTINFOS
      
      echo "Applying firewall rules..."
      aws lightsail put-instance-public-ports \
        --region ${var.aws_region} \
        --instance-name ${aws_lightsail_instance.iviss_app.name} \
        --port-infos file:///tmp/iviss-lightsail-port-infos.json
      
      rm -f /tmp/iviss-lightsail-port-infos.json
      
      echo ""
      echo "✅ Firewall rules applied successfully."
      echo "   - Port 80 restricted to CloudFront origin-facing IPs only"
      echo "   - ${length(local.cloudfront_origin_ipv4_cidrs)} CIDRs in ${length(local.cloudfront_cidr_chunks)} rules"
      echo ""
      echo "Note: Verify firewall rules in AWS Console or with:"
      echo "  aws lightsail get-instance-public-ports --region ${var.aws_region} --instance-name ${aws_lightsail_instance.iviss_app.name}"
    EOT
  }

  depends_on = [
    aws_lightsail_instance.iviss_app,
    aws_lightsail_static_ip_attachment.iviss_ip_attach
  ]
}
