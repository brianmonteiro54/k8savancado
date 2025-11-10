# Aula 05 — Deploy Blue/Green no Kubernetes (com Rust)

Este pacote entrega **código em Rust**, **YAMLs do Kubernetes**, um **orquestrador em Rust (kube-rs)** para comutar Blue⇄Green, e um **roteiro completo de apresentação** para a aula.

> **Objetivo:** Demonstrar, na prática, deploy **Blue/Green** com **rollback instantâneo**, usando um **único container Rust** e dois *Deployments* (blue/green) alternados por um **Service**.
> **Ideia central:** o tráfego nunca para — apenas muda de destino.

---

## Visão geral do projeto

O ambiente foi construído para mostrar de forma prática cada componente de um rollout Blue/Green.
A estrutura geral é a seguinte:

* **`service/`** – Microserviço HTTP escrito em Rust (Axum):

  * Expõe:

    * `GET /` → retorna JSON com `version`, `color` e `hostname`.
      *Serve como prova visual de qual versão está respondendo.*
    * `GET /healthz` → endpoint usado para *liveness* e *readiness probes*.
  * Cada versão (Blue ou Green) é **idêntica no código**, diferindo apenas nas variáveis de ambiente.

* **`k8s/`** – Manifestos Kubernetes (`YAML`):

  * `namespace.yaml` – cria o namespace isolado da aula.
  * `deployment-blue.yaml` e `deployment-green.yaml` – definem as duas versões simultâneas da aplicação.
  * `service.yaml` – define o ponto fixo de entrada do tráfego, que alterna entre blue e green.
  * `README.md` explica cada campo e seu papel no ciclo de vida.

* **`orchestrator/`** – Aplicação CLI em Rust utilizando `kube-rs`:

  * Permite automatizar o fluxo:

    * `init` → cria todos os objetos.
    * `status` → verifica qual versão está ativa.
    * `switch --to blue|green` → realiza o cutover entre ambientes.
    * `cleanup` → limpa o namespace por completo.
  * *Motivo:* abstrair o kubectl e mostrar automação declarativa nativa via client API.

* **`scripts/`** – Utilitários:

  * `docker-desktop-setup.sh` – garante que o contexto `docker-desktop` está ativo e funcionando.
  * `demo.sh` – executa a demonstração automatizada (Blue → Green → rollback).


> **Por que esse design?**
> Ele é agnóstico e independente. Pode rodar em Kind, Minikube, AKS, EKS, GKE ou qualquer cluster compatível.
> A imagem Docker é **única**; as diferenças de comportamento são injetadas por **labels** e **env vars**.

---

## Pré-requisitos e justificativa

* **Docker 24+** — necessário para construir e carregar a imagem.
* **kubectl 1.28+** — principal interface de controle do Kubernetes.
* **Cluster Kubernetes funcional** — qualquer distribuição.
* **(Opcional) Rust 1.80+ e Cargo** — para compilar o orquestrador localmente.

> Por que não exigimos ferramentas específicas?
> Para demonstrar o princípio de **portabilidade de ambiente**: tudo pode ser reproduzido em qualquer lugar, sem depender de vendors.

---

## Passo a passo detalhado

eval $(minikube docker-env)

### 0) Subir o cluster local

#### Docker Desktop Kubernetes

```bash
scripts/docker-desktop-setup.sh
```

O script valida se o contexto `docker-desktop` existe, seleciona-o e mostra o status do cluster. Caso o contexto não esteja disponível, habilite **Settings > Kubernetes > Enable Kubernetes** no Docker Desktop e aguarde até que os nós apareçam como `Ready`.

> **Por que Docker Desktop?**
> Ele compartilha o mesmo daemon Docker do host, então qualquer imagem construída localmente fica imediatamente disponível para o cluster — sem `kind load` ou registry auxiliar.

---

### 1) Build da imagem do serviço

```bash
cd service
docker build -t myapp:latest .
```

* Compilamos o binário Rust no estágio `builder` e copiamos apenas o executável para a imagem final baseada em Debian slim.
* É uma **imagem leve, imutável e determinística**, ideal para ambientes de produção.

> A imagem é compartilhada por Blue e Green. A distinção vem das **variáveis de ambiente** (`COLOR`, `VERSION`).

---

### 2) Aplicar os manifests Kubernetes

```bash
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/deployment-blue.yaml
kubectl apply -f k8s/deployment-green.yaml
kubectl apply -f k8s/service.yaml
```

**Explicação de cada recurso:**

* `namespace.yaml` — cria um espaço lógico para isolar a aula.
* `deployment-blue.yaml` — define o ambiente ativo inicial (v1.0).
* `deployment-green.yaml` — prepara a nova versão (v2.0) aguardando promoção.
* `service.yaml` — o **ponto estável de rede**. Ele define o selector `env: blue`, conectando o tráfego à versão ativa.

