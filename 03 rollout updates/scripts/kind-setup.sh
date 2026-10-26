#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "[kind-setup.sh] Este script foi descontinuado. Usando Docker Desktop..."
exec "${SCRIPT_DIR}/docker-desktop-setup.sh"
