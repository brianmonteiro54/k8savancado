# Aula 06 — Canary Deploys (Kubernetes + Rust)

> **Objetivo didático**: entregar um laboratório completo, reprodutível e observável para **Canary Releases** em Kubernetes. Você executará todo o ciclo: provisionar cluster, compilar serviços Rust, implantar as duas versões (v1/v2), **dividir tráfego de forma progressiva**, observar métricas e **executar rollback imediato** via CLI em Rust.
> **Importante**: este guia é **minucioso** e segue um **passo-a-passo numerado**, com explicações do “porquê” de cada ação e os scripts incluídos neste projeto.

---

## 0) Visão geral do que será implantado

Você terá:

* Um **serviço Rust** `versioned-echo` (Axum) com endpoints `/`, `/version`, `/health` e `/metrics`. A versão exibida (v1 ou v2) é controlada **por variável de ambiente** `APP_VERSION`, para que a demonstração foque em **roteamento e tráfego** e não na lógica da aplicação.
* Um **CLI Rust** `canaryctl`, que cria/remover o canário (Deployment v2 via API Kubernetes) e **ajusta pesos** do **Istio VirtualService** (90/10, 70/30, 50/50, 0/100, rollback 100/0).
* **YAMLs** de Kubernetes para a abordagem com **Istio** (DestinationRule/VirtualService), e **alternativas** com **NGINX Ingress** (canário por header) e **Argo Rollouts** (passos declarativos).
* **Scripts**:
  * `scripts/docker-desktop-istio-prom.sh`: prepara o **Docker Desktop Kubernetes** com Istio e kube-prometheus-stack (Prometheus/Grafana).
  * `scripts/demo.sh`: “caminho feliz” com build v1/v2, deploy base, canário e mudança de pesos.

---

## 1) Preparar pré-requisitos (Windows, Linux, macOS)

### 1.1 Windows (com **Chocolatey**)

Abra **PowerShell como Administrador**:

1. **Instale o Chocolatey** (se ainda não tiver):

   ```powershell
   Set-ExecutionPolicy Bypass -Scope Process -Force; `
   [System.Net.ServicePointManager]::SecurityProtocol = `
   [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; `
   iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
   ```

1. **Instale ferramentas**:

   ```powershell
   choco install git docker-desktop rust kubernetes-cli istioctl helm -y
   ```

   Esse comando instala Docker Desktop, Rust, kubectl, istioctl e Helm — componentes necessários para compilar os binários Rust, gerenciar o cluster Kubernetes e instalar Istio com o pacote kube-prometheus-stack.

1. **Reinicie** o PC (ou ao menos o Docker Desktop) e **habilite WSL2** no Docker Desktop (Settings → Resources → WSL Integration).

1. **Valide versões** (e confirme que o Kubernetes do Docker Desktop está habilitado em *Settings > Kubernetes*):

   ```powershell
   docker --version
   rustc --version
   kubectl version --client
   istioctl version
   helm version
   ```

> **Observação**: os scripts `.sh` podem ser executados no **Git Bash** (instalado com o Git) ou no **WSL2**. Se preferir ficar apenas no PowerShell, reproduza manualmente os comandos descritos nos scripts.

### 1.2 Linux (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install -y git docker.io rustc cargo kubectl
# istioctl
curl -L https://istio.io/downloadIstio | sh -
sudo mv istio-*/bin/istioctl /usr/local/bin/istioctl
# helm
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
# habilite docker
sudo usermod -aG docker $USER
newgrp docker
```

Ative o Kubernetes no Docker Desktop para Linux (quando disponível) ou utilize uma distribuição compatível (Minikube, k3d, MicroK8s). Caso o cluster não compartilhe o daemon Docker do host, publique as imagens em um registry acessível antes de aplicar os Deployments.

Valide com:

```bash
docker --version && rustc --version && kubectl version --client && istioctl version && helm version
```

### 1.3 macOS (Homebrew)

```bash
brew install git docker rust kubectl istioctl helm
```

Se usar Docker Desktop no macOS, inicie o aplicativo (habilitando Kubernetes nas preferências) e valide:

