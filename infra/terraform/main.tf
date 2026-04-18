terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

# SSH Key Pair for Lightsail
resource "aws_lightsail_key_pair" "iviss_key" {
  name = "${var.project_name}-${var.environment}-key-v2"
}

# Lightsail Instance
resource "aws_lightsail_instance" "iviss_app" {
  name              = "${var.project_name}-${var.environment}-app-v2"
  availability_zone = "${var.aws_region}a"
  blueprint_id      = var.lightsail_blueprint_id
  bundle_id         = var.lightsail_bundle_id
  key_pair_name     = aws_lightsail_key_pair.iviss_key.name

  tags = {
    Project     = var.project_name
    Environment = var.environment
  }
}

# Static IP for the Instance
resource "aws_lightsail_static_ip" "iviss_ip" {
  name = "${var.project_name}-${var.environment}-ip-v2"
  
  lifecycle {
    prevent_destroy = true
  }
}

resource "aws_lightsail_static_ip_attachment" "iviss_ip_attach" {
  static_ip_name = aws_lightsail_static_ip.iviss_ip.id
  instance_name  = aws_lightsail_instance.iviss_app.id
}

# Firewall rules
resource "aws_lightsail_instance_public_ports" "iviss_ports" {
  instance_name = aws_lightsail_instance.iviss_app.name

  port_info {
    protocol  = "tcp"
    from_port = 22
    to_port   = 22
  }

  port_info {
    protocol  = "tcp"
    from_port = 80
    to_port   = 80
  }

  port_info {
    protocol  = "tcp"
    from_port = 443
    to_port   = 443
  }
}

# Ansible Deployment Trigger
resource "null_resource" "ansible_deploy" {
  count = var.auto_deploy ? 1 : 0

  triggers = {
    instance_id = aws_lightsail_instance.iviss_app.id
    static_ip   = aws_lightsail_static_ip.iviss_ip.ip_address
  }

  provisioner "local-exec" {
    command = <<EOT
      set -e
      ANSIBLE_DIR="../ansible"

      # Save SSH Key
      echo "${aws_lightsail_key_pair.iviss_key.private_key}" > $ANSIBLE_DIR/iviss-key.pem
      chmod 600 $ANSIBLE_DIR/iviss-key.pem

      # Generate Inventory
      cat <<EOF > $ANSIBLE_DIR/inventory.ini
[iviss_prod]
${aws_lightsail_static_ip.iviss_ip.ip_address} ansible_user=ubuntu ansible_ssh_private_key_file=./iviss-key.pem ansible_ssh_common_args='-o StrictHostKeyChecking=no'
EOF

      # Wait for SSH
      echo "Waiting for SSH to be ready..."
      until nc -zvw5 ${aws_lightsail_static_ip.iviss_ip.ip_address} 22; do
        sleep 5
      done

      # Run Ansible
      cd $ANSIBLE_DIR
      ansible-playbook -i inventory.ini playbook.yml \
        -e "db_password=${var.admin_bootstrap_password}" \
        -e "jwt_private_key_pem='${var.jwt_private_key_pem}'" \
        -e "jwt_public_key_pem='${var.jwt_public_key_pem}'" \
        -e "activation_code_pepper='${var.activation_code_pepper}'" \
        -e "admin_bootstrap_email='${var.admin_bootstrap_email}'" \
        -e "admin_bootstrap_password='${var.admin_bootstrap_password}'" \
        -e "admin_bootstrap_phone='${var.admin_bootstrap_phone}'" \
        -e "admin_bootstrap_username='${var.admin_bootstrap_username}'" \
        -e "twilio_account_sid='${var.twilio_account_sid}'" \
        -e "twilio_auth_token='${var.twilio_auth_token}'" \
        -e "twilio_from_number='${var.twilio_from_number}'" \
        -e "sms_provider='${var.sms_provider}'" \
        -e "vonage_api_key='${var.vonage_api_key}'" \
        -e "vonage_api_secret='${var.vonage_api_secret}'" \
        -e "email_provider='${var.email_provider}'" \
        -e "resend_api_key='${var.resend_api_key}'" \
        -e "resend_from_email='${var.resend_from_email}'" \
        -e "smtp_host='${var.smtp_host}'" \
        -e "smtp_port='${var.smtp_port}'" \
        -e "smtp_username='${var.smtp_username}'" \
        -e "smtp_password='${var.smtp_password}'" \
        -e "smtp_from_email='${var.smtp_from_email}'" \
        -e "shift_start_hour='${var.shift_start_hour}'" \
        -e "shift_end_hour='${var.shift_end_hour}'" \
        -e "docker_username='${var.github_username}'" \
        -e "docker_password='${var.github_token}'" \
        -e "vite_api_url=http://${var.domain_name != "" ? var.domain_name : aws_lightsail_static_ip.iviss_ip.ip_address}:3000" \
        ${var.domain_name != "" ? "-e \"domain_name=${var.domain_name}\" -e \"certbot_email=${var.certbot_email}\"" : ""}
    EOT
  }

  depends_on = [
    aws_lightsail_instance_public_ports.iviss_ports,
    aws_lightsail_static_ip_attachment.iviss_ip_attach
  ]
}
