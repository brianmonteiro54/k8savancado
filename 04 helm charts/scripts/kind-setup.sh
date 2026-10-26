#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "[kind-setup.sh] Script legado. Usando Docker Desktop como cluster local."
exec "${SCRIPT_DIR}/docker-desktop-setup.sh"
