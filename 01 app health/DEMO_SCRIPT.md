# Roteiro de Demonstração — Saúde da Aplicação e Gerenciamento de Recursos (Rust + K8s)

> Objetivo: reproduzir localmente, com **Minikube + Prometheus**, os efeitos das probes de saúde, dos limites de recursos e dos sinais de observabilidade da aplicação Rust.

## 0) Preparação do ambiente (5 min)

- Pré-requisitos instalados: Docker Desktop (Linux containers), Minikube, kubectl, PowerShell 7+, toolchain Rust.
- Clonar este repositório e abrir a pasta `app-health-demo` no VS Code.
- Executar os comandos abaixo em um terminal PowerShell aberto na raiz do projeto.

```powershell
# 0.1 - subir o cluster local
minikube start --driver=docker

# 0.2 - validar o nó
kubectl get nodes

# 0.3 - usar o daemon Docker interno do Minikube
minikube -p minikube docker-env --shell powershell | Invoke-Expression

# 0.4 - compilar a imagem esperada pelo Deployment
# (usa Dockerfile fornecido na raiz)
docker build -t app-health-demo:local .

# 0.5 - aplicar o baseline + Prometheus (usa kustomize)
kubectl apply -k k8s/base

# 0.6 - aguardar pods ficarem Running
kubectl get pods -n app-health-demo -w
```

- Abrir dois terminais adicionais para port-forward.

```powershell
# Terminal A - expõe a API Rust
kubectl port-forward svc/app-health-demo -n app-health-demo 8080:8080

# Terminal B - expõe a UI do Prometheus
kubectl port-forward svc/prometheus -n app-health-demo 9090:9090
```

- Validar acessos iniciais em `http://127.0.0.1:8080/info` e `http://127.0.0.1:9090/targets` (target deve estar **UP**).

## 1) Instrumentação e Golden Signals (7 min)

- Revisar `src/main.rs` e destacar o handler `/metrics` e os registradores do crate `prometheus`.
- Gerar tráfego sintético para aquecer métricas.

```powershell
for ($i = 0; $i -lt 20; $i++) { curl http://127.0.0.1:8080/simulate/latency?ms=150 | Out-Null }
```

- Explorar `http://127.0.0.1:8080/metrics`, comentando `http_requests_total`, `http_request_duration_seconds_bucket`, `http_requests_in_flight`, `http_errors_total`.
- No Prometheus (`http://127.0.0.1:9090`), executar consultas `rate(http_requests_total[1m])` e `histogram_quantile(0.9, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))`; conectar com os quatro Golden Signals.

## 2) Health Probes em ação (8 min)

- Verificar os endpoints de saúde manualmente.

```powershell
curl http://127.0.0.1:8080/health/startup
curl http://127.0.0.1:8080/health/ready
curl http://127.0.0.1:8080/health/live
```

- Explicar `STARTUP_DELAY_MS` e `READY_AFTER_MS` definidos em `k8s/base/deployment.yaml`.
- Acompanhar logs para ver transições de estado.

```powershell
kubectl logs deploy/app-health-demo -n app-health-demo --tail=50 -f
```

- Simular indisponibilidade de readiness e observar o Service retirando o pod.

```powershell
curl http://127.0.0.1:8080/admin/flip_ready?value=false
kubectl get pods -n app-health-demo -w
```

- Reativar readiness com `value=true`.
- Simular queda de liveness para provocar reinício e monitorar `RESTARTS`.

```powershell
curl http://127.0.0.1:8080/admin/flip_live?value=false
kubectl get pods -n app-health-demo -w
```

- Reativar liveness com `value=true` antes de prosseguir.

## 3) Requests, Limits e classes de QoS (12 min)

- Aplicar overlay **Burstable** (`requests < limits`) e aguardar pods.

```powershell
kubectl apply -k k8s/overlays/burstable
kubectl get pods -n app-health-demo -w
```

- Induzir carga longa de CPU e observar throttling (`kubectl top pod` e histograma de latência).

```powershell
Measure-Command { curl "http://127.0.0.1:8080/simulate/cpu?seconds=25" }
kubectl top pod -n app-health-demo
```

- Comentar aumento em `http_request_duration_seconds`.
- Alternar para overlay **Guaranteed** (`requests = limits`).

```powershell
kubectl delete -k k8s/overlays/burstable
kubectl apply -k k8s/overlays/guaranteed
kubectl get pods -n app-health-demo -w
```

- Repetir teste de CPU mostrando maior previsibilidade.
- Demonstrar OOM controlado com vazamento de memória e analisar eventos do pod.

```powershell
curl "http://127.0.0.1:8080/simulate/mem?mb=512&leak=true"
kubectl describe pod -n app-health-demo <nome-do-pod>
```

## 4) HPA e saturação (7 min)

- Monitorar o HPA (`k8s/base/hpa.yaml` alvo 70% CPU).

```powershell
kubectl get hpa -n app-health-demo -w
```

- Gerar carga contínua para saturar CPU e observar réplicas aumentando.

```powershell
1..60 | ForEach-Object { Start-Job { curl http://127.0.0.1:8080/simulate/cpu?seconds=5 | Out-Null } }
```

- Correlacionar `saturation_gauge`, métricas de CPU e comportamento do HPA.
- Discutir trade-offs de eficiência vs. previsibilidade em cada classe de QoS.

## 5) Falhas e comportamento não determinístico (4 min)

- Exercitar `/unstable` para observar respostas lentas e erros intermitentes.

```powershell
for ($i = 0; $i -lt 30; $i++) { curl http://127.0.0.1:8080/unstable; Start-Sleep -Milliseconds 200 }
```

- Inspecionar `http_errors_total` e `saturation_gauge` no Prometheus.
- Relacionar com práticas de retries, circuit breakers e outlier ejection em service meshes.

## 6) Encerramento e limpeza (3 min)

- Reforçar: probes detectam rapidamente, Requests/Limits definem QoS, HPA reage a sinais, métricas suportam SLOs.
- Próximos passos sugeridos: adicionar Grafana, testar service mesh (Linkerd/Istio) para retries e ejection.
- Encerrar limpando o ambiente.

```powershell
kubectl delete -k k8s/overlays/guaranteed
kubectl delete -k k8s/base
minikube stop
```

