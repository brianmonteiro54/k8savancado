# ROTEIRO DA APRESENTAÇÃO — Aula 05 (Blue/Green com Rust + Kubernetes)

Tempo total sugerido: **35–45 min**

## 1) Abertura (2–3 min)

- Problema: deploy sem downtime e com rollback instantâneo.
- Estratégia: **Blue/Green** (dois ambientes idênticos; troca de tráfego).
- Tese: *Service selector* como chave do cutover no Kubernetes.

## 2) Arquitetura (3 min)

- Um **container** (Rust/Axum) → duas **cores** (blue/green) via **env vars** e **labels**.
- Dois **Deployments** (`myapp-blue`, `myapp-green`) e um **Service** (`myapp`).
- **Cutover** = mudar `spec.selector` do Service (`env: blue|green`).

## 3) Setup (5–7 min)

Terminal 1:

```bash
scripts/docker-desktop-setup.sh
cd service && docker build -t myapp:latest .
```

Terminal 2:

```bash
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/deployment-blue.yaml
kubectl apply -f k8s/deployment-green.yaml
kubectl apply -f k8s/service.yaml
kubectl -n aula05 get deploy,svc,pods -o wide
```

## 4) Teste inicial (3 min)

```bash
kubectl -n aula05 port-forward svc/myapp 8080:80
# novo terminal
watch -n 1 curl -s http://localhost:8080/ | jq .
```

- Comente `version` e `color` retornados. Deve ser **blue** (v1.0).

## 5) Cutover Blue→Green (3–4 min)

```bash
kubectl -n aula05 patch svc myapp -p '{"spec":{"selector":{"app":"myapp","env":"green"}}}'
```

- Mostre atualização quase instantânea no terminal do `curl`.
- Explique por que o swap é atômico do ponto de vista lógico.

## 6) Rollback imediato (2 min)

```bash
kubectl -n aula05 patch svc myapp -p '{"spec":{"selector":{"app":"myapp","env":"blue"}}}'
```

- Mostre retorno ao **blue**.
- Enfatize recuperação rápida e previsível.

## 7) Orquestrador Rust (5–7 min)

```bash
cd orchestrator
cargo run --release -- init
cargo run --release -- status
cargo run --release -- switch --to green
cargo run --release -- status
```

- Leia o selector do Service; explique os objetos Kubernetes gerados via `kube-rs`.

## 8) Boas práticas & limites (5–8 min)

- **DB migrations**: *backward/forward compatible*, *expand & contract*, toggles.
- **Observabilidade**: readiness/liveness, logs e métricas logo após o cutover.
- **Custo**: duplicação temporária de recursos (planejar janelas).
- **GitOps**: manifests idempotentes, PR de promoção, RBAC.

## 9) Encerramento (2 min)

- Reforce: simplicidade do modelo, poder do rollback, como evoluir p/ Canary/Flags.
