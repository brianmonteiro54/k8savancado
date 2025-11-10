#!/usr/bin/env bash
set -euo pipefail

RELEASE="${RELEASE:-myrustapp-release}"
NAMESPACE="${NAMESPACE:-dev}"
IMAGE_REPO="${IMAGE_REPO:-myrustapp}"
IMAGE_TAG="${IMAGE_TAG:-1.0.0}"

kubectl create namespace "${NAMESPACE}" 2>/dev/null || true

echo "[+] Dry run render:"
helm install --dry-run --debug "${RELEASE}" ./helm/myrustapp-chart   --namespace "${NAMESPACE}"   --set image.repository="${IMAGE_REPO}"   --set image.tag="${IMAGE_TAG}"

echo "[+] Installing release ${RELEASE} in ns ${NAMESPACE}..."
helm install "${RELEASE}" ./helm/myrustapp-chart   --namespace "${NAMESPACE}"   --set image.repository="${IMAGE_REPO}"   --set image.tag="${IMAGE_TAG}"   --create-namespace

echo "[+] Waiting for rollout..."
kubectl rollout status deploy/${RELEASE}-myrustapp -n "${NAMESPACE}"

echo "[+] Port-forward: http://localhost:8080"
kubectl -n "${NAMESPACE}" port-forward svc/${RELEASE}-myrustapp 8080:80
