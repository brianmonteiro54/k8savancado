# Kubernetes Avançado - FIAP Pós Tech

Este repositório contém todo o **material prático em código** desenvolvido para o curso de **Kubernetes Avançado** da **FIAP Pós Tech**. Cada módulo apresenta implementações reais, em sua maioria escritas em **Rust**, cobrindo os principais conceitos avançados de orquestração de containers e práticas de produção em Kubernetes.

## 📚 Sobre o Curso

O curso de Kubernetes Avançado da FIAP Pós Tech aborda conceitos fundamentais e avançados para profissionais que desejam dominar a orquestração de containers em ambientes de produção. Este repositório serve como laboratório prático, onde cada aula possui seu próprio diretório com código-fonte, manifestos Kubernetes (YAML) e documentação detalhada.

---

## 🎯 Estrutura dos Módulos

### [01 - App Health](./01%20app%20health/)
**Saúde da Aplicação e Gerenciamento de Recursos**

Demonstração completa de monitoramento de saúde de aplicações em Kubernetes, incluindo:
- **Golden Signals** (latência, erros, tráfego, saturação) via endpoint Prometheus `/metrics`
- **Health Probes**: Liveness, Readiness e Startup probes
- **Modos de Falha**: CPU throttling, memory leak/OOM, injeção de latência e erros
- **QoS Classes**: Burstable vs Guaranteed via Kustomize overlays
- **HPA** (Horizontal Pod Autoscaler) baseado em métricas de CPU
- **Graceful Shutdown** e **PDB** (Pod Disruption Budget)

**Tecnologias**: Rust + Axum, Kustomize, Prometheus

**Pré-requisitos**:
- Docker
- kubectl (v1.24+)
- Cluster Kubernetes (Kind, Minikube, Docker Desktop ou cloud)
- Opcional: `hey` ou `vegeta` para testes de carga

---

### [02 - Advanced Scheduler](./02%20advanced%20scheduler/)
**Agendamento Avançado e Gerenciamento de Nós**

Laboratório prático sobre os mecanismos avançados de agendamento do Kubernetes:
- **Taints & Tolerations** para isolamento de workloads
- **Node Affinity & Anti-Affinity** para controle de posicionamento
- **Pod Affinity & Anti-Affinity** para co-localização ou distribuição
- **Topology Spread Constraints** para distribuição equilibrada
- **Priority Classes & Preemption** para cargas críticas
- **QoS Classes** (BestEffort, Burstable, Guaranteed) e políticas de eviction

**Tecnologias**: Rust, Kind, múltiplos nós workers

**Pré-requisitos**:
- Docker
- kubectl
- Kind (para criar cluster multi-node)
- Aplicação Rust inclusa para demonstrações

---

### [03 - Rollout Updates](./03%20rollout%20updates/)
**Estratégias de Atualização e Rollout**

Implementação prática de diferentes estratégias de deploy e atualização:
- **RollingUpdate**: atualização gradual com controle de `maxSurge` e `maxUnavailable`
- **Recreate**: substituição total (útil para incompatibilidades de versão)
- **Simulador de Algoritmo**: ferramenta para visualizar o comportamento do RollingUpdate
- **Argo Rollouts** (opcional): Progressive Delivery com canary declarativo

**Tecnologias**: Rust + Axum, Docker, Argo Rollouts

**Pré-requisitos**:
- Docker Desktop (com Kubernetes habilitado) ou Minikube
- kubectl
- Rust toolchain
- Opcional: Argo Rollouts CLI

---

### [04 - Helm Charts](./04%20helm%20charts/)
**Gerenciamento com Helm e Empacotamento de Aplicações**

Demonstração completa de Helm como ferramenta de empacotamento e gestão:
- **Chart completo** com templates parametrizáveis
- **values.yaml** com overrides para dev/prod
- **values.schema.json** para validação de inputs
- **Helpers e NOTES.txt** para experiência profissional
- **Install, Upgrade e Rollback** idempotentes
- **ConfigMap** para separação de configuração

**Tecnologias**: Rust, Helm 3, Docker

**Pré-requisitos**:
- Docker Desktop com Kubernetes
- Helm 3
- kubectl
- Rust toolchain

---

### [05 - Blue/Green Deployment](./05%20blue%20green/)
**Deploy Blue/Green para Zero Downtime**

