# Roteiro de Apresentação Técnica — Aula 06 (Canary)

> Duração sugerida: 45–60 min • Público: pós-graduação / sênior

## 1) Abertura (5’)
- Problema: por que **big-bang deploy** é arriscado?
- Conceito: **canary** = exposição **progressiva** + **feedback** real.
- Diferenças para **Blue/Green** e **Feature Flags** (1 slide comparativo).

## 2) Arquitetura do laboratório (5’)

- Dois **Deployments** (v1, v2) e um **Service**.
- **Istio**: `DestinationRule` (subsets v1/v2) e `VirtualService` (pesos).
- Ferramentas: Rust (**serviço** + **CLI**), Docker Desktop Kubernetes, Prometheus (opcional).

## 3) Mão na massa (25’)

1. **Provisionar** (Docker Desktop + Istio) — `scripts/docker-desktop-istio-prom.sh` (mostrar comandos).
2. **Build** imagens v1/v2 — explicar `APP_VERSION` no container.
3. **Deploy v1** + Service + DR/VS (`k8s/base`).
4. **Criar v2** (YAML ou `canaryctl create-canary`).
5. **Divisão de tráfego**:
   - YAML prontos (90/10, 70/30, 50/50, 0/100) **ou**
   - `canaryctl set-traffic 90 10` etc.
6. **Testes**: `kubectl run curl ... curl http://versioned-echo/version` (mostrar alternância).
7. **Rollback** ao primeiro sinal de falha: `canaryctl rollback`.

> Dica: monitore rapidamente no Grafana (se instalado).

## 4) Fundamentos avançados (8’)

- **A/B vs sequencial** (intuição do SPRT / limites).
- **SLO‑driven** rollout: latência p95, erro < 1‰ etc.
- **Bandits** (Thompson Sampling): ideia e quando faz sentido.

## 5) Mercado e tendências (5’)

- Práticas: operadores (Argo Rollouts, Flagger), GitOps, ACA.
- Como encaixar no seu pipeline (CI/CD com gates).

## 6) Encerramento (2’)

- Checklist mental para um canário saudável.
- Repositório e próximos passos (ex.: incluir alertas automáticos).
