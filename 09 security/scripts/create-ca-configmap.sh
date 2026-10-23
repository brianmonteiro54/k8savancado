#!/usr/bin/env bash
set -euo pipefail
NS="${1:-default}"
# Extrai a CA do Secret gerado pelo cert-manager e cria um ConfigMap para consumo por pods
tmpdir="$(mktemp -d)"
kubectl -n "$NS" get secret root-ca-secret -o jsonpath='{.data.ca\.crt}' | base64 -d > "$tmpdir/ca.crt"
kubectl -n "$NS" delete configmap root-ca-bundle --ignore-not-found
kubectl -n "$NS" create configmap root-ca-bundle --from-file=ca.crt="$tmpdir/ca.crt"
kubectl -n "$NS" label configmap root-ca-bundle app=root-ca-bundle --overwrite
echo "✅ ConfigMap 'root-ca-bundle' criado em $NS. Monte-o em /ca (ou use com curl --cacert)."
