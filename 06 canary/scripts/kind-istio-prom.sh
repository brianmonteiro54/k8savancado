istioctl install -y
#!/usr/bin/env bash
set -euo pipefail

cat <<'EOF' >&2
[deprecated] scripts/kind-istio-prom.sh foi substituído por scripts/docker-desktop-istio-prom.sh.
Este script permanece apenas para referência histórica e não executa mais nenhuma ação.
Execute:
  bash scripts/docker-desktop-istio-prom.sh
para preparar o ambiente usando o cluster Kubernetes do Docker Desktop.
EOF

exit 1
