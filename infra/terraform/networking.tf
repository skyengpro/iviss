# Networking for the K8s cluster is managed by Hetzner Cloud (hcloud).
# The Nginx Ingress Controller creates a Hetzner Load Balancer
# which receives traffic directly (no CloudFront).
#
# See infra/terraform/hetzner/ for the K8s cluster provisioning.