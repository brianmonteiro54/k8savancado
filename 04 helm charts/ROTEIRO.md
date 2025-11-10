# Roteiro de Apresentação — Aula 04 (Helm Charts)

**Duração sugerida:** 45–60 minutos

## 0. Abertura (2 min)

- Contexto do problema: muitos YAMLs duplicados, ambientes diferentes, risco de erro.
- Objetivo: mostrar como Helm resolve isso e como Rust entra como app de demonstração e como ferramenta confiável.

## 1. Conceitos (5–8 min)

- Helm = gerenciador de pacotes do Kubernetes (Chart = pacote + templates + valores).
- DRY, idempotência, imutabilidade, versionamento (chart vs appVersion).
- `values.schema.json` como “tipagem” de parâmetros.

## 2. Estrutura do Chart (5 min)

- Mostrar `Chart.yaml`, `values.yaml`, `templates/`.
- Explicar `helpers.tpl`, labels padrão e seletor consistente (evita drift).

## 3. Build e instalação (10–12 min)

1. `./scripts/build.sh` (gera imagem; Docker Desktop já enxerga a imagem local).
1. `helm install --dry-run --debug ...` (mostrar YAML renderizado).
1. `./scripts/helm-install.sh` (instala e port-forward).
1. Abrir `http://localhost:8080/` e **provar** que está rodando.

Pontos didáticos:

- Ver os `probes` em ação (`/health`).
- Ver envs (ConfigMap `GREETING`, `APP_VERSION`, `POD_NAME`).

## 4. Upgrade e rollback (10 min)

- `IMAGE_TAG=1.0.1 ./scripts/build.sh`
- `IMAGE_TAG=1.0.1 ./scripts/helm-upgrade.sh`
- Mostrar `helm history`.
- Induzir um erro (opcional) e demonstrar `./scripts/helm-rollback.sh`.

## 5. Overrides de ambiente e schema (5–8 min)

- Instalar com `-f values.dev.yaml` e comentar diferenças.
- Mostrar um erro proposital de tipo em valores e a falha de validação (se o Helm do aluno suporta schema).

## 6. Fechamento (3 min)

- Recapitular a tríade: Helm + GitOps + Policy.
- Dicas de próximos passos: Ingress/HTTPS, HPA/KEDA, CI/CD, assinatura de charts.
