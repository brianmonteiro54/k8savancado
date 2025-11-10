# Passo-a-passo de YAML (com explicações)

Este documento destaca cada manifesto e **por que** ele é necessário.

## 00-namespace.yaml
Isola os recursos do demo sob `keda-demo`, evitando colisão com outros workloads.

## 10-rabbitmq-secret.yaml
Armazena a connection string **AMQP** de forma segura. O `TriggerAuthentication` do KEDA irá referenciá-la como `host`.

## 11-rabbitmq.yaml e 12-rabbitmq-svc.yaml
Sobe o RabbitMQ no cluster e o expõe internamente. A UI (`15672`) facilita inspeção manual (opcional).

## 20-worker-deploy.yaml
Define o `Deployment` do consumidor em Rust. **Importante**: o KEDA/HPA controlam o número de réplicas. A imagem começa em `0` pods (scale-to-zero).

## 21-worker-svc.yaml
Exposição HTTP opcional para health checks (`/healthz`). Ajuda a demonstrar readiness/liveness.

## keda/30-trigger-auth.yaml
O **TriggerAuthentication** indica ao KEDA **como** obter credenciais/endpoint para falar com RabbitMQ — sem hard‑code no `ScaledObject`.

## keda/31-scaledobject.yaml
Coração do demo. O `ScaledObject`:
- aponta para o `Deployment` alvo (`scaleTargetRef`);
- configura gatilho `rabbitmq` e parâmetros (`queueName`, `mode`, `value`);
- habilita `minReplicaCount: 0` (scale-to-zero);
- ajusta `pollingInterval` e `cooldownPeriod` para estabilidade.

## 40-publisher-job.yaml
Gera backlog controlado, criando condições para visualizar o **autoscaling** em tempo real.
