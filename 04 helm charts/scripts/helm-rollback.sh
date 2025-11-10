#!/usr/bin/env bash
set -euo pipefail

RELEASE="${RELEASE:-myrustapp-release}"
NAMESPACE="${NAMESPACE:-dev}"
REVISION="${REVISION:-1}"

echo "[+] Rolling back ${RELEASE} to revision ${REVISION} ..."
helm rollback "${RELEASE}" "${REVISION}" -n "${NAMESPACE}"
kubectl rollout status deploy/${RELEASE}-myrustapp -n "${NAMESPACE}"