```bash
docker --version
rustc --version
kubectl version --client
istioctl version
helm version
```

---

## 2) Obter o projeto e entender a estrutura

2.1 **Baixe o ZIP** do projeto (ou clone seu repositório privado) e extraia.
A estrutura relevante:

```text
.
├── Cargo.toml                # Workspace Rust (members: canaryctl, versioned-echo)
├── versioned-echo/           # Serviço Rust (HTTP + Prometheus)
│   ├── Dockerfile
│   └── src/main.rs
├── canaryctl/                # CLI Rust para criar canário e ajustar tráfego
│   └── src/main.rs
├── k8s/
│   ├── base/                 # v1 + Service + DestinationRule + VirtualService (100/0)
│   ├── canary/               # v2 + VS com pesos 90/10, 70/30, 50/50, 0/100
│   ├── ingress-nginx/        # Alternativa: canário por header
│   └── argo-rollouts/        # Alternativa: passos declarativos
└── scripts/
  ├── docker-desktop-istio-prom.sh    # Habilita Istio + kube-prometheus-stack no Docker Desktop Kubernetes
    └── demo.sh               # Caminho feliz end-to-end
```

**Por que nessa ordem?**
Garantimos uma **base estável** (v1) antes de introduzir v2. Assim, o cluster já sabe rotear e o operador pode observar o comportamento **sem** risco.

---

## 3) Preparar o cluster (Docker Desktop + Istio + Prometheus)

1. Confirme que o contexto ativo é o `docker-desktop`:

  ```bash
  kubectl config current-context
  ```

  Se o resultado for diferente, ajuste com `kubectl config use-context docker-desktop`.

1. Execute o script de provisionamento deste módulo:

  ```bash
  bash scripts/docker-desktop-istio-prom.sh
  ```

  O script garante que você está no contexto correto, instala o **Istio** (perfil padrão) via `istioctl`, marca o namespace `default` com `istio-injection=enabled` e instala o **kube-prometheus-stack** via Helm no namespace `monitoring`.

1. Aguarde todos os pods ficarem prontos:

  ```bash
  kubectl get pods -n istio-system
  kubectl get pods -n monitoring
  ```

  Siga somente quando todos estiverem `Running`/`Completed`. O antigo script `kind-istio-prom.sh` foi mantido apenas como stub para evitar uso acidental.

---

## 4) Construir as imagens `versioned-echo` (v1 e v2)

Na raiz do módulo, construa as duas versões usando o mesmo Dockerfile (a versão é controlada por variável de ambiente `APP_VERSION`):

```bash
docker build -t versioned-echo:v1 ./versioned-echo
docker build --build-arg DUMMY=1 -t versioned-echo:v2 ./versioned-echo
```

Use `docker images | grep versioned-echo` para validar a presença das tags. Como usamos Docker Desktop, o cluster compartilha o daemon Docker do host, então não é necessário `kind load` ou push para registry.

---

## 5) Implantar a base (v1) com Istio configurado

1. Aplique os manifestos base, que incluem o Deployment v1, o Service, a `DestinationRule` e o `VirtualService` inicial (100% v1):

  ```bash
  kubectl apply -f k8s/base/deployment-v1.yaml
  kubectl apply -f k8s/base/destinationrule.yaml
  kubectl apply -f k8s/base/virtualservice.yaml
  ```

1. Aguarde enquanto o Deployment v1 fica disponível:

  ```bash
  kubectl rollout status deploy/versioned-echo-v1
  ```

1. Confirme objetos chave:

  ```bash
  kubectl get deploy,svc -l app=versioned-echo
  kubectl get destinationrule,virtualservice
  ```

  Este estado estável é seu ponto de partida para experimentar o canário.

---

## 6) Confirmar injeção do sidecar (Essencial para o Istio funcionar)

Verifique se os **pods v1** possuem **2 containers** (app + envoy):

```bash
kubectl get pod -l app=versioned-echo,version=v1 -o jsonpath='{range .items[*]}{.metadata.name}{" => containers: "}{.spec.containers[*].name}{"\n"}{end}'
```

