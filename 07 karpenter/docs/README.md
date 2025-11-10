# Aula 07 — Escalabilidade de Nodes com Karpenter (Demo em Rust + YAML)

Este repositório entrega **todo o material prático** da Aula 07, com **código em Rust**, **manifests YAML completos**, e um **roteiro de apresentação** passo a passo.
O objetivo é **entender de forma prática e visual** como funciona o provisionamento sob demanda de nós — o mesmo princípio que fundamenta o **Karpenter**, o autoscaler de última geração do ecossistema Kubernetes — porém de forma **agnóstica**, ou seja, **sem depender de provedores de nuvem**.

A demonstração é construída com um **mini-controlador em Rust**, usando a biblioteca `kube-rs`, que executa duas funções principais:

1. **Detecção automática de Pods “Pending”** (não agendáveis) e **geração de um plano de provisionamento** em um `ConfigMap`.
   Esse plano é um arquivo YAML que poderia ser aplicado como um **Provisioner/NodePool real**, caso o cluster tivesse o Karpenter instalado.

2. **Simulação de consolidação de nós**: periodicamente, o controlador calcula a **utilização de CPU e memória solicitadas** (*requests*) em cada nó e gera sugestões para **encerrar nós ociosos**, apresentando a lista de Pods que poderiam ser realocados.

> 💡 **Importante:** este projeto é totalmente **didático**. Ele **não cria nem remove VMs reais**.
> Se for utilizado em um cluster com Karpenter, basta aplicar o YAML gerado para observar o comportamento real de provisionamento.

---

## 1) Preparação do ambiente local (instalação detalhada)

Para acompanhar a aula e executar a demonstração, você precisa de um ambiente que suporte **Rust**, **Docker**, **kubectl**, e um **cluster Kubernetes** simples. Abaixo estão instruções completas para **Windows**, **macOS** e **Linux**, com foco em usuários de Windows, onde o **Chocolatey** é a opção mais prática para gerenciar dependências.

### 🪟 Windows (com Chocolatey)

