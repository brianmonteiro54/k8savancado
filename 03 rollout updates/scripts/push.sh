#!/usr/bin/env bash
set -euo pipefail
IMAGE="${IMAGE:=myorg/myapp:1.0.0}"

echo "Enviando imagem '${IMAGE}' para o registry configurado..."
docker push "$IMAGE" || {
	echo "Falha no push. Em clusters Docker Desktop não é necessário dar push; apenas use a imagem local." >&2
	exit 1
}
echo "Imagem publicada: $IMAGE"
