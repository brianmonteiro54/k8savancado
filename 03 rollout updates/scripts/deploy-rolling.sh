#!/usr/bin/env bash
set -euo pipefail
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/deployment-rolling.yaml
kubectl rollout status deployment/myapp