> Aqui ocorre o conceito central: **os dois ambientes coexistem**, mas **apenas um recebe tráfego**.
> O Service é o interruptor lógico da troca.

---

### 3) Testar a versão ativa (Blue)

```bash
kubectl -n aula05 port-forward svc/myapp 8080:80
curl -s http://localhost:8080/ | jq .
```

Você deve ver algo como:

```json
{
  "service": "myapp",
  "version": "1.0",
  "color": "blue",
  "hostname": "myapp-blue-7d4..."
}
```

> Isso comprova que o Service está roteando corretamente para o conjunto de Pods com `env: blue`.
> O `hostname` facilita a visualização de load balancing entre réplicas.

---

### 4) Promover GREEN (cutover Blue → Green)

```bash
kubectl -n aula05 patch svc myapp -p '{"spec":{"selector":{"app":"myapp","env":"green"}}}'
```

Esse comando altera **apenas o selector do Service**, trocando a origem do tráfego de `env=blue` para `env=green`.
Nenhum Pod é reiniciado, nenhuma conexão é quebrada.

Validação:

```bash
curl -s http://localhost:8080/ | jq .
```

Agora a resposta deve conter `"color": "green"`, `"version": "2.0"`.

> **Por que não usar rolling update?**
> Porque queremos **paralelismo total** entre as versões.
> No Blue/Green, a versão anterior continua viva até o momento do corte e rollback possível.

---

### 5) Rollback imediato (Green → Blue)

```bash
kubectl -n aula05 patch svc myapp -p '{"spec":{"selector":{"app":"myapp","env":"blue"}}}'
```

Rollback = trocar novamente o selector.
Como ambas as versões seguem prontas, o retorno é **instantâneo**.

> **Motivo técnico:** rollback Blue/Green não envolve recriar pods nem manipular réplicas.
> É apenas uma operação de metadata sobre o objeto Service, garantida pelo Kubernetes API Server — rápida, transacional e auditável.

---

## Orquestrador em Rust (CLI)

O diretório `orchestrator/` contém uma aplicação CLI construída com **kube-rs**, demonstrando como se pode interagir com a API Kubernetes diretamente, sem usar `kubectl`.

### Principais comandos

```bash
cargo build --release
./target/release/orchestrator init
./target/release/orchestrator status
./target/release/orchestrator switch --to green
./target/release/orchestrator status
./target/release/orchestrator switch --to blue
./target/release/orchestrator cleanup
```

#### Explicando a implementação

* `init` → cria Namespace, Deployments Blue/Green e Service.
* `status` → lê o selector atual do Service e exibe a cor ativa.
* `switch --to` → aplica o mesmo patch declarativo do exemplo anterior.
* `cleanup` → remove o namespace inteiro para reset do ambiente.

> **Por que criar um orquestrador?**
>
> * Mostra como o Rust pode se integrar à API Kubernetes (client nativo).
> * Ilustra automação idempotente (executar duas vezes não quebra nada).
> * Pode ser adaptado para pipelines GitOps, operadores ou CI/CD reais.

---

## Roteiro de apresentação (live demo)

O arquivo `ROTEIRO.md` traz:

* A sequência narrativa da aula (abertura, arquitetura, testes, cutover, rollback).
* Comandos organizados por tempo estimado.
* Tópicos de discussão sobre boas práticas e observabilidade.

> **Importância pedagógica:**
> Reforça o raciocínio por trás do modelo Blue/Green e ensina a ler os objetos Kubernetes como entidades vivas (metadata, labels, selectors).

---

## Por que este design?

| Decisão                           | Justificativa                                                      |
| --------------------------------- | ------------------------------------------------------------------ |
| **Um único container**            | Reduz complexidade e garante consistência binária.                 |
| **Distinção via labels/env vars** | Mantém versionamento declarativo, visível e auditável.             |
| **Service como chave do tráfego** | Evita downtime e mantém rollback atômico.                          |
| **Orquestrador em Rust**          | Demonstra uso prático do client `kube-rs` e automação idempotente. |
| **YAMLs limpos e portáveis**      | Funcionam em qualquer cluster e podem ser versionados em GitOps.   |
| **Scripts auxiliares**            | Reduzem dependências e promovem reprodutibilidade.                 |

---

## Conclusão

Este projeto foi desenhado para que cada comando executado **tenha um propósito técnico e pedagógico**:

* **Mostrar** como o Kubernetes gerencia versões paralelas.
* **Ensinar** o conceito de atomicidade de tráfego via Service.
* **Evidenciar** a integração de Rust no ecossistema de DevOps moderno.

> O resultado é um ambiente simples, autônomo e reproduzível — pronto para ensinar Blue/Green na prática, com rollback em segundos e total rastreabilidade.
