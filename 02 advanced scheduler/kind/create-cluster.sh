#!/usr/bin/env bash
set -euo pipefail

CLUSTER_NAME="${1:-aula02}"
echo "[*] Creating kind cluster: ${CLUSTER_NAME}"
kind create cluster --name "${CLUSTER_NAME}" --config kind-cluster.yaml

echo "[*] Waiting for nodes to be Ready..."
kubectl wait --for=condition=Ready node --all --timeout=120s || true

echo "[*] Done. Current nodes:"
kubectl get nodes -o wide
