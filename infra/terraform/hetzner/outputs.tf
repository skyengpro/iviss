output "control_plane_ips" {
  value = hcloud_server.control_plane[*].ipv4_address
}

output "worker_ips" {
  value = hcloud_server.worker[*].ipv4_address
}

output "kubeconfig" {
  description = "Kubeconfig for the cluster (retrieve via SSH)"
  value       = "ssh ${var.cluster_name}-cp-1 'cat /etc/rancher/k3s/k3s.yaml'"
}

output "private_key_openssh" {
  value     = tls_private_key.k3s.private_key_openssh
  sensitive = true
}

output "cluster_name" {
  value = var.cluster_name
}
