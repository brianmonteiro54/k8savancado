#!/usr/bin/env bash
set -euo pipefail

# Root of the repo (one level up from this script directory)
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

RUST_DIR="$ROOT/rust"
HOST_PORT="${HOST_PORT:=8080}"

# Allow overriding image/tag externally: IMAGE=myorg/myapp:dev ./scripts/build.sh
IMAGE="${IMAGE:=myorg/myapp:1.0.0}"

# Allow overriding version (picked up by container env). If not provided, try to read from Cargo.toml.
if [[ -z "${APP_VERSION:-}" ]]; then
	if grep -q '^version' "$RUST_DIR/myapp/Cargo.toml"; then
		APP_VERSION=$(grep '^version' "$RUST_DIR/myapp/Cargo.toml" | head -1 | cut -d '"' -f2)
	else
		APP_VERSION="0.0.0"
	fi
fi

echo "Building Linux release binary via multi-stage Dockerfile at: $RUST_DIR/Dockerfile"

if [[ ! -f "$RUST_DIR/Dockerfile" ]]; then
	echo "ERROR: Expected Dockerfile at $RUST_DIR/Dockerfile not found." >&2
	exit 1
fi

docker build \
	-f "$RUST_DIR/Dockerfile" \
	-t "$IMAGE" \
	--build-arg APP_VERSION="$APP_VERSION" \
	"$RUST_DIR"

cat <<INFO
============================================================
Image built:   $IMAGE
Version:       $APP_VERSION
Dockerfile:    $RUST_DIR/Dockerfile

Push to local registry (if used):
	docker push $IMAGE  # Docker Desktop compartilha imagens locais; push só é necessário para registries remotos.

Redeploy to cluster:
	kubectl delete deployment myapp 2>/dev/null || true
	kubectl apply -f "$ROOT/03 rollout updates/k8s/deployment-rolling.yaml"

Force rollout (no spec change):
	kubectl rollout restart deployment/myapp

Run locally (foreground, Ctrl+C to stop):
	docker run --rm -p ${HOST_PORT}:8080 -e PORT=8080 $IMAGE
============================================================
INFO

if [[ "${SKIP_RUN:-}" == "1" ]]; then
	echo "SKIP_RUN=1 set: not running container."
	exit 0
fi

if [[ "${RUN_DETACHED:-}" == "1" ]]; then
	echo "Running container detached (name: myapp-test)..."
	docker run -d --rm --name myapp-test -p "${HOST_PORT}:8080" -e PORT=8080 "$IMAGE" || {
		echo "Failed to run container detached. Use HOST_PORT=<porta> para alterar a porta exposta ou SKIP_RUN=1 para pular." >&2
		exit 1
	}
	echo "Logs: docker logs -f myapp-test"
else
	echo "Running container em http://localhost:${HOST_PORT} (Ctrl+C to stop)..."
	docker run --rm -p "${HOST_PORT}:8080" -e PORT=8080 "$IMAGE" || {
		echo "Falha ao subir o container. Configure HOST_PORT=<porta> para usar outra porta ou defina SKIP_RUN=1." >&2
		exit 1
	}
fi
INFO
