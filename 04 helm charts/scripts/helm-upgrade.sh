#!/usr/bin/env bash
set -euo pipefail

RELEASE="${RELEASE:-myrustapp-release}"
NAMESPACE="${NAMESPACE:-dev}"
IMAGE_REPO="${IMAGE_REPO:-myrustapp}"
IMAGE_TAG="${IMAGE_TAG:-1.0.1}"

echo "[+] Upgrading to image ${IMAGE_REPO}:${IMAGE_TAG} ..."
helm upgrade "${RELEASE}" ./helm/myrustapp-chart   --namespace "${NAMESPACE}"   --set image.repository="${IMAGE_REPO}"   --set image.tag="${IMAGE_TAG}"   --atomic --cleanup-on-fail

kubectl rollout status deploy/${RELEASE}-myrustapp -n "${NAMESPACE}"
