#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# IVISS K8s Cluster Bootstrap
# Run this AFTER `terraform apply` provisions the Hetzner nodes.
# Installs: CNPG, Nginx Ingress Controller, ArgoCD, cert-manager
# ============================================================

CLUSTER_NAME="${CLUSTER_NAME:-iviss}"
INGRESS_VERSION="${INGRESS_VERSION:-1.12.1}"
ARGOCD_VERSION="${ARGOCD_VERSION:-2.14.3}"
CERT_MANAGER_VERSION="${CERT_MANAGER_VERSION:-1.17.1}"
CNPG_VERSION="${CNPG_VERSION:-1.24.0}"

echo "==> Verifying kubeconfig..."
if ! kubectl cluster-info > /dev/null 2>&1; then
  echo "ERROR: kubectl is not connected to a cluster."
  echo "Export KUBECONFIG or copy it from the control plane:"
  echo "  scp ubuntu@<cp-ip>:/etc/rancher/k3s/k3s.yaml ~/.kube/config"
  exit 1
fi

echo "==> Installing Cloud Native PG (CNPG) operator..."
kubectl apply -f "https://raw.githubusercontent.com/cloudnative-pg/cloudnative-pg/main/releases/cnpg-${CNPG_VERSION}.yaml"
echo "Waiting for CNPG controller to be ready..."
kubectl wait deployment -n cnpg-system cnpg-controller-manager --for=condition=Available --timeout=120s

echo "==> Installing Nginx Ingress Controller..."
helm upgrade --install ingress-nginx ingress-nginx \
  --repo https://kubernetes.github.io/ingress-nginx \
  --version "${INGRESS_VERSION}" \
  --namespace ingress-nginx --create-namespace \
  --set controller.service.type=LoadBalancer \
  --set controller.service.annotations."loadbalancer\.hetzner\.cloud/location"="fsn1" \
  --set controller.service.annotations."loadbalancer\.hetzner\.cloud/use-private-ip"="false" \
  --set controller.config.use-forwarded-headers="true" \
  --wait

echo "==> Getting Ingress LoadBalancer IP..."
sleep 10
LB_IP=$(kubectl get svc -n ingress-nginx ingress-nginx-controller -o jsonpath='{.status.loadBalancer.ingress[0].ip}')
echo "Ingress LoadBalancer IP: ${LB_IP}"

echo ""
echo "==> Update your DNS: Create an A record for k8s.iviss.cloud -> ${LB_IP}"
echo "    Then set K8S_ORIGIN_HOSTNAME in Terraform variables."
echo ""

echo "==> Installing ArgoCD..."
helm upgrade --install argocd argo-cd \
  --repo https://argoproj.github.io/argo-helm \
  --version "${ARGOCD_VERSION}" \
  --namespace argocd --create-namespace \
  --set server.config."resource\.customizations"="" \
  --set configs.cm."timeout\.reconciliation"="60s" \
  --set server.ingress.enabled=true \
  --set server.ingress.ingressClassName=nginx \
  --set server.ingress.hosts[0]=argocd.iviss.cloud \
  --set server.ingress.tls[0].hosts[0]=argocd.iviss.cloud \
  --set server.ingress.tls[0].secretName=argocd-tls \
  --wait

echo "==> Installing cert-manager..."
helm upgrade --install cert-manager cert-manager \
  --repo https://charts.jetstack.io \
  --version "${CERT_MANAGER_VERSION}" \
  --namespace cert-manager --create-namespace \
  --set crds.enabled=true \
  --wait

echo ""
echo "==> ArgoCD initial password:"
kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d
echo ""

echo ""
echo "==> Applying IVISS ArgoCD project..."
kubectl apply -k argocd/

echo ""
echo "================================================"
echo "Bootstrap complete!"
echo ""
echo "Next steps:"
echo "  1. Update values-production.yaml with secrets"
echo "  2. Push to repo (ArgoCD auto-syncs)"
echo "  3. Set up cert-manager ClusterIssuer for Let's Encrypt"
echo "  4. Verify: kubectl get pods -n iviss"
echo "  5. Check CNPG cluster: kubectl get cluster -n iviss"
echo "================================================"
