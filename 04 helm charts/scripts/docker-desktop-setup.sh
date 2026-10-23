#!/usr/bin/env bash
set -euo pipefail

CONTEXT="${KUBE_CONTEXT:=docker-desktop}"

echo "Verificando contexto Kubernetes '${CONTEXT}'..."
if ! kubectl config get-contexts "${CONTEXT}" >/dev/null 2>&1; then
	echo "Contexto '${CONTEXT}' não encontrado. Habilite o Kubernetes no Docker Desktop (Settings > Kubernetes) e tente novamente." >&2
	exit 1
fi

kubectl config use-context "${CONTEXT}" >/dev/null

echo "Contexto selecionado: ${CONTEXT}"
kubectl cluster-info
kubectl get nodes