Implementação completa de estratégia Blue/Green com automação:
- **Dois Deployments paralelos** (blue e green)
- **Service como switch** de tráfego via selector
- **Rollback instantâneo** sem recriar pods
- **Orquestrador em Rust** (CLI com kube-rs) para automação
- **Scripts de demonstração** completos

**Tecnologias**: Rust + Axum, kube-rs, Docker

**Pré-requisitos**:
- Docker Desktop com Kubernetes ou Kind
- kubectl
- Rust toolchain (para o orquestrador)

---

### [06 - Canary Deployment](./06%20canary/)
**Canary Releases com Controle de Tráfego**

Laboratório completo de Progressive Delivery com múltiplas abordagens:
- **Istio Service Mesh**: DestinationRule + VirtualService para controle de pesos
- **CLI em Rust** (`canaryctl`) para criação e ajuste de canários
- **Controle gradual de tráfego**: 90/10 → 70/30 → 50/50 → 0/100
- **Observabilidade**: Prometheus + Grafana (kube-prometheus-stack)
- **Alternativas**: NGINX Ingress (canary por header) e Argo Rollouts

**Tecnologias**: Rust, Istio, Prometheus, Grafana

**Pré-requisitos**:
- Docker Desktop com Kubernetes
- kubectl
- istioctl
- Helm 3
- Rust toolchain

---

### [07 - Karpenter](./07%20karpenter/)
**Autoscaling de Nós com Karpenter**

Demonstração de autoscaling inteligente de nós com Karpenter.

**Tecnologias**: Rust, Karpenter

**Pré-requisitos**:
- Verificar documentação específica em `docs/README.md`

---

### [08 - KEDA](./08%20keda/)
**Autoscaling Event-Driven com KEDA**

Sistema completo de autoscaling baseado em eventos externos:
- **KEDA** (Kubernetes Event-driven Autoscaler)
- **Scale-to-Zero**: reduz pods para 0 quando não há carga
- **RabbitMQ** como fonte de eventos (fila de mensagens)
- **Worker em Rust** que escala automaticamente (0 a N pods)
- **Publisher em Rust** para simular carga
- **ScaledObject** com configuração de polling, cooldown e histerese

**Tecnologias**: Rust, KEDA, RabbitMQ, Helm

**Pré-requisitos**:
- Docker Desktop com Kubernetes
- kubectl
- Helm 3
- Rust toolchain

---

### [09 - Security](./09%20security/)
**Segurança em Kubernetes - Hands-on Completo**

Implementação prática dos principais pilares de segurança em K8s:
- **ServiceAccounts dedicadas** (identidade de workload)
- **RBAC mínimo** (Role + RoleBinding com menor privilégio)
- **TLS automatizado** com cert-manager (CA interna)
- **Servidor HTTPS em Rust** usando certificados emitidos automaticamente
- **Identidade federada** (anotações para EKS IRSA, GKE Workload Identity, AKS Managed Identity)
- **Orquestrador em Rust** para automação de RBAC e identidades

**Tecnologias**: Rust + Actix-web, cert-manager, kube-rs

**Pré-requisitos**:
- Docker Desktop com Kubernetes
- kubectl
- Helm 3
- Rust toolchain

---

## 🛠️ Instalação das Ferramentas Necessárias

### Windows (via Chocolatey)

Abra o **PowerShell como Administrador** e execute:

```powershell
# Instalar Chocolatey (se ainda não tiver)
Set-ExecutionPolicy Bypass -Scope Process -Force; `
[System.Net.ServicePointManager]::SecurityProtocol = `
[System.Net.ServicePointManager]::SecurityProtocol -bor 3072; `
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# Instalar ferramentas principais
choco install -y docker-desktop kubernetes-cli helm rustup.install git make

# Configurar Rust
rustup default stable
```

**Importante**: Habilite o Kubernetes no Docker Desktop em **Settings → Kubernetes → Enable Kubernetes**.

### Linux (Ubuntu/Debian)

```bash
# Dependências base
sudo apt update && sudo apt install -y curl git make docker.io docker-compose-plugin

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# kubectl
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl

# Helm
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# Kind (para clusters locais)
curl -Lo ./kind https://kind.sigs.k8s.io/dl/latest/kind-linux-amd64
chmod +x ./kind
sudo mv ./kind /usr/local/bin/kind

