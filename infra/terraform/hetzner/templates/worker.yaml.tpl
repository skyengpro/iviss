#cloud-config
package_upgrade: true

runcmd:
  - curl -sfL https://get.k3s.io | INSTALL_K3S_VERSION="${k3s_version}" K3S_URL="${k3s_url}" K3S_TOKEN="${k3s_token}" sh -
