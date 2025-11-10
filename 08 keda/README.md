# KEDA + Rust (RabbitMQ Queue) — Demo de Escalonamento Event-Driven com Scale-to-Zero

Este projeto tem como objetivo demonstrar, de forma prática e didática, o conceito de **escalabilidade orientada a eventos (event-driven)** no Kubernetes utilizando o **KEDA (Kubernetes Event-driven Autoscaler)**. Para isso, construímos dois serviços simples em **Rust** que simulam um fluxo produtivo de mensagens:

- **`worker`**: um consumidor que processa mensagens de uma fila RabbitMQ chamada `orders`, sendo automaticamente escalado (aumentando ou reduzindo o número de réplicas) pelo KEDA, de 0 até N pods, conforme o tamanho da fila.
- **`publisher`**: um publicador que gera mensagens artificialmente na fila `orders`, simulando um aumento de carga que aciona o autoscaling.

A demonstração cobre todos os **conceitos práticos da Aula 08 de Kubernetes Avançado**, incluindo:
- Integração entre **KEDA e HPA**;
- Gatilhos externos baseados em fila RabbitMQ;
- Mecanismos de **scale-to-zero**, **cooldown**, **polling** e **histerese**;
- Demonstração em **ambiente agnóstico** (validada no Kubernetes do Docker Desktop, com adaptações simples para Minikube, AKS, EKS e GKE).

---

## 1. Arquitetura Geral do Projeto

A arquitetura do ambiente foi desenhada para mostrar, de ponta a ponta, como o KEDA responde automaticamente a eventos externos (mensagens na fila):

```

+-------------+        amqp://       +------------------+        KEDA -> HPA -> K8s
| publisher   |  --->  RabbitMQ  --->| worker (Rust)    |  --->  escala 0..N pods
+-------------+        (Cluster)     +------------------+

````

Explicando cada componente:

- **RabbitMQ**: atua como o sistema de mensageria dentro do cluster, permitindo que o publisher e o worker troquem mensagens via protocolo AMQP.
- **Publisher**: insere mensagens na fila `orders`, simulando o acúmulo de tarefas a serem processadas.
- **Worker**: consome mensagens dessa fila e processa cada uma delas. Ele será escalado automaticamente com base no tamanho da fila.
- **KEDA**: observa a métrica de backlog (quantidade de mensagens) e envia métricas personalizadas para o **Horizontal Pod Autoscaler (HPA)** do Kubernetes.
- **HPA**: decide quantos pods o worker deve ter em execução naquele momento.
- **Scale-to-Zero**: quando não há mais mensagens a processar, o KEDA reduz o número de pods para **zero**, liberando completamente os recursos.

---

## 2. Configuração do Ambiente

Antes de iniciar, precisamos preparar o ambiente de desenvolvimento. A ideia é que qualquer aluno ou profissional possa reproduzir o experimento em **Windows**, **Linux** ou **macOS**.

### 2.1 Windows (via Chocolatey)

Execute o **PowerShell como Administrador** e instale as dependências:

```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force; `
[System.Net.ServicePointManager]::SecurityProtocol = `
[System.Net.ServicePointManager]::SecurityProtocol -bor 3072; `
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

choco install docker-desktop kubernetes-cli helm rustup.install make git -y```

Em seguida, configure o toolchain estável do Rust:

```powershell
rustup default stable
```

Valide as versões principais (e confirme que o Kubernetes está habilitado no Docker Desktop em *Settings → Kubernetes*):

```powershell
docker --version
kubectl version --client
helm version
rustc --version
kubectl config current-context
```

O último comando deve retornar `docker-desktop`.

### 2.2 Linux (Ubuntu/Debian)

```bash
sudo apt update && sudo apt install -y curl git make docker.io docker-compose-plugin
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
sudo snap install kubectl --classic
sudo snap install helm --classic
```

No Linux, você pode usar o Docker Desktop (habilitando o Kubernetes nas configurações) ou uma distribuição compatível como Minikube/k3d. Caso o cluster não compartilhe o daemon Docker do host, será necessário **publicar as imagens** em um registry acessível antes de aplicar os Deployments.

