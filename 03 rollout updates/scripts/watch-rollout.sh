#!/usr/bin/env bash
set -euo pipefail
if command -v watch >/dev/null 2>&1; then
	watch -n2 '
echo "ReplicaSets:"
kubectl get rs -l app=myapp -o wide
echo
echo "Pods:"
kubectl get pods -l app=myapp -o wide
'
	exit 0
fi

echo "Comando 'watch' não encontrado; usando loop simples. Pressione Ctrl+C para sair." >&2

while true; do
	clear || printf '\033c'
	echo "ReplicaSets:"
	kubectl get rs -l app=myapp -o wide || true
	echo
	echo "Pods:"
	kubectl get pods -l app=myapp -o wide || true
	sleep 2
done
