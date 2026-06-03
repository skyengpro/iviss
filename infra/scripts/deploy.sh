#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# IVISS Deploy Script
#
# Deploys (or re-syncs) the IVISS ArgoCD applications.
# Infrastructure (CNPG, secrets, etc.) is managed by ArgoCD
# GitOps — push to main and ArgoCD auto-syncs.
#
# Prerequisites:
#   - K8s cluster provisioned on Hetzner (infra/terraform/hetzner/)
#   - Operators installed (bootstrap-k8s.sh)
#   - kubectl connected to the cluster
#   - ArgoCD CLI installed (optional, for manual sync)
# ============================================================

echo "==> Deploying IVISS ArgoCD applications..."
kubectl apply -k argocd/

echo ""
echo "==> Waiting for ArgoCD to sync..."
sleep 5

echo ""
echo "==> Application status:"
kubectl get applications -n argocd

echo ""
echo "================================================"
echo "Deploy complete!"
echo ""
echo "ArgoCD will auto-sync on every push to main."
echo "Manual sync:  argocd app sync iviss-database && argocd app sync iviss-backend && argocd app sync iviss-frontend"
echo "Check status: argocd app get iviss-backend"
echo "================================================"