# Kubernetes Avançado — Aula 03 (Rollouts & Estratégias de Atualização)

Este guia **didático e detalhado** descreve **cada passo** para instalar as dependências, subir um cluster local, construir e publicar a aplicação (em **Rust**), aplicar os **YAMLs** e **conduzir a apresentação** demonstrando RollingUpdate, Recreate e (opcionalmente) Progressive Delivery com Argo Rollouts.

> **Objetivo pedagógico**: ao final, você conseguirá **explicar o porquê de cada parâmetro** (`maxSurge`, `maxUnavailable`, `minReadySeconds`, probes, etc.), **executar a demo do zero** em qualquer sistema (Windows/macOS/Linux), e **comparar cenários** (Rolling vs Recreate vs Canary).

---

## 0) O que você vai instalar (e por quê)

| Componente                   | Para que serve                                                                        | Instalação rápida                                                              |
| ---------------------------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| **Docker**                   | Build e execução da imagem container da nossa app Rust.                               | Windows/macOS: **Docker Desktop**; Linux: `apt/yum` + `docker compose plugin`. |
| **kubectl**                  | CLI oficial do Kubernetes para aplicar YAMLs e inspecionar recursos.                  | Binário oficial (curl/choco/brew).                                             |
| **Kubernetes (Docker Desktop)** | Cluster local integrado ao Docker Desktop, usado para aplicar os manifests.          | Ative nas preferências do Docker Desktop (`Settings > Kubernetes`).            |
| **Rust toolchain**           | Compilar os binários `simulator` e `myapp`.                                           | `rustup` (instalador oficial).                                                 |
| **(Opcional) Argo Rollouts** | Controlador para canário/blue-green/experimentos.                                     | Aplicar manifests do projeto Argo e (opcional) plugin `kubectl-argo-rollouts`. |



---

## 1) Instalação — passo a passo por sistema operacional

### Windows 10/11 (PowerShell)

1. **Docker Desktop**
   Baixe e instale; confirme que o serviço está “Running” e habilite `Settings > Kubernetes > Enable Kubernetes`.

2. **kubectl**

   ```powershell
   choco install kubernetes-cli -y
   kubectl version --client
   ```

