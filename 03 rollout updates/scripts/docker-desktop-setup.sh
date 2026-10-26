#!/usr/bin/env bash
set -euo pipefail

CONTEXT="${KUBE_CONTEXT:=docker-desktop}"

echo "Verificando contexto Kubernetes '${CONTEXT}'..."
if ! kubectl config get-contexts "${CONTEXT}" >/dev/null 2>&1; then
	echo "Contexto '${CONTEXT}' não encontrado. Habilite o Kubernetes no Docker Desktop e tente novamente." >&2
	exit 1
fi

kubectl config use-context "${CONTEXT}" >/dev/null

echo "Contexto selecionado: ${CONTEXT}"
kubectl cluster-info
kubectl get nodes

# Este módulo constrói a imagem localmente e aplica os manifests com
# imagePullPolicy: IfNotPresent, sem push para registry. Isso depende de o cluster
# usar o image store local do Docker, o que vale para o provisionador kubeadm e não
# para o kind. Referência: https://docs.docker.com/desktop/features/kubernetes/
if kubectl get nodes -o name 2>/dev/null | grep -q 'control-plane'; then
	cat >&2 <<'WARN'

AVISO: este cluster parece ser baseado em kind (nó "*-control-plane").
       O módulo constrói a imagem localmente e não faz push para registry, então
       os pods podem falhar com ImagePullBackOff mesmo com a imagem presente no
       "docker images", porque o cluster não usa o image store local do Docker.

       Se o cluster foi criado pelo Docker Desktop:
         Kubernetes > Edit cluster > provisionador "kubeadm"
         (o "kind load" tende a falhar com "no nodes found", porque os nós do
          Docker Desktop não ficam visíveis para o CLI do kind)

       Se for um kind standalone, carregue a imagem no nó:
         kind load docker-image myorg/myapp:1.0.0 --name <seu-cluster>

       Referência: https://docs.docker.com/desktop/features/kubernetes/

WARN
fi
