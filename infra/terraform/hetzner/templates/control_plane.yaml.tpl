#cloud-config
package_upgrade: true

write_files:
  - path: /etc/rancher/k3s/config.yaml
    content: |
      cluster-init: true
      token: "${k3s_token}"
      tls-san:
%{ for san in tls_sans ~}
        - ${san}
%{ endfor ~}
      disable:
        - traefik
        - servicelb
      flannel-backend: wireguard-native
      cluster-cidr: 10.42.0.0/16
      service-cidr: 10.43.0.0/16
      cluster-dns: 10.43.0.10
      node-name: "${cluster_name}-cp-${node_index}"

runcmd:
  - curl -sfL https://get.k3s.io | INSTALL_K3S_VERSION="${k3s_version}" sh -
  - sleep 10
%{ if node_index == 0 ~}
  - |
    cat /etc/rancher/k3s/k3s.yaml | sed 's/127.0.0.1/${tls_sans[0]}/' > /root/kubeconfig
%{ endif ~}
