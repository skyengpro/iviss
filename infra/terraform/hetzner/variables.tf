variable "hcloud_token" {
  description = "Hetzner Cloud API token"
  type        = string
  sensitive   = true
}

variable "cluster_name" {
  description = "Name of the K3s cluster"
  type        = string
  default     = "iviss"
}

variable "location" {
  description = "Hetzner datacenter location"
  type        = string
  default     = "fsn1"
}

variable "image_os" {
  description = "OS image for the nodes"
  type        = string
  default     = "ubuntu-22.04"
}

variable "control_plane_count" {
  description = "Number of control plane nodes (1 for single-node, 3+ for HA)"
  type        = number
  default     = 1
}

variable "control_plane_type" {
  description = "Hetzner server type for control plane nodes"
  type        = string
  default     = "cax21"
}

variable "worker_count" {
  description = "Number of worker nodes"
  type        = number
  default     = 2
}

variable "worker_type" {
  description = "Hetzner server type for worker nodes"
  type        = string
  default     = "cax21"
}

variable "k3s_version" {
  description = "K3s version to install"
  type        = string
  default     = "v1.32.3+k3s1"
}

variable "additional_tls_sans" {
  description = "Additional TLS SANs for the K3s API server certificate"
  type        = list(string)
  default     = []
}

variable "cloudfront_cidrs" {
  description = "CloudFront origin-facing CIDR blocks to restrict ingress to. If null, allows all."
  type        = list(string)
  default     = null
}
