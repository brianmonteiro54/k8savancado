# Passo‑a‑Passo da Implementação (YAML + Kubernetes)

> Objetivo: aplicar, observar, modificar **sem depender** de provedor específico.

## 1) Base
```
kubectl apply -k k8s/base
kubectl -n app-health-demo get all
```

- `Deployment` com probes HTTP (`startup`, `readiness`, `liveness`).
- `Service` ClusterIP na porta 8080.
- `HPA` (CPU 70%) e `PDB` (`minAvailable: 1`).

## 2) QoS — Burstable vs Guaranteed

Aplicar **burstable** (requests < limits):
```
kubectl apply -k k8s/overlays/burstable
kubectl -n app-health-demo describe pod <pod>
```

Trocar para **guaranteed** (requests == limits):
```
kubectl apply -k k8s/overlays/guaranteed
```

## 3) Probes

```
kubectl -n app-health-demo port-forward svc/app-health-demo 8080:8080

# Startup / Readiness / Liveness
curl -i localhost:8080/health/startup
curl -i localhost:8080/health/ready
curl -i localhost:8080/health/live
```

Ajustes padrão:
- `startupProbe.failureThreshold * periodSeconds` ≈ janela de boot
- `readinessProbe.initialDelaySeconds` + `failureThreshold * periodSeconds` ≈ janela para ficar pronto
- `livenessProbe.periodSeconds` controla a periodicidade de checagem

## 4) HPA

Gerar CPU e observar scale‑out:
```
kubectl -n app-health-demo port-forward svc/app-health-demo 8080:8080
while true; do curl -s "http://localhost:8080/simulate/cpu?seconds=2" > /dev/null; done
# em outra janela
kubectl -n app-health-demo get hpa -w
kubectl -n app-health-demo get deploy app-health-demo -w
```

## 5) Logs, métricas e reinícios

```
kubectl -n app-health-demo logs -f deploy/app-health-demo
kubectl -n app-health-demo get pods
kubectl -n app-health-demo describe pod <pod> | egrep -i "oom|killed|throttle|limit|probe"
```

## 6) Limpeza

```
kubectl delete ns app-health-demo
```