Se **não** houver `istio-proxy`, confira se o namespace `default` tem `istio-injection=enabled` e **reinicie** o deployment:

```bash
kubectl label ns default istio-injection=enabled --overwrite
kubectl rollout restart deploy/versioned-echo-v1
```

---

## 7) Criar o canário (v2): por YAML **ou** pelo `canaryctl`

### 7.1 Usando YAML

```bash
kubectl apply -f k8s/canary/deployment-v2.yaml
kubectl rollout status deploy/versioned-echo-v2
```

### 7.2 Usando o CLI Rust `canaryctl`

Compile e rode:

```bash
cargo run -p canaryctl -- --ns default --app versioned-echo create-canary \
  --image versioned-echo:v2 --replicas 2
```

**O que o CLI faz**: usa a API Kubernetes para criar o Deployment v2 com as mesmas labels e probes, mas `APP_VERSION=v2`.
**Por que um CLI?** Para demonstrar **automação do control-plane**: ideal para integrar em **pipelines** (CI/CD, GitOps) e **padronizar** a operação.

---

## 8) Ajustar pesos do tráfego (Istio VirtualService)

Você pode fazer pelo **YAML** ou pelo **CLI**:

### 8.1 Via YAML (arquivos prontos, ideais para checkpoints didáticos)

```bash
kubectl apply -f k8s/canary/vs-90-10.yaml
kubectl apply -f k8s/canary/vs-70-30.yaml
kubectl apply -f k8s/canary/vs-50-50.yaml
kubectl apply -f k8s/canary/vs-0-100.yaml
```

Cada apply redefine a rota `v1/v2` com pesos que somam 100.

### 8.2 Via `canaryctl` (rápido para experimentos e rollback)

```bash
cargo run -p canaryctl -- set-traffic 90 10 --vs versioned-echo-virtualservice --host versioned-echo
cargo run -p canaryctl -- set-traffic 70 30 --vs versioned-echo-virtualservice --host versioned-echo
cargo run -p canaryctl -- set-traffic 50 50 --vs versioned-echo-virtualservice --host versioned-echo
```

**Rollback imediato**:

```bash
cargo run -p canaryctl -- rollback --vs versioned-echo-virtualservice --host versioned-echo
# equivale a 100% v1 / 0% v2
```

**Por que avançar gradualmente?**
Para **reduzir risco**. Começamos com 10% dos usuários no canário, observamos métricas/erros, e só então aumentamos a exposição. Se algo sair do SLO, **revertemos** instantaneamente.

---

## 9) Testar a alternância de versões (dentro do cluster)

Crie um pod “curl” efêmero e **observe o split** na prática:

```bash
kubectl run curl --image=curlimages/curl -it --rm --restart=Never -- \
  sh -lc 'while true; do echo -n "$(date) -> "; curl -s http://versioned-echo/version; sleep 1; done'
```

Você verá algo como:

```text
2025-10-16 12:00:00 -> {"version":"v1"}
2025-10-16 12:00:01 -> {"version":"v1"}
2025-10-16 12:00:02 -> {"version":"v2"}
...
```

Ao aplicar `vs-70-30.yaml`, a proporção muda. Em `vs-0-100.yaml`, tudo vira `v2`.

**Por que esse teste é valioso?**
Feedback instantâneo e visual do **efeito do VirtualService**. Não depende de ferramentas externas e deixa claro o que o canário faz.

---

## 10) Observabilidade com Prometheus/Grafana (opcional mas recomendado)

O serviço expõe `/metrics`. Com `kube-prometheus-stack`, você pode:

* Visualizar **taxa de requisições** (por rota/versão, se você adicionar rótulos nas métricas em evoluções futuras).
* Analisar **latência** (adicionando histogram/summary no serviço em versões estendidas do laboratório).
* Monitorar **erros** (simulados via contadores ou testes com falhas).

**Acesso local**:

```bash
kubectl port-forward svc/kps-grafana -n monitoring 3000:80
```

Crie um dashboard simples e acompanhe a evolução enquanto muda os pesos.
**Decisão canário** deve ser **orientada a dados**: se p95 de latência e taxa de erro estão dentro do SLO, avance; caso contrário, **rollback**.

