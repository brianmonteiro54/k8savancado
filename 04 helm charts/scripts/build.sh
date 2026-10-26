#!/usr/bin/env bash
set -euo pipefail

IMAGE_REPO="${IMAGE_REPO:-myrustapp}"
IMAGE_TAG="${IMAGE_TAG:-1.0.0}"

echo "[+] Building Docker image ${IMAGE_REPO}:${IMAGE_TAG}..."
docker build -t "${IMAGE_REPO}:${IMAGE_TAG}" ./myrustapp

cat <<NOTE
[i] Imagem disponível localmente.
    - Clusters Docker Desktop consomem direto do daemon local (sem push adicional).
    - Para registries remotos, execute: docker push ${IMAGE_REPO}:${IMAGE_TAG}
NOTE