### 2.3 macOS (Homebrew)

```bash
brew install kubectl helm make git
brew install --cask docker
brew install rustup-init
rustup-init -y
rustup default stable
```

Inicie o Docker Desktop, habilite o Kubernetes e valide o contexto com `kubectl config current-context`.

---

## 3. Estrutura do Projeto e YAMLs

Os arquivos do projeto estão organizados de forma modular, em pastas separadas:

- `worker/` — código fonte do consumidor (Rust)
- `publisher/` — código fonte do gerador (Rust)
- `k8s/` — todos os manifestos Kubernetes (YAML)
- `README.md` — documentação detalhada (este arquivo)

Cada arquivo YAML possui uma função bem definida no cluster. Vamos detalhar abaixo:

1. **Namespace e Segredos**

   - `00-namespace.yaml` cria o namespace `keda-demo`, isolando todos os recursos.
   - `10-rabbitmq-secret.yaml` contém a connection string codificada em base64 usada pelo RabbitMQ e pelo KEDA para autenticação.

2. **RabbitMQ**

   - `11-rabbitmq.yaml` instala o RabbitMQ no cluster com o dashboard de administração.
   - `12-rabbitmq-svc.yaml` cria um `Service` para expor as portas 5672 (mensageria AMQP) e 15672 (UI de administração).

3. **Worker**

   - `20-worker-deploy.yaml` define o deployment inicial com `replicas: 0`, pois o KEDA controlará a escala.
   - `21-worker-svc.yaml` cria um serviço HTTP interno com endpoint `/healthz` para checagem de saúde.

4. **KEDA**

    - `30-trigger-auth.yaml` cria o recurso `TriggerAuthentication`, que vincula o segredo de conexão do RabbitMQ.
    - `31-scaledobject.yaml` define o comportamento de escalonamento:

       - `minReplicaCount: 0` ativa o scale-to-zero;
       - `maxReplicaCount: 10` limita o número máximo de réplicas;
       - `cooldownPeriod: 60` e `pollingInterval: 15` configuram o ritmo de sondagem;
       - o gatilho `rabbitmq` observa a fila `orders` e define 10 mensagens por pod como alvo.

5. **Publisher**

   - `40-publisher-job.yaml` cria um Job Kubernetes que publica N mensagens na fila, simulando carga e provocando o autoscaling do `worker`.

---

## 4. Passo a Passo de Execução (Docker Desktop Kubernetes)

### 4.1 Validar o cluster embutido

1. Abra o Docker Desktop e habilite o Kubernetes em *Settings → Kubernetes* (se ainda não estiver ativo).
2. No terminal, confirme que o contexto atual é o `docker-desktop`:

   ```bash
   kubectl config current-context
   ```

   Caso outro contexto esteja selecionado, troque com `kubectl config use-context docker-desktop`.
3. Confira nós e namespaces básicos para garantir que o cluster está saudável:

   ```bash
   kubectl get nodes
   kubectl get ns
   ```

### 4.2 Construir as imagens

Na raiz do projeto, construa as imagens locais dos serviços Rust:

```bash
docker build -t keda-demo/worker:local ./worker
docker build -t keda-demo/publisher:local ./publisher
```

Como o Kubernetes do Docker Desktop compartilha o daemon Docker com o host, **não é necessário** executar comandos extras para carregar as imagens no cluster. Apenas garanta que os manifests referenciem as mesmas tags.

### 4.3 Instalar o KEDA

Agora vamos instalar o KEDA com Helm (gerenciador de pacotes Kubernetes):

```bash
helm repo add kedacore https://kedacore.github.io/charts
helm repo update
helm install keda kedacore/keda -n keda --create-namespace
kubectl -n keda get pods
```

Você deve ver pods como `keda-operator` e `keda-metrics-apiserver` em execução.

### 4.4 Aplicar os Manifestos do Projeto