---

## 11) Alternativas pedagógicas: NGINX Ingress e Argo Rollouts

### 11.1 NGINX Ingress (canário por **header**)

Arquivo: `k8s/ingress-nginx/ingress-canary-header.yaml`
Ele usa anotações `nginx.ingress.kubernetes.io/canary-by-header: X-Canary` e `...-value: v2`.
**Uso típico**: enviar só usuários internos (com header) para v2, mantendo outros em v1.

### 11.2 Argo Rollouts (canário **declarativo** por passos)

Arquivo: `k8s/argo-rollouts/rollout.yaml`
Define `setWeight` + `pause` (10% → 30% → 60% com pausas).
**Vantagem**: integra *analysis*, *gates* e UI; facilita padronizar políticas de rollout na organização.

> **Observação**: estes manifestos são extras didáticos; o caminho principal desta aula usa **Istio**.

---

## 12) Executar o caminho feliz completo com `scripts/demo.sh`

Se você prefere ver **tudo encadeado**, use:

```bash
bash scripts/demo.sh
```

O script:

1. **Builda** as imagens v1 e v2 do `versioned-echo`.
2. **Aplica** a base (`k8s/base`: v1 + Service + DR/VS).
3. **Cria** v2 (`k8s/canary/deployment-v2.yaml`).
4. **Altera pesos** progressivamente: 90/10 → 70/30 → 50/50 → 0/100, com pequenos `sleep` para observar.
5. **Sugere** um comando de “curl” contínuo dentro do cluster para visualizar o tráfego.

**Por que este script importa?**
Mostra o **fluxo operacional mínimo** de um rollout canário e serve de **modelo** para automações de CI/CD.

---

## 13) Entendendo os YAMLs: do zero ao roteamento ponderado

* **Deployment v1/v2**: definem **labels** `app: versioned-echo` e `version: v1|v2`.
  As **probes** em `/health` garantem que só pods **prontos** entrem no *load*.
* **Service `versioned-echo`**: seleciona `app=versioned-echo` (v1 **e** v2).
  Quem decide para **qual versão** vai o tráfego **não** é o Service, e sim o **VirtualService**.
* **DestinationRule**: mapeia **subsets** (`v1`, `v2`) para **labels** de pods.
  Sem isso, o VirtualService **não consegue** rotear por versão.
* **VirtualService**: define **rotas HTTP** com **weights** para `subset: v1` e `subset: v2`.
  A soma deve ser **100**. Ajustar pesos = **mudar a fração de tráfego**.

Valide seus recursos:

```bash
kubectl get deploy,svc -l app=versioned-echo
kubectl get destinationrule,virtualservice
kubectl describe virtualservice versioned-echo-virtualservice
```

---

## 14) Fluxos de rollback e recuperação

* **Rollback lógico instantâneo** (reverter tráfego para v1):

  ```bash
  cargo run -p canaryctl -- rollback --vs versioned-echo-virtualservice --host versioned-echo
  ```

* **Rollback físico** (remoção do v2):

  ```bash
  kubectl delete deploy/versioned-echo-v2
  # ou
  cargo run -p canaryctl -- delete-canary
  ```

* **Recuperação de pods v1**:

  ```bash
  kubectl rollout restart deploy/versioned-echo-v1
  ```

**Crucial**: **MTTR baixo** é parte da estratégia. Você quer **facilidade** de reverter e **confiança** para avançar novamente.

---

## 15) Troubleshooting (erros frequentes, como pensar e corrigir)

* **VirtualService sem efeito (sempre 100% v1)**
  Verifique se o **sidecar** foi injetado:

  ```bash
  kubectl get pod -l app=versioned-echo -o jsonpath='{range .items[*]}{.metadata.name}{" => "}{.spec.containers[*].name}{"\n"}{end}'
  ```

  Se faltar `istio-proxy`, recoloque a label no namespace e reinicie os deployments.
