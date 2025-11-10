#!/usr/bin/env bash
set -euo pipefail

NS=aula05
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/deployment-blue.yaml
kubectl apply -f k8s/deployment-green.yaml
kubectl apply -f k8s/service.yaml

echo "Port-forward em background..."
kubectl -n "$NS" port-forward svc/myapp 8080:80 >/tmp/pf.log 2>&1 &
PF_PID=$!
sleep 2

echo "Hit BLUE:"
curl -s http://localhost:8080/ | jq . || true

echo "Cutover para GREEN..."
kubectl -n "$NS" patch svc myapp -p '{"spec":{"selector":{"app":"myapp","env":"green"}}}'

sleep 1
echo "Hit GREEN:"
curl -s http://localhost:8080/ | jq . || true

echo "Rollback para BLUE..."
kubectl -n "$NS" patch svc myapp -p '{"spec":{"selector":{"app":"myapp","env":"blue"}}}'

sleep 1
echo "Hit BLUE novamente:"
curl -s http://localhost:8080/ | jq . || true

kill $PF_PID || true
