# Aula 09 — Segurança em Kubernetes (Hands-on completo)

Este projeto foi desenvolvido para ser uma **demonstração prática e didática** dos principais conceitos abordados na **Aula 09 – Segurança no Cluster**, dentro do curso **Kubernetes Avançado (FIAP Pos Tech)**.
A proposta é que o aluno consiga experimentar, de forma **agnóstica** (ou seja, independente de provedor de nuvem), como aplicar **boas práticas de segurança** diretamente no cluster.

Os temas abordados são:

* **Identidade de workloads com ServiceAccounts** – garantindo que cada aplicação possua uma identidade própria, evitando o uso da conta `default`.
* **RBAC mínimo (Role/RoleBinding)** – implementação de permissões mínimas necessárias para reduzir a superfície de ataque.
* **Anotação de ServiceAccount para identidade federada** – integração com identidades gerenciadas em provedores de nuvem (EKS IRSA, GKE Workload Identity, AKS Managed Identity).
* **TLS automatizado com cert-manager** – emissão e renovação automática de certificados com Autoridade Certificadora (CA) interna.
* **Servidor Rust HTTPS (service-a)** – aplicação real servindo HTTPS, consumindo o certificado emitido pelo cert-manager.
* **Manifestos YAML organizados** – estrutura modular e reutilizável.
* **Documentação detalhada** – guia passo-a-passo e roteiro para apresentações técnicas.

> ✅ Este projeto foi validado no **Kubernetes embutido no Docker Desktop** e mantém instruções para adaptação a outros ambientes (Minikube, AKS, EKS, GKE).

---

## 1) Configuração completa do ambiente

### 💻 1.1 No Windows (recomendado via Chocolatey)