* **Subsets não batem**
  Confirme que os pods v1 têm `version=v1` e os v2 `version=v2`:

  ```bash
  kubectl get pod -l app=versioned-echo -o=jsonpath='{range .items[*]}{.metadata.name}{" => "}{.metadata.labels.version}{"\n"}{end}'
  ```

  E que o `DestinationRule` referencia exatamente essas labels.
* **Time-outs/5xx após liberar v2**
  Faça rollback lógico para 100/0, **inspecione logs** do v2 e os **readiness/liveness**.
  Se necessário, **remova v2** e verifique **recursos** (CPU/mem), **quota**, **políticas de rede**.
* **Imagens não encontradas no cluster**
  Em ambientes que **não** compartilham o daemon Docker com o host (ex.: clusters remotos ou Minikube sem `--driver=docker`), publique a imagem em um registry acessível ou use o comando equivalente (`minikube image load`, `nerdctl push`, etc.).

---

## 16) Limpeza do ambiente

Remova os recursos:

```bash
kubectl delete -f k8s/canary --ignore-not-found
kubectl delete -f k8s/base --ignore-not-found
```

**Por que limpar?**
Para restabelecer o estado, liberar recursos e garantir **reprodutibilidade** em execuções futuras.

---

## 17) Por que cada etapa é importante (ligação com a teoria da aula)

* **Dois Deployments em paralelo (v1/v2)** tornam possível **exposição progressiva** sem interrupção do serviço.
* **Subsets e VirtualService do Istio** separam **tráfego lógico** (v1/v2) do **número de réplicas físicas**; isso dá controle fino da porcentagem real de usuários expostos ao canário.
* **Pesos graduais** (90/10 → 70/30 → 50/50 → 0/100) implementam uma **estratégia de risco mínimo**, permitindo **medir** antes de **promover**.
* **Observabilidade** garante decisões **baseadas em evidência** (SLO, taxa de erro, latência).
* **Automação via `canaryctl`** mostra que o controle pode ser **padronizado e versionado**, integrando-se com pipelines e reduzindo erro humano.

---

## 18) Apêndice — Comandos úteis de verificação

* Ver pods por versão:

  ```bash
  kubectl get pods -l app=versioned-echo -o=jsonpath='{range .items[*]}{.metadata.name}{" => "}{.metadata.labels.version}{"\n"}{end}'
  ```

* Ver rotas do VirtualService:

  ```bash
  kubectl get virtualservice versioned-echo-virtualservice -o yaml
  ```

* Descrever DR/VS (erros comuns aparecem aqui):

  ```bash
  kubectl describe virtualservice versioned-echo-virtualservice
  kubectl describe destinationrule versioned-echo
  ```

* Teste simples fora do pod curl (com port-forward):

  ```bash
  kubectl port-forward svc/versioned-echo 8080:80
  curl -s http://localhost:8080/version
  ```

---

## 19) Apêndice — Considerações de segurança e custo

* Em produção, use **namespaces dedicados**, **RBAC** restritivo e **políticas de imagem** (admissão).
* Gere **dashboards** e **alertas** específicos de rollout canário (ex.: alertar se erro > X% por Y minutos durante aumento de peso).
* Em nuvem, monitore **custos** e **cotizações**; *canaries* em horários de pico podem custar mais (telemetria, pods extras).

---

## 20) Apêndice — Glossário rápido

* **Canary Release**: liberar uma nova versão para uma **pequena parcela** de usuários, **medir impacto**, e só então aumentar a exposição.
* **Subset (Istio)**: agrupamento lógico de pods com as mesmas **labels** (neste guia, `version=v1` e `version=v2`).
* **VirtualService (Istio)**: regra de tráfego L7 que **decide** a rota e **aplica pesos** entre subsets.
* **Rollback**: voltar tráfego para versão estável (100/0) de forma **rápida e segura**.

---

### Encerramento

Seguindo estas **etapas numeradas** e utilizando os **scripts fornecidos**, você terá um ambiente **completo** para demonstrar canário em Kubernetes com Istio, incluindo **observabilidade** e **automação**. A prática aqui proposta **espelha o processo real** em produção, onde a decisão de promover uma versão é **estatística** e **operacional**, suportada por **dados** e um **plano de reversão** simples e confiável.