1. **Instale o Chocolatey** (caso ainda não tenha):
   Abra o *PowerShell* como Administrador e execute:

   ```powershell
   Set-ExecutionPolicy Bypass -Scope Process -Force; `
   [System.Net.ServicePointManager]::SecurityProtocol = `
   [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; `
   iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
   ```

2. **Instale os pacotes necessários:**

   ```powershell
   choco install -y rust kubernetes-cli docker-desktop make git
   ```

   * `rust` instala o compilador e o gerenciador de toolchains `rustup`.
   * `kubernetes-cli` instala o `kubectl`.
   * `docker-desktop` fornece o ambiente Docker Engine, o cluster Kubernetes embutido e integra com o WSL2.
   * `make` permite usar o Makefile incluído no projeto.
   * `git` facilita o clone e versionamento.

3. **Configure o Docker Desktop**:

    * Abra o Docker Desktop → *Settings > Kubernetes* → marque **Enable Kubernetes**.
    * Aguarde até o cluster iniciar; verifique com:

       ```powershell
       kubectl get nodes
       kubectl config current-context
       ```

4. **Configure o Rust toolchain:**

   ```powershell
   rustup update
   rustup default stable
   ```

5. **Verifique o ambiente:**

   ```powershell
   cargo --version
   kubectl version --client
   docker --version
   ```

### 🍎 macOS (Homebrew)

```bash
brew install rustup-init kubectl make git
brew install --cask docker
rustup-init -y
rustup default stable
```

### 🐧 Linux (Debian/Ubuntu)

```bash
sudo apt update && sudo apt install -y curl git make docker.io
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
sudo snap install kubectl --classic
```

Com todos os componentes instalados, **priorize o Kubernetes embutido no Docker Desktop**, mantendo o contexto `docker-desktop` ativo. Se optar por alternativas (Minikube, k3d, MicroK8s), lembre-se de que essas distribuições podem usar um daemon Docker diferente: publique as imagens em um registry acessível antes de aplicar os Deployments.

> ✅ **Verificação final:**
> Execute `kubectl get nodes` e confirme que o cluster está ativo e **Ready**. Depois, valide com `kubectl config current-context` que você está no `docker-desktop`.

---

## 2) Estrutura do projeto

```text
aula07_karpenter_rust_demo/
├─ rust-controller/
│  ├─ Cargo.toml                # dependências Rust
│  ├─ Dockerfile                # build da imagem do controlador
│  └─ src/main.rs               # código-fonte principal
├─ k8s/
│  ├─ 00-namespace.yaml
│  ├─ 01-serviceaccount.yaml
│  ├─ 02-clusterrole.yaml
│  ├─ 03-clusterrolebinding.yaml
│  ├─ 04-deployment.yaml
│  ├─ samples/
│  │  ├─ pending-deploy.yaml
│  │  └─ consolidation-workload.yaml
│  └─ optional/karpenter/provisioner-template.yaml
├─ docs/
│  ├─ README.md  (este guia completo)
│  └─ ROTEIRO.md (guia da apresentação)
└─ Makefile
```

O projeto está dividido em três camadas:

* **Código-fonte (Rust):** implementa a lógica de detecção e análise.
* **Manifestos YAML:** definem todos os objetos Kubernetes necessários (namespace, RBAC, e exemplos).
* **Documentação:** explica cada conceito de forma didática e inclui o roteiro para a aula.

---

## 3) Passo a passo detalhado (YAML + execução)

### 3.1 Criar o namespace e as permissões

O Kubernetes é seguro por padrão; cada componente precisa de permissões explícitas.
Primeiro criamos o namespace `aula07` e, dentro dele, a conta de serviço, a função de acesso (RBAC) e o vínculo da função.

```bash
kubectl apply -f k8s/00-namespace.yaml
kubectl apply -f k8s/01-serviceaccount.yaml
kubectl apply -f k8s/02-clusterrole.yaml
kubectl apply -f k8s/03-clusterrolebinding.yaml
```

Esses manifests garantem que o controlador poderá **listar Pods, ler nós e criar ConfigMaps** — nada além disso, preservando o princípio do menor privilégio.

---

### 3.2 Rodar o controlador localmente (recomendado para começar)

Com o cluster pronto, podemos executar o controlador direto da sua máquina.
Entre no diretório do código e compile:

```bash
cd rust-controller
cargo run --release
```

O comando `cargo run` compila o projeto e executa o binário, conectando-se ao Kubernetes via `~/.kube/config`.

A partir desse momento o controlador:

* examina periodicamente os Pods Pending;
* gera um `ConfigMap` com o plano de provisionamento (`plan-<pod>`);
* analisa todos os nós existentes e cria sugestões de consolidação (`consolidation-<node>`).

Monitore em outro terminal:

```bash
kubectl get pods -A | grep Pending
kubectl get configmap -n aula07
```

---

### 3.3 Executar dentro do cluster (opcional)

Para executar “como um serviço nativo” dentro do cluster, é necessário empacotar a aplicação em Docker.

```bash
docker build -t aula07/mini-karpenter-rs:latest ./rust-controller
```

Com o Kubernetes do Docker Desktop, **não é necessário** executar nenhum `load`: o cluster usa o mesmo daemon Docker do host. Se estiver trabalhando com um cluster externo que não compartilhe o daemon, publique a imagem em um registry acessível (por exemplo `docker push <SEU_REGISTRO>/aula07/mini-karpenter-rs:latest`) e ajuste a referência em `k8s/04-deployment.yaml` ou via `make IMAGE=<...> deploy`.

Em seguida, aplique o Deployment:

```bash
kubectl apply -f k8s/04-deployment.yaml
kubectl -n aula07 logs deploy/mini-karpenter-rs -f
```

O controlador rodará como um Pod dentro do cluster, reproduzindo o mesmo comportamento da execução local.

---

### 3.4 Gerar Pods Pending de propósito

Para observar a detecção automática, aplicamos um Deployment que deliberadamente **não pode ser agendado** (usa um `nodeSelector` inexistente).

```bash
kubectl apply -f k8s/samples/pending-deploy.yaml
kubectl -n aula07 get pods -w
```

Após alguns segundos, o controlador criará um `ConfigMap plan-<pod>`:

```bash
kubectl -n aula07 get configmap | grep plan-
kubectl -n aula07 get configmap plan-<POD> -o yaml
```

Dentro do campo `data.plan.yaml`, você verá um YAML de Provisioner ou NodePool, similar ao que o Karpenter aplicaria para resolver a falta de capacidade.

---

### 3.5 Simular consolidação de nós

Para visualizar o outro lado do processo — a otimização de recursos — crie workloads pequenos:

```bash
kubectl apply -f k8s/samples/consolidation-workload.yaml
```

Após alguns ciclos (~30 segundos), consulte os ConfigMaps de consolidação:

```bash
kubectl -n aula07 get configmap | grep consolidation-
kubectl -n aula07 get configmap consolidation-<NODE> -o yaml
```

Essas sugestões mostram quais Pods estão em nós com **utilização < 20%**, demonstrando o princípio de **scale-down inteligente**.

---

### 3.6 Integrar com Karpenter real (opcional)

Se o cluster tiver o Karpenter instalado:

1. Extraia o `plan.yaml` do ConfigMap `plan-*`;
2. Ajuste os campos `requirements` conforme a sua região e tipos de nó;
3. Aplique diretamente:

   ```bash
   kubectl apply -f plan.yaml
   ```

4. Observe a criação de novos nós e o Pod sair do estado Pending.

> O fluxo é **100% declarativo e auditável**: você gera, valida, versiona e aplica as decisões — conceito central em GitOps e FinOps.

---

## 4) Entendendo o código

O arquivo `main.rs` implementa um loop assíncrono com `tokio` que:

1. Conecta-se ao cluster (`Client::try_default()` usa o mesmo kubeconfig do kubectl).
2. Busca Pods Pending a cada 10 segundos.
3. Extrai suas **requests** de CPU e memória.
4. Seleciona, a partir de um catálogo estático de tamanhos de nó, o menor que atende à demanda (heurística *least waste*).
5. Gera um `ConfigMap` com duas chaves:

   * `plan.yaml` → definição do Provisioner;
   * `info.json` → metadados com os cálculos.
6. Depois, percorre todos os nós e soma as requests dos Pods neles alocados.
7. Se o uso máximo de CPU ou memória for < 20 %, gera um `ConfigMap consolidation-<node>` sugerindo drain e remoção.

O uso de ConfigMaps como saída permite inspecionar facilmente os planos sem aplicar mudanças reais.

---

## 5) Conexão com a teoria da aula

Cada etapa prática reflete um conceito estudado:

* **Pods Pending → Plano:** representa o detector de “unschedulables” do Karpenter.
* **Catálogo de instâncias + least-waste:** simula o *bin packing multidimensional*, buscando o melhor encaixe entre CPU e memória.
* **Consolidação:** demonstra o uso de políticas de redução de ociosidade para diminuir custos.
* **Fluxo declarativo:** reforça a governança via YAML versionado (GitOps + FinOps).

---

## 6) Limitações e possíveis extensões

* **Sem ação real**: o controlador não cria nem remove nós físicos.
* **Utilização por requests:** poderia ser substituída por métricas reais via Metrics Server.
* **Catálogo estático:** em produção, deriva-se de catálogos reais (AWS EC2, Azure VMs, etc.) com custo por hora.
* **Regras avançadas:** poderiam considerar Pod Disruption Budgets, Spread Constraints, afinidades e prioridades.

Essas extensões são ótimos pontos de continuação para quem desejar evoluir o projeto para um protótipo de autoscaler real.

---

## 7) Limpeza do ambiente

Ao final dos testes, execute:

```bash
kubectl delete -f k8s/samples/consolidation-workload.yaml --ignore-not-found
kubectl delete -f k8s/samples/pending-deploy.yaml --ignore-not-found
kubectl delete -f k8s/04-deployment.yaml --ignore-not-found
kubectl delete -f k8s/03-clusterrolebinding.yaml --ignore-not-found
kubectl delete -f k8s/02-clusterrole.yaml --ignore-not-found
kubectl delete -f k8s/01-serviceaccount.yaml --ignore-not-found
kubectl delete -f k8s/00-namespace.yaml --ignore-not-found
```

Isso remove tudo que foi criado no namespace `aula07`.

---

## 8) Troubleshooting (comum em aula)

* **Nenhum Pod fica Pending:** confira se o `pending-deploy.yaml` está no namespace certo e possui um `nodeSelector` impossível.
* **Sem ConfigMaps:** verifique logs (`cargo run` ou `kubectl logs`) e as permissões RBAC.
* **Imagem não encontrada:** ajuste o campo `image:` no Deployment ou publique a imagem em um registry acessível ao seu cluster.
* **Cluster inativo:** garanta que o Docker Desktop esteja executando (ou que sua alternativa, como Minikube, esteja ativa e com o contexto selecionado).

---

Feito com ❤️ para a *Aula 07 — FIAP / Kubernetes Avançado*
Este laboratório demonstra como a observação de Pods Pending e a otimização de nós se integram em um processo único de **autoscaling inteligente**, moderno e governável.