O [Chocolatey](https://chocolatey.org/) é um gerenciador de pacotes que simplifica a instalação de ferramentas de desenvolvimento. Para configurar o ambiente de forma rápida e padronizada, execute o **PowerShell como Administrador** e siga os comandos abaixo:

```powershell
# Instale o Chocolatey (caso ainda não tenha)
Set-ExecutionPolicy Bypass -Scope Process -Force; `
[System.Net.ServicePointManager]::SecurityProtocol = `
[System.Net.ServicePointManager]::SecurityProtocol -bor 3072; `
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# Feche e reabra o PowerShell antes de prosseguir
```

Agora instale as dependências principais:

```powershell
choco install -y docker-desktop kubernetes-cli helm rustup.install git make
```

* **Docker Desktop**: necessário para buildar e executar imagens de container (habilite o Kubernetes em *Settings → Kubernetes*).
* **kubectl**: ferramenta de linha de comando para comunicação com o cluster.
* **Helm**: gerenciador de pacotes Kubernetes, usado para instalar o cert-manager.
* **rustup** e **make**: ferramentas para compilar os crates em Rust e utilizar os scripts do projeto.
* **git**: opcional, mas útil para versionamento.

Depois da instalação, inicialize o ambiente Rust:

```powershell
rustup default stable
rustup update
```

Por fim, confirme que o Kubernetes do Docker Desktop está habilitado e selecionado:

```powershell
kubectl config current-context
kubectl get nodes
```

O contexto deve ser `docker-desktop` e os nós devem aparecer como `Ready`.

---

### 🐧 1.2 No Linux

No Linux (Ubuntu/Debian), instale os componentes base:

```bash
sudo apt update && sudo apt install -y curl git make docker.io docker-compose-plugin
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"

# kubectl
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl

# Helm
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
```

> 💡 Se estiver usando Docker Desktop para Linux, habilite o Kubernetes nas configurações e verifique o contexto com `kubectl config current-context`. Em outras distribuições (Minikube, k3d, MicroK8s) pode ser necessário publicar as imagens em um registry acessível ao cluster em vez de confiar no daemon Docker local.

### 🍎 1.3 No macOS (Homebrew)

```bash
brew install kubectl helm make git
brew install --cask docker
brew install rustup-init
rustup-init -y
rustup default stable
```

Inicie o Docker Desktop, habilite o Kubernetes nas preferências e confirme o contexto com `kubectl config current-context` (deve retornar `docker-desktop`).

---

## 2) Estrutura do projeto

A estrutura do diretório foi cuidadosamente planejada para que cada componente da aula represente um conceito teórico aplicado na prática:

```text
aula09-k8s-security/
├─ Cargo.toml                         # Workspace Rust (orchestrator + service-a)
├─ crates/
│  ├─ orchestrator/                   # CLI em Rust: cria SA, RBAC, Pod e faz anotações
│  └─ service-a/                      # Servidor HTTPS em Rust, certificado pelo cert-manager
├─ k8s-manifests/
│  ├─ analytics/                      # Namespace + RBAC + Pod de exemplo
│  ├─ cert-manager/                   # Issuer/CA/Certificate do serviço
│  ├─ service-a-deployment.yaml       # Deployment HTTPS (service-a)
│  └─ service-a-service.yaml          # Service ClusterIP (porta 8443)
├─ scripts/
│  ├─ kind-load-image.sh              # Auxiliar para clusters que não compartilham o daemon Docker
│  └─ create-ca-configmap.sh          # Cria ConfigMap com CA para validar TLS
└─ docs/
   ├─ ROTEIRO.md                      # Roteiro da apresentação técnica
   └─ PASSO-A-PASSO.md                # Guia de implantação agnóstico
```

Cada subdiretório foi criado para isolar responsabilidades:

* **`crates/`** contém o código-fonte Rust: o *orchestrator* (CLI) e o *service-a* (API HTTPS).
* **`k8s-manifests/`** traz todos os manifests YAML aplicáveis ao cluster.
* **`scripts/`** inclui ferramentas auxiliares de linha de comando.
* **`docs/`** centraliza o material didático e o roteiro da aula.

---

## 3) Execução passo a passo

> O guia completo e detalhado está em [`docs/PASSO-A-PASSO.md`](docs/PASSO-A-PASSO.md).
> Aqui, apresentamos um resumo prático e comentado.

Antes de iniciar os comandos, confirme que o contexto ativo é o `docker-desktop`:

```bash
kubectl config current-context
```

Se necessário, ajuste com `kubectl config use-context docker-desktop`.

### 🧩 3.1 Instalação do cert-manager

O cert-manager é o responsável por emitir e renovar automaticamente certificados TLS dentro do cluster. Para instalá-lo de maneira simples, use o Helm:

```bash
helm repo add jetstack https://charts.jetstack.io
helm repo update
helm install cert-manager jetstack/cert-manager \
  --namespace cert-manager --create-namespace \
  --set crds.enabled=true
```

Após a instalação, confirme o funcionamento:

```bash
kubectl -n cert-manager get pods
```

Todos os pods devem aparecer com o status `Running`.

---

### 🔒 3.2 Aplicando certificados e o serviço Rust

Os manifests YAML já estão prontos para criar a CA, o certificado do serviço e o deployment do microsserviço em Rust. Basta aplicar:

```bash
kubectl apply -f k8s-manifests/cert-manager/issuers-and-certs.yaml
kubectl apply -f k8s-manifests/service-a-deployment.yaml
kubectl apply -f k8s-manifests/service-a-service.yaml
```

O cert-manager criará automaticamente o Secret `service-a-tls` contendo os arquivos `tls.crt` e `tls.key`.
Esse Secret é montado no container Rust e usado pelo Actix-web para servir HTTPS de forma segura.

---

### 🧰 3.3 Criando identidades e permissões (ServiceAccount + RBAC)

Nesta etapa, criamos a identidade do workload (`ServiceAccount`) e configuramos permissões mínimas com **RBAC**.

Para reproduzir tudo automaticamente via código Rust:

```bash
cargo run -p orchestrator -- bootstrap
```

Esse comando cria:

1. O namespace `analytics`
2. A ServiceAccount `analytics-sa`
3. A Role `analytics-read` (leitura de pods e configmaps)
4. O RoleBinding correspondente
5. Um Pod de teste usando essa identidade

Caso queira aplicar tudo manualmente via YAML:

```bash
kubectl apply -f k8s-manifests/analytics/namespace.yaml
kubectl apply -f k8s-manifests/analytics/rbac.yaml
kubectl apply -f k8s-manifests/analytics/pod.yaml
```

---

### 🔍 3.4 Validando o princípio do menor privilégio

Agora, testamos se o RBAC está realmente aplicando as restrições:

```bash
# Permissão esperada (OK)
kubectl auth can-i --as=system:serviceaccount:analytics:analytics-sa get pods -n analytics

# Permissão negada (esperado)
kubectl auth can-i --as=system:serviceaccount:analytics:analytics-sa create pods -n analytics
```

Esses comandos ilustram o **princípio do menor privilégio**, garantindo que workloads só possam realizar ações necessárias e nenhuma a mais.

---

### 🧪 3.5 Testando a comunicação HTTPS dentro do cluster

Com o serviço HTTPS em execução e certificado válido, validamos a conexão criptografada:

1. Gere um ConfigMap com a CA usada pelo cert-manager:

   ```bash
   ./scripts/create-ca-configmap.sh
   ```

2. Crie um Pod temporário para testar:

   ```bash
   kubectl -n default run tls-tester --image=alpine:3.19 -it --rm -- \
     sh -lc "apk add --no-cache curl && curl --cacert /ca/ca.crt https://service-a.default.svc.cluster.local:8443/healthz"
   ```

3. Se tudo estiver correto, você verá:

   ```json
   {"status":"ok"}
   ```

Isso comprova que o certificado foi emitido corretamente e a aplicação está servindo HTTPS seguro dentro do cluster.

---

### ☁️ 3.6 (Opcional) Anotando a ServiceAccount para Identidades em Cloud

Quando trabalhamos em provedores de nuvem, podemos vincular identidades externas (IAM Roles, Service Accounts do GCP, Managed Identities do Azure) diretamente à nossa SA.
O *orchestrator* facilita isso com um simples comando:

```bash
# AWS EKS
cargo run -p orchestrator -- annotate --provider eks --value arn:aws:iam::123456789012:role/S3Reader

# Google GKE
cargo run -p orchestrator -- annotate --provider gke --value meu-servico@projeto.iam.gserviceaccount.com

# Azure AKS
cargo run -p orchestrator -- annotate --provider aks --value <AZURE_CLIENT_ID_DA_MANAGED_IDENTITY>
```

Essas anotações permitem autenticação federada sem uso de chaves fixas, reduzindo drasticamente riscos de vazamento.

---

## 4) Por que cada passo é importante

Cada etapa da prática está vinculada a um **pilar de segurança em Kubernetes**:

* **ServiceAccount dedicada**: define uma identidade única para cada aplicação, isolando permissões e eliminando o uso de `default`.
* **RBAC mínimo**: garante o princípio do menor privilégio e reduz vetores de ataque laterais.
* **TLS automatizado**: remove a necessidade de emitir certificados manualmente, evitando falhas humanas e interrupções por expiração.
* **Servidor HTTPS em Rust**: demonstra na prática a integração segura entre aplicação e infraestrutura.
* **Identidade federada (anotações)**: conecta o cluster Kubernetes com provedores externos de identidade, seguindo as melhores práticas de segurança em nuvem.

---

## 5) Limpeza do ambiente

Após a conclusão dos testes, é importante remover os recursos criados para manter o cluster limpo:

```bash
cargo run -p orchestrator -- cleanup
kubectl delete -f k8s-manifests --ignore-not-found
helm uninstall cert-manager -n cert-manager || true
```

Isso deleta o namespace, ServiceAccount, roles, pods e certificados gerados.

Se quiser liberar recursos rapidamente no Docker Desktop, basta desativar temporariamente o Kubernetes nas configurações ou encerrar o aplicativo.

---

## 6) Licença

Este projeto é distribuído sob a licença **MIT**, podendo ser utilizado tanto para fins didáticos quanto profissionais.

---

Feito com ❤️ para a aula do curso **K8s Avançado – FIAP Pos Tech**.
