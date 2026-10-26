#!/usr/bin/env bash
set -euo pipefail
IMG="${1:-service-a:local}"
if ! command -v kind >/dev/null 2>&1; then
  echo "kind não encontrado. Use este script apenas quando seu cluster não compartilha o daemon Docker (ex.: kind)."
  exit 1
fi
kind load docker-image "$IMG"
echo "✅ Imagem '$IMG' carregada no cluster kind."
