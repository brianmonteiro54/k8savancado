#!/usr/bin/env bash
set -euo pipefail

CLUSTER_NAME="${CLUSTER_NAME:-kind-aula05}"
REG_NAME='kind-registry'
REG_PORT='5001'

# registry local (se não existir)
running="$(docker inspect -f '{{.State.Running}}' "${REG_NAME}" 2>/dev/null || true)"
if [ "${running}" != 'true' ]; then
  docker run -d --restart=always -p "127.0.0.1:${REG_PORT}:5000" --name "${REG_NAME}" registry:2
fi

# cluster kind (se não existir)
if ! kind get clusters | grep -q "^${CLUSTER_NAME}$"; then
cat <<EOF | kind create cluster --name "${CLUSTER_NAME}" --config=-
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
containerdConfigPatches:
- |-
  [plugins."io.containerd.grpc.v1.cri".registry.mirrors."localhost:${REG_PORT}"]
    endpoint = ["http://kind-registry:5000"]
nodes:
- role: control-plane
- role: worker
- role: worker
EOF

  # conecta registry ao cluster
  docker network connect "kind" "${REG_NAME}" 2>/dev/null || true

  # configurações do registry como ConfigMap p/ descoberta (opcional)
  kubectl apply -f - <<EOF
apiVersion: v1
kind: ConfigMap
metadata:
  name: local-registry-hosting
  namespace: kube-public
data:
  localRegistryHosting.v1: |
    host: "localhost:${REG_PORT}"
    help: "https://kind.sigs.k8s.io/docs/user/local-registry/"
EOF
fi

echo "Kind cluster '${CLUSTER_NAME}' pronto. Registry em localhost:${REG_PORT}."
echo "Dica: docker build -t localhost:${REG_PORT}/myapp:latest . && docker push localhost:${REG_PORT}/myapp:latest"
