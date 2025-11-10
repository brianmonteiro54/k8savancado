#!/usr/bin/env bash
set -euo pipefail
kubectl apply -f k8s/deployment-recreate.yaml
kubectl rollout status deployment/myapp-recreate
