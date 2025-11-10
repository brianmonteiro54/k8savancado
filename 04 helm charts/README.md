# Aula 04 — Helm Charts com Rust (Kubernetes Avançado)

Este projeto entrega **código em Rust**, **Helm Chart completo**, **YAMLs templatizados**, e um **roteiro de apresentação** com passo‑a‑passo prático e agnóstico para que você rode a demonstração em qualquer ambiente Kubernetes (local com Docker Desktop/minikube ou em cloud).

> Objetivos práticos:
> 1) Construir e publicar uma imagem de um microserviço Rust.
> 2) Empacotar a aplicação em um Helm Chart parametrizável (DRY).
> 3) Instalar, atualizar e fazer rollback usando Helm **de forma idempotente**.
> 4) Validar valores via `values.schema.json`.
> 5) Explicar cada decisão técnica (por que é importante para a aula).


## 1. Pré‑requisitos


### (Opcional) Validar o contexto do Docker Desktop

```bash
./scripts/docker-desktop-setup.sh
```

O script confirma se o contexto `docker-desktop` está disponível e selecionado. Ative **Settings ▸ Kubernetes ▸ Enable Kubernetes** no Docker Desktop caso ainda não tenha feito isso.

## 2. Estrutura do projeto

```text
aula04-helm-rust/
├── myrustapp/                    # App Rust + Dockerfile
├── helm/
│   └── myrustapp-chart/          # Chart Helm completo
│       ├── Chart.yaml
│       ├── values.yaml
│       ├── values.dev.yaml
│       ├── values.prod.yaml
│       ├── values.schema.json    # Validação dos inputs
│       └── templates/
│           ├── _helpers.tpl
│           ├── configmap.yaml
│           ├── deployment.yaml
│           ├── service.yaml
│           └── NOTES.txt
└── scripts/
  ├── build.sh
  ├── docker-desktop-setup.sh
  ├── kind-setup.sh (shim legado → chama docker-desktop-setup.sh)
  ├── helm-install.sh
  ├── helm-upgrade.sh
  └── helm-rollback.sh
```

**Por que essa estrutura é importante?**  
Separa claramente **aplicação**, **infra com Helm** e **automação**. Isso ajuda na governança (versionar cada parte), no reuso de charts (DRY) e na portabilidade.

## 3. Build da aplicação e imagem

> A aplicação expõe `GET /` (JSON com `greeting`, `version`, `pod`) e `GET /health` (probe).

```bash
# na raiz do projeto
./scripts/build.sh
```


## 4. Instalação com Helm (idempotência na prática)

```bash
# Dry-run (render sem aplicar)
helm install --dry-run --debug myrustapp-release ./helm/myrustapp-chart   --namespace dev   --set image.repository=myrustapp   --set image.tag=1.0.0

# Instalar de fato
./scripts/helm-install.sh
# (faz o install e abre um port-forward)
```

Abra <http://localhost:8080/> e você verá um JSON parecido com:

```json
{
  "app": "myrustapp",
  "version": "1.0.0",
  "greeting": "Olá, Helm + Rust!",
  "pod": "myrustapp-release-myrustapp-xxxx",
  "note": "Deployado via Helm Chart"
}
```

**Idempotência:** se você repetir o `helm install` com os mesmos valores, nada novo precisa ser aplicado: o estado desejado já foi alcançado.

## 5. Atualização (upgrade) e rollback

### Upgrade do app (trocar a tag da imagem)

1. Rebuild com nova versão e carregue a imagem:

  ```bash
  IMAGE_TAG=1.0.1 ./scripts/build.sh
  ```

1. Faça o upgrade:

  ```bash
  IMAGE_TAG=1.0.1 ./scripts/helm-upgrade.sh
  ```

1. Se algo der errado, faça rollback para a revisão anterior:

  ```bash
  ./scripts/helm-rollback.sh  # (REVISION=1 por padrão)
  ```

**Imutabilidade:** um novo `ReplicaSet` é criado para a nova versão. O histórico permite **rollback** limpo.

## 6. Parametrização & DRY (values.yaml)

- `replicaCount`, `service.port`, `service.containerPort`, `resources`, `config.greeting` etc.
- Overrides por ambiente:
  - `values.dev.yaml` (réplica única, custo reduzido)
  - `values.prod.yaml` (mais réplicas e recursos)

Exemplo de instalação usando overrides:

```bash
helm install myrustapp-release ./helm/myrustapp-chart -n dev -f helm/myrustapp-chart/values.dev.yaml
```

## 7. Validação de inputs (`values.schema.json`)

O chart inclui um `values.schema.json` (JSON Schema) para validar os valores antes do render.  
**Benefício:** falha cedo se um campo obrigatório estiver ausente ou com tipo errado.

## 8. RBAC e ServiceAccount (opcional)

Por padrão **não** cria RBAC/SA. Para ambientes bloqueados, habilite no `values.yaml` (ou via `--set`) e ajuste as permissões mínimas necessárias.

## 9. Acesso ao serviço

Sem Ingress por simplicidade. Opcões:

- `kubectl port-forward svc/myrustapp-release-myrustapp 8080:80`
- Alterar `service.type=LoadBalancer` em um cluster de cloud.

## 10. Por que cada decisão é didática?

- **Rust**: binário rápido, footprint pequeno e confiável; ótimo para automações de DevOps ou microserviços.
- **Helm**: encapsula múltiplos manifestos com **versionamento e DRY**; demonstra **idempotência** e **rollbacks**.
- **Probes**: mostram práticas de produção (readiness/liveness).
- **ConfigMap**: ilustra **configuração externa** ao container.
- **Schema de valores**: traz “sistema de tipos” para configs, evitando erros de digitação.
- **Scripts**: padronizam a demo (menos atrito ao apresentar).

## 11. Troubleshooting rápido

- **Pods não sobem**: `kubectl describe pod -n dev` e `kubectl logs -n dev <pod>`
- **Imagem não encontrada**: garanta que a imagem está disponível
  - Docker Desktop: reconstrua com `./scripts/build.sh` (não precisa push)
  - Minikube: `minikube image load myrustapp:1.0.0`
- **Port-forward ocupado**: feche sessões anteriores ou mude a porta local (`8081:80`).


## 12. Limpeza

```bash
helm uninstall myrustapp-release -n dev
kubectl delete ns dev --ignore-not-found
```
