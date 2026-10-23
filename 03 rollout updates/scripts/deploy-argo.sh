#!/usr/bin/env bash
set -euo pipefail
kubectl apply -f k8s/argo/analysis-template.yaml
kubectl apply -f k8s/argo/rollout-canary.yaml
echo "Use: kubectl argo rollouts get rollout myapp -w"
