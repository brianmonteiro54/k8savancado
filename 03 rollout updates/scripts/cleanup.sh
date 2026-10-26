#!/usr/bin/env bash
set -euo pipefail
kubectl delete -f k8s/argo/rollout-canary.yaml --ignore-not-found
kubectl delete -f k8s/argo/analysis-template.yaml --ignore-not-found
kubectl delete -f k8s/deployment-recreate.yaml --ignore-not-found
kubectl delete -f k8s/deployment-rolling.yaml --ignore-not-found
kubectl delete -f k8s/service.yaml --ignore-not-found
echo "Recursos removidos do cluster atual (${KUBE_CONTEXT:=docker-desktop})."
