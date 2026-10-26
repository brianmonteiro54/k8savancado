\
#!/usr/bin/env bash
set -euo pipefail

echo "[*] Labeling and tainting nodes for Aula 02 demo..."

mapfile -t WORKERS < <(kubectl get nodes -o jsonpath='{range .items[*]}{.metadata.name}{"\n"}{end}' | grep -v control-plane || true)
if [ ${#WORKERS[@]} -lt 1 ]; then
  echo "No worker nodes found; using all nodes."
  mapfile -t WORKERS < <(kubectl get nodes -o jsonpath='{range .items[*]}{.metadata.name}{"\n"}{end}')
fi

ZONES=(az-a az-b az-c)
for i in "${!WORKERS[@]}"; do
  Z="${ZONES[$((i % ${#ZONES[@]}))]}"
  echo " - Labeling ${WORKERS[$i]} with topology.kubernetes.io/zone=${Z}"
  kubectl label node "${WORKERS[$i]}" "topology.kubernetes.io/zone=${Z}" --overwrite
done

if [ ${#WORKERS[@]} -ge 1 ]; then
  echo " - Labeling ${WORKERS[0]} disktype=ssd"
  kubectl label node "${WORKERS[0]}" disktype=ssd --overwrite
fi

NOISY="${WORKERS[-1]}"
echo " - Labeling ${NOISY} workload=noisy and applying taint workload=noisy:NoSchedule"
kubectl label node "${NOISY}" workload=noisy --overwrite
kubectl taint node "${NOISY}" workload=noisy:NoSchedule --overwrite || true

echo "[*] Done."
