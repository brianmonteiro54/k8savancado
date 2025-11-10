#!/usr/bin/env bash
set -euo pipefail

# Demo script: build images (v1, v2), deploy v1, create DR/VS, add v2, shift traffic.
APP=versioned-echo

echo ">>> Build service images v1 and v2"
docker build -t ${APP}:v1 ./versioned-echo
docker build --build-arg DUMMY=1 -t ${APP}:v2 ./versioned-echo  # reusing same Dockerfile; version is via env

echo ">>> Apply base (v1 + Service + Istio DR/VS)"
kubectl apply -f k8s/base/deployment-v1.yaml
kubectl apply -f k8s/base/destinationrule.yaml
kubectl apply -f k8s/base/virtualservice.yaml

echo ">>> Wait for v1 ready"
kubectl rollout status deploy/${APP}-v1

echo ">>> Create canary v2"
kubectl apply -f k8s/canary/deployment-v2.yaml
kubectl rollout status deploy/${APP}-v2

echo ">>> Shift traffic 90/10"
kubectl apply -f k8s/canary/vs-90-10.yaml
sleep 10

echo ">>> Shift traffic 70/30"
kubectl apply -f k8s/canary/vs-70-30.yaml
sleep 10

echo ">>> Shift traffic 50/50"
kubectl apply -f k8s/canary/vs-50-50.yaml
sleep 10

echo ">>> Promote 0/100"
kubectl apply -f k8s/canary/vs-0-100.yaml

echo ">>> Done. Try curl inside cluster:"
echo "kubectl exec -it $(kubectl get pod -l app=${APP} -o name | head -n1) -- curl -s http://${APP}/version"
