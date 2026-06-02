terraform {
  required_providers {
    hcloud = {
      source  = "hetznercloud/hcloud"
      version = "~> 1.49"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.17"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.36"
    }
    kubectl = {
      source  = "gavinbunney/kubectl"
      version = "~> 2.1"
    }
    tls = {
      source  = "hashicorp/tls"
      version = "~> 4.0"
    }
  }
}

provider "hcloud" {
  token = var.hcloud_token
}

# --- SSH Key ---
resource "tls_private_key" "k3s" {
  algorithm = "ED25519"
}

resource "hcloud_ssh_key" "k3s" {
  name       = "${var.cluster_name}-key"
  public_key = tls_private_key.k3s.public_key_openssh
}

# --- Firewall ---
resource "hcloud_firewall" "k3s" {
  name = "${var.cluster_name}-firewall"

  rule {
    direction = "in"
    protocol  = "tcp"
    source_ips = ["0.0.0.0/0", "::/0"]
    port       = "22"
    description = "SSH"
  }

  rule {
    direction = "in"
    protocol  = "tcp"
    source_ips = var.cloudfront_cidrs != null ? var.cloudfront_cidrs : ["0.0.0.0/0"]
    port       = "443"
    description = "HTTPS (Ingress)"
  }

  rule {
    direction = "in"
    protocol  = "tcp"
    source_ips = ["0.0.0.0/0", "::/0"]
    port       = "80"
    description = "HTTP (Ingress, redirect to HTTPS)"
  }

  rule {
    direction = "in"
    protocol  = "udp"
    source_ips = ["10.0.0.0/8"]
    port       = "51820"
    description = "WireGuard (k3s flannel)"
  }

  rule {
    direction = "in"
    protocol  = "tcp"
    source_ips = ["10.0.0.0/8"]
    port       = "6443"
    description = "Kubernetes API"
  }

  rule {
    direction = "in"
    protocol  = "tcp"
    source_ips = ["10.0.0.0/8"]
    port       = "2379-2380"
    description = "etcd"
  }

  rule {
    direction = "in"
    protocol  = "udp"
    source_ips = ["10.0.0.0/8"]
    port       = "8472"
    description = "Flannel VXLAN"
  }

  rule {
    direction = "in"
    protocol  = "tcp"
    source_ips = ["10.0.0.0/8"]
    port       = "10250"
    description = "kubelet"
  }
}

# --- Server nodes ---
resource "hcloud_server" "control_plane" {
  count       = var.control_plane_count
  name        = "${var.cluster_name}-cp-${count.index + 1}"
  server_type = var.control_plane_type
  image       = var.image_os
  location    = var.location
  ssh_keys    = [hcloud_ssh_key.k3s.id]
  firewall_ids = [hcloud_firewall.k3s.id]

  user_data = templatefile("${path.module}/templates/control_plane.yaml.tpl", {
    k3s_version      = var.k3s_version
    cluster_secret   = random_password.cluster_secret.result
    tls_sans         = concat([for s in hcloud_server.control_plane[*] : s.ipv4_address], var.additional_tls_sans)
    k3s_token        = random_password.k3s_token.result
    node_count       = var.control_plane_count
    node_index       = count.index
  })
}

resource "hcloud_server" "worker" {
  count       = var.worker_count
  name        = "${var.cluster_name}-worker-${count.index + 1}"
  server_type = var.worker_type
  image       = var.image_os
  location    = var.location
  ssh_keys    = [hcloud_ssh_key.k3s.id]
  firewall_ids = [hcloud_firewall.k3s.id]

  user_data = templatefile("${path.module}/templates/worker.yaml.tpl", {
    k3s_version    = var.k3s_version
    k3s_url        = "https://${hcloud_server.control_plane[0].ipv4_address}:6443"
    k3s_token      = random_password.k3s_token.result
  })

  depends_on = [hcloud_server.control_plane]
}

# --- Secrets ---
resource "random_password" "cluster_secret" {
  length  = 48
  special = false
}

resource "random_password" "k3s_token" {
  length  = 48
  special = false
}