# istioctl (para módulo 06)
curl -L https://istio.io/downloadIstio | sh -
sudo mv istio-*/bin/istioctl /usr/local/bin/

# Adicionar usuário ao grupo docker
sudo usermod -aG docker $USER
newgrp docker
```

### macOS (via Homebrew)

```bash
# Instalar ferramentas
brew install kubectl helm make git kind
brew install --cask docker

# Rust
brew install rustup-init
rustup-init -y
rustup default stable

# istioctl (para módulo 06)
brew install istioctl
```

**Importante**: Inicie o Docker Desktop e habilite o Kubernetes em **Preferences → Kubernetes**.

---

## 🚀 Como Usar Este Repositório

Cada módulo é **independente** e pode ser executado isoladamente. A estrutura padrão de cada módulo é:

```
XX módulo/
├── README.md              # Documentação detalhada do módulo
├── ROTEIRO.md             # Roteiro de apresentação (quando aplicável)
├── src/ ou crates/        # Código-fonte Rust
├── k8s/                   # Manifestos Kubernetes (YAML)
├── scripts/               # Scripts auxiliares
└── docs/                  # Documentação adicional
```

### Fluxo Geral de Execução

1. **Entre no diretório do módulo**:
   ```bash
   cd "01 app health"
   ```

2. **Leia o README** específico do módulo

3. **Execute o setup do cluster** (se houver script):
   ```bash
   ./scripts/docker-desktop-setup.sh
   # ou
   ./scripts/kind-setup.sh
   ```

4. **Build das imagens** (quando aplicável):
   ```bash
   docker build -t nome-da-imagem:tag .
   # ou execute o script fornecido
   ./scripts/build.sh
   ```

5. **Aplique os manifestos Kubernetes**:
   ```bash
   kubectl apply -f k8s/
   # ou use Kustomize
   kubectl apply -k k8s/overlays/dev
   ```

6. **Acompanhe os recursos**:
   ```bash
   kubectl get pods -w
   kubectl logs -f <pod-name>
   ```

---

## 📖 Ordem Recomendada de Estudo

Para aproveitar melhor o conteúdo, recomendamos seguir esta sequência:

1. **App Health** (01) - fundamentos de observabilidade e health checks
2. **Advanced Scheduler** (02) - entender como o Kubernetes agenda workloads
3. **Rollout Updates** (03) - estratégias básicas de atualização
4. **Helm Charts** (04) - empacotamento e gestão de aplicações
5. **Blue/Green** (05) - deploy sem downtime
6. **Canary** (06) - progressive delivery avançado
7. **KEDA** (08) - autoscaling event-driven
8. **Karpenter** (07) - autoscaling de infraestrutura
9. **Security** (09) - práticas de segurança em produção

---

## 🔧 Troubleshooting Comum

### Imagens não encontradas

**Docker Desktop**: Certifique-se de que o Kubernetes está usando o mesmo daemon Docker:
```bash
kubectl config current-context  # deve retornar "docker-desktop"
```

**Kind**: Carregue as imagens manualmente:
```bash
kind load docker-image nome-da-imagem:tag --name nome-do-cluster
```

**Minikube**: Use o daemon do Minikube:
```bash
eval $(minikube docker-env)
# então faça o build novamente
```

### Pods não iniciam

```bash
# Verifique eventos
kubectl describe pod <pod-name>

# Verifique logs
kubectl logs <pod-name>

# Verifique recursos
kubectl top nodes
kubectl top pods
```

### Contexto errado do Kubernetes

```bash
# Liste contextos disponíveis
kubectl config get-contexts

# Mude para o contexto desejado
kubectl config use-context docker-desktop
```

---

## 📝 Licença

Este projeto é distribuído sob a licença **MIT**, podendo ser utilizado tanto para fins didáticos quanto profissionais.

---

## 🤝 Contribuições

Este é um repositório educacional da FIAP Pós Tech. Sugestões e melhorias são bem-vindas através de issues ou pull requests.

---

## 📬 Contato

Para dúvidas sobre o curso ou conteúdo técnico, consulte a documentação específica de cada módulo ou entre em contato através dos canais oficiais da FIAP Pós Tech.

---

**Desenvolvido com ❤️ para o curso Kubernetes Avançado - FIAP Pós Tech**