```bash
kubectl apply -f k8s/00-namespace.yaml
kubectl apply -f k8s/10-rabbitmq-secret.yaml
kubectl apply -f k8s/11-rabbitmq.yaml
kubectl apply -f k8s/12-rabbitmq-svc.yaml
kubectl apply -f k8s/20-worker-deploy.yaml
kubectl apply -f k8s/21-worker-svc.yaml
kubectl apply -f k8s/keda/30-trigger-auth.yaml
kubectl apply -f k8s/keda/31-scaledobject.yaml
```

Após isso, verifique se tudo foi criado corretamente:

```bash
kubectl -n keda-demo get all
```

Note que o `worker` estará com **zero pods ativos** — o comportamento esperado do scale-to-zero.

### 4.5 Gerar Backlog de Mensagens (para disparar o autoscaling)

#### Opção A — Usar o Job Kubernetes

```bash
kubectl apply -f k8s/40-publisher-job.yaml -n keda-demo
kubectl -n keda-demo logs job/publisher --follow
```

O job criará mensagens na fila `orders`. Em poucos segundos, o KEDA detectará o aumento da fila e iniciará novos pods `worker`.

#### Opção B — Executar Localmente com Port Forward

```bash
kubectl -n keda-demo port-forward svc/rabbitmq 5672:5672 &
cargo run -p publisher -- --count 200 --concurrency 20
```

---

## 5. Observando o Escalonamento em Tempo Real

Enquanto o sistema processa as mensagens, monitore o comportamento do autoscaling:

```bash
kubectl -n keda-demo get hpa -w
kubectl -n keda-demo get deploy/worker -w
kubectl -n keda-demo describe hpa -l app=worker
```

Você verá o número de réplicas subir (de 0 até N), conforme o backlog aumenta, e descer novamente após o cooldown.

---

## 6. Conceitos Didáticos e Pontos de Reflexão

Durante a demonstração, é importante destacar que:

- O **KEDA** age como um mediador entre métricas externas e o **HPA**, criando escalabilidade orientada a eventos.
- O parâmetro `minReplicaCount: 0` é o que efetivamente permite o **scale-to-zero**.
- O `pollingInterval` determina o ritmo de sondagem das métricas, e o `cooldownPeriod` evita oscilações abruptas (histerese).
- O valor `value: 10` significa que, para cada 10 mensagens em fila, um novo pod será criado.
- O **scale down** ocorre apenas quando a fila zera e o tempo de cooldown é respeitado.

---

## 7. Variantes e Portabilidade

- **Minikube/k3d/MicroK8s**: dependendo do driver, talvez seja necessário usar `minikube image load`/`k3d image import` **ou** fazer push das imagens para um registry acessível ao cluster.
- **AKS/EKS/GKE**: envie as imagens para um registry (ACR, ECR ou GCR) e ajuste os YAMLs com o caminho completo das imagens.
- **Outros gatilhos do KEDA**: o mesmo padrão se aplica a triggers para Kafka, Azure Queue, AWS SQS, Prometheus e muitos outros.

---

## 8. Solução de Problemas (Troubleshooting)

- **Pods não sobem**: verifique se o KEDA está rodando (`kubectl -n keda get pods`) e se o RabbitMQ está acessível (`kubectl -n keda-demo get pods`).
- **Sem scale-to-zero**: confirme se `minReplicaCount: 0` está definido e se não há mensagens residuais na fila.
- **Publisher falhou**: cheque o `port-forward` e a variável `AMQP_ADDR` do job.
- **Sem métricas no HPA**: alguns provedores gerenciados requerem permissões adicionais (ClusterRole e RoleBinding) para permitir coleta de métricas customizadas.

---

## 9. Encerramento e Limpeza do Ambiente

Após o término da demonstração, você pode remover todos os recursos criados com:

```bash
kubectl delete ns keda-demo
helm uninstall keda -n keda
```

Se desejar liberar recursos imediatamente no Docker Desktop, desative o Kubernetes temporariamente nas configurações ou apenas encerre o aplicativo.

---

## 10. Licença

Distribuído sob a licença **MIT**.

---

Feito com ❤️ para a aula do curso **K8s Avançado - FIAP Pos Tech**.
