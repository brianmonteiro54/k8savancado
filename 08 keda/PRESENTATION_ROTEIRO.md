# Roteiro de Apresentação — KEDA + Rust (Aula 08)

**Objetivo:** demonstrar, ao vivo, escalabilidade event-driven com KEDA, incluindo scale-to-zero, gatilho RabbitMQ e interplay com HPA.

## Agenda

1. Contexto e motivação
2. Arquitetura do demo
3. Leitura guiada dos YAMLs críticos
4. Build & deploy
5. Demonstração do autoscaling
6. Tópicos avançados e Q&A

## Comandos úteis (cola)

```bash
# Garantir que o cluster ativo é o Docker Desktop
kubectl config use-context docker-desktop

# Instalar KEDA
helm repo add kedacore https://kedacore.github.io/charts
helm repo update
helm install keda kedacore/keda -n keda --create-namespace
kubectl -n keda get pods

# Aplicar manifests
kubectl apply -f k8s/00-namespace.yaml
kubectl apply -f k8s/10-rabbitmq-secret.yaml
kubectl apply -f k8s/11-rabbitmq.yaml
kubectl apply -f k8s/12-rabbitmq-svc.yaml
kubectl apply -f k8s/20-worker-deploy.yaml
kubectl apply -f k8s/21-worker-svc.yaml
kubectl apply -f k8s/keda/30-trigger-auth.yaml
kubectl apply -f k8s/keda/31-scaledobject.yaml

# Observar escala
kubectl -n keda-demo get hpa -w
kubectl -n keda-demo get deploy/worker -w
kubectl -n keda-demo describe hpa -l app=worker

# Gerar backlog (Job)
kubectl apply -f k8s/40-publisher-job.yaml -n keda-demo
kubectl -n keda-demo logs job/publisher --follow
```

## Pontos de ênfase didáticos

- **Scale-to-zero real** (minReplicaCount: 0) e comportamento de reativação por evento (mensagem na fila).
- **`value` = 10 msgs/pod**: intuído como “alvo” para dimensionamento.
- **`cooldownPeriod`** e **`pollingInterval`**: amortecem oscilações (jitter).
- **Observabilidade**: `describe hpa` mostra o porquê do scale up/down.
- **Portabilidade**: o mesmo padrão funciona em AKS/EKS/GKE/on‑prem.