3. **Rust**
   Baixe o instalador de [https://rustup.rs/](https://rustup.rs/) e execute.
   Depois confirme:


   ```powershell
   rustc --version
   cargo --version
   ```

4. **(Opcional) Argo Rollouts**


   ```powershell
   choco install argoproj-argo-rollouts -y
   kubectl argo rollouts version
   ```

---

### macOS (Terminal)

1. **Docker Desktop** (Apple Silicon ou Intel).
   Instale, confirme que está “Running” e habilite Kubernetes nas preferências.


2. **Homebrew**
   Se não tiver: [https://brew.sh](https://brew.sh)

3. **kubectl e Rust**

   ```bash

   brew install kubectl rustup-init
   rustup-init -y
   source $HOME/.cargo/env
   kubectl version --client
   rustc --version && cargo --version
   ```

4. **(Opcional) Argo Rollouts**

   ```bash
   brew install argoproj/tap/kubectl-argo-rollouts
   kubectl argo rollouts version
   ```

---

### Linux (Ubuntu/Debian-like)

1. **Docker Engine**

   ```bash
   sudo apt-get update
   sudo apt-get install -y ca-certificates curl gnupg
   curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
   echo "deb [arch=$(dpkg --print-architecture)] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list
   sudo apt-get update
   sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
   sudo usermod -aG docker $USER && newgrp docker
   docker info
   ```

2. **kubectl**

   ```bash
   curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
   chmod +x kubectl
   sudo mv kubectl /usr/local/bin/
   kubectl version --client
   ```

3. **Rust**

   ```bash
   curl https://sh.rustup.rs -sSf | sh -s -- -y
   source $HOME/.cargo/env
   rustc --version && cargo --version
   ```

4. **(Opcional) Argo Rollouts**

   ```bash
   curl -sL https://github.com/argoproj/argo-rollouts/releases/latest/download/kubectl-argo-rollouts-linux-amd64 -o kubectl-argo-rollouts
   chmod +x kubectl-argo-rollouts
   sudo mv kubectl-argo-rollouts /usr/local/bin/
   kubectl argo rollouts version
   ```

💡 No Linux, use o Docker Desktop (que também disponibiliza o contexto `docker-desktop`) ou um equivalente como `minikube`/`k3d`. Os manifests deste módulo assumem que o cluster compartilha o daemon Docker local.

---

## 2) Estrutura do projeto

```text
k8s-advanced-aula03/
├─ rust/
│  ├─ simulator/        # Simulador do algoritmo de RollingUpdate
│  └─ myapp/            # Microserviço Rust (Axum)
├─ k8s/
│  ├─ service.yaml
│  ├─ deployment-rolling.yaml
│  ├─ deployment-recreate.yaml
│  └─ argo/
│     ├─ rollout-canary.yaml
│     └─ analysis-template.yaml
└─ scripts/
   ├─ docker-desktop-setup.sh
   ├─ build.sh
   ├─ push.sh
   ├─ deploy-rolling.sh
   ├─ deploy-recreate.sh
   ├─ deploy-argo.sh
   ├─ watch-rollout.sh
   └─ cleanup.sh
```

---

## 3) Configurando o cluster local (Docker Desktop)

```bash
scripts/docker-desktop-setup.sh
```

O script apenas garante que o contexto `docker-desktop` está selecionado e mostra o status do cluster. Caso não apareçam nós `Ready`, abra o Docker Desktop e aguarde o Kubernetes concluir a inicialização.

---

## 4) Build da aplicação Rust

```bash
scripts/build.sh
```

O script:

* Compila o binário `myapp` em modo release via Docker multi-stage;
* Gera uma imagem local `myorg/myapp:1.0.0` já disponível para o cluster `docker-desktop` (não requer push);
* Opcionalmente executa o container em `localhost:8080` para testes.

> Precisa enviar a imagem para um registry remoto (por exemplo, Docker Hub)? Execute `scripts/push.sh` após definir `IMAGE=seuusuario/myapp:tag`.

---

## 5) Implantando com RollingUpdate

```bash
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/deployment-rolling.yaml
kubectl rollout status deployment/myapp
scripts/watch-rollout.sh
```

Agora edite `k8s/deployment-rolling.yaml`:

* Troque a imagem para `:2.0.0`
* Troque `APP_VERSION=2.0.0`

A seguir, gere a nova imagem local e reaplique o manifest:

```bash
IMAGE=myorg/myapp:2.0.0 scripts/build.sh
kubectl apply -f k8s/deployment-rolling.yaml
kubectl rollout status deployment/myapp
```

Se a porta `8080` já estiver em uso no host, execute `HOST_PORT=8081 IMAGE=myorg/myapp:2.0.0 scripts/build.sh` ou defina `SKIP_RUN=1` para apenas construir a imagem.

Acesse:

```bash
kubectl port-forward svc/myapp 8080:80 &
curl localhost:8080/version
```

---

## 6) Simulando o algoritmo de RollingUpdate

```bash
cargo run -p simulator -- 10 25% 25%
cargo run -p simulator -- 12 0% 50%
```

O simulador mostra:

* Quantos pods novos são criados a cada passo;
* Quantos antigos são removidos;
* Quantos permanecem disponíveis.

Isso ajuda a **visualizar** o equilíbrio entre velocidade e disponibilidade.

---

## 7) Recreate (sem coexistência)

```bash
kubectl apply -f k8s/deployment-recreate.yaml
kubectl rollout status deployment/myapp-recreate
```

Usado quando versões não podem coexistir (mudanças de schema ou estado incompatíveis).

---

## 8) (Opcional) Canary com Argo Rollouts

1. Instale o Argo Rollouts Controller:

   ```bash
   kubectl apply -n argo-rollouts -f https://github.com/argoproj/argo-rollouts/releases/latest/download/install.yaml
   ```
2. Aplique os manifests:

   ```bash
   kubectl apply -f k8s/argo/analysis-template.yaml
   kubectl apply -f k8s/argo/rollout-canary.yaml
   kubectl argo rollouts get rollout myapp -w
   ```

---

## 9) Alternativas

### Minikube

```bash
minikube start
eval $(minikube docker-env)
scripts/build.sh
kubectl apply -f k8s/
```

### k3d

```bash
k3d registry create reg.localhost --port 5001
k3d cluster create aula03 --registry-use k3d-reg.localhost:5001
scripts/build.sh && scripts/push.sh
```

### Cloud (AKS/GKE/EKS)

Use os mesmos YAMLs; altere apenas o `image` e adicione `imagePullSecrets` se necessário.

---

## 10) Troubleshooting

| Sintoma                      | Causa possível                        | Solução                                            |
| ---------------------------- | ------------------------------------- | -------------------------------------------------- |
| `permission denied (docker)` | Usuário não no grupo docker           | `sudo usermod -aG docker $USER && newgrp docker`   |
| `ImagePullBackOff`           | Tag incorreta ou imagem não publicada | Verifique `docker images` e `kubectl describe pod` |
| Readiness não fica ok        | Endpoint incorreto                    | Verifique `/readyz` no container                   |
| Rollout preso                | Probes falhando                       | Veja eventos com `kubectl describe deploy`         |
| Argo não responde            | Controller ausente                    | Reaplique manifest do Argo Rollouts                |

---

## 11) Limpeza

```bash
scripts/cleanup.sh
```

Depois da demo você pode desabilitar o Kubernetes nas preferências do Docker Desktop se quiser liberar recursos.

---

## TL;DR

```bash
scripts/docker-desktop-setup.sh
scripts/build.sh
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/deployment-rolling.yaml
scripts/watch-rollout.sh
# edite a tag no YAML (ex.: :2.0.0) e reaplique para observar o rollout
```
