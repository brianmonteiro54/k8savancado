#!/usr/bin/env bash
set -euo pipefail

# Prepares the Docker Desktop Kubernetes cluster by enabling Istio and installing
# kube-prometheus-stack (Prometheus/Grafana) via Helm. Assumes Docker Desktop
# Kubernetes is already enabled on the host.

CONTEXT="${KUBE_CONTEXT:=docker-desktop}"

echo "Verificando contexto Kubernetes '${CONTEXT}'..."
if ! kubectl config get-contexts "${CONTEXT}" >/dev/null 2>&1; then
	echo "Contexto '${CONTEXT}' não encontrado. Habilite o Kubernetes no Docker Desktop (Settings > Kubernetes) e tente novamente." >&2
	exit 1
fi

kubectl config use-context "${CONTEXT}" >/dev/null

echo "Contexto selecionado: ${CONTEXT}"
kubectl cluster-info

# Istio install is idempotent; istioctl handles upgrades when rerun.
echo "Instalando Istio (perfil default)..."
istioctl install -y >/dev/null

echo "Habilitando sidecar injection no namespace 'default'..."
kubectl label ns default istio-injection=enabled --overwrite

# Install or upgrade kube-prometheus-stack with Grafana exposed internally.
echo "Instalando kube-prometheus-stack via Helm..."
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts >/dev/null 2>&1 || true
helm repo update >/dev/null
helm upgrade --install kps prometheus-community/kube-prometheus-stack \
	--namespace monitoring --create-namespace \
	--set grafana.enabled=true \
	--set prometheus.prometheusSpec.serviceMonitorSelectorNilUsesHelmValues=false

cat <<EOF
Pronto! Recursos principais:
- Namespace 'istio-system' com control plane Istio.
- Namespace 'monitoring' com Prometheus e Grafana (release 'kps').

Dicas de port-forward:
  kubectl -n monitoring port-forward svc/kps-grafana 3000:80
  kubectl -n monitoring port-forward svc/kps-kube-prometheus-prometheus 9090:9090
EOF
