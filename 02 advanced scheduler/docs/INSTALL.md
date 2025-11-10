# Guia Detalhado de Instalação & Execução — Advanced Scheduler (Aula 02)

Este documento transforma a lista curta de comandos em um **passo a passo pedagógico**, explicando **o que cada etapa faz**, **por que ela existe** e **como validar** que tudo funcionou. A ideia é que você consiga demonstrar conceitos de scheduling Kubernetes (prioridades, preempção, taints/tolerations, overcommit/Quality of Service e afinidade) de forma reprodutível em qualquer sistema.

---

## 🎯 Objetivo da Demo

Criar um cluster local (Kind) e aplicar um conjunto de workloads desenhados para evidenciar:

| Conceito | Onde aparece | O que observar |
| -------- | ------------ | -------------- |
| PriorityClass & Preemption | `01-priorityclasses.yaml`, pods `critical` vs `lowprio` | Pods de menor prioridade podem ser desalojados (preempted). |
| Taints & Tolerations | Script `02-node-labels-and-taints.sh` + distribuição de pods | Certos workloads só agendam em nós marcados. |
| QoS Classes (Guaranteed/Burstable/BestEffort) | Manifests `14-besteffort-worker.yaml`, `15-burstable-worker.yaml` | Efeito de limites e ausência deles. |
| Overprovision / Filler pods | `13-lowprio-filler.yaml` | Preencher capacidade para provocar preempção depois. |
| Affinity / Anti-Affinity | `20-affinity-demo.yaml` | Controle de co-localização ou dispersão de pods. |

---

## 🧱 Arquitetura Lógica

1. **Cluster Kind** com nós homogêneos (podemos etiquetar e taintar manualmente após criação).
2. **Imagem de demonstração** (Rust) buildada localmente e carregada para o cluster (sem registry externo para reduzir atrito).
3. **Recursos de controle de scheduling**: PriorityClasses, node labels, taints, affinities.
4. **Workloads contrastantes**: estáveis, canários ruidosos, críticos, best‑effort, burstable e fillers.

---

## ✅ Pré‑requisitos

| Ferramenta | Por que é necessária | Verificação sugerida |
| ---------- | -------------------- | -------------------- |
| Docker / Docker Desktop | Executar containers e o cluster Kind (Kind roda dentro de Docker). | `docker info` |
| kubectl | Aplicar e inspecionar manifests. | `kubectl version --client` |
| kind | Criar cluster local reproduzível. | `kind --version` |
| (Opcional) jq | Melhor formatação de saídas JSON. | `jq --version` |

### Instalação Rápida

### Windows (PowerShell / Chocolatey)

```powershell
choco install docker-desktop kubernetes-cli kind -y
```

### macOS (Homebrew)

```bash
brew install --cask docker
brew install kubectl kind
```

### Linux (Ubuntu/Debian-like)

Use os scripts oficiais de instalação ou pacotes (Docker, kubectl, kind). Ex.: <https://kind.sigs.k8s.io/>

---

## 🔍 Passo 0 — Validar Ambiente

Antes de iniciar, confirme que tudo responde:

```bash
docker info | grep -i server
kubectl version --client
kind --version
```

Se algum falhar, ajuste ANTES de prosseguir (reduz ruído de troubleshooting later).

---

## 🏗️ Passo 1 — Criar o Cluster Kind

Arquivo de script: `kind/create-cluster.sh`

```bash
cd kind
./create-cluster.sh aula02
```

O script:
 
1. Cria o cluster chamado `aula02`.
2. Configura context automaticamente (kubeconfig).


### Verificações

```bash
kubectl config current-context         # Deve apontar para kind-aula02
kubectl get nodes -o wide              # Lista nós Ready
```

> Se algum nó estiver `NotReady`, espere alguns segundos ou execute `kubectl describe node <nome>` para eventos.

**Por que primeiro o cluster?** Sem o cluster não podemos aplicar PriorityClasses (objeto de API) nem taints; tudo depende de ter o plano de controle funcional.

---

## 🧪 Passo 2 — Construir a Imagem de Demonstração

Entrar na pasta de código Rust e gerar a imagem utilizada pelos workloads.

```bash
cd ../rust
docker build -t aula02-scheduler-demo:demo .
```

### O que está acontecendo

| Ação | Justificativa |
| ---- | ------------- |
| Build local | Evita depender de registry remoto (mais rápido em aula). |
| Tag simples `:demo` | Facilita reaproveitar sem versionamento complexo nesta fase. |

### Verificação

```bash
docker images | grep aula02-scheduler-demo
```

---

## 🚚 Passo 3 — Carregar a Imagem no Cluster Kind

Kind não “vê” automaticamente imagens locais fora dos seus nós; precisamos carregá-las:

```bash
kind load docker-image aula02-scheduler-demo:demo --name aula02
```

### Verificação rápida

```bash
kubectl get nodes -o name | head -1 | xargs -I{} kubectl debug {} --image=busybox -- cat /etc/hosts 2>/dev/null | true
# (Opcional) Apenas se quiser testar pulling — geralmente não é necessário.
```

**Por que não usar registry?** Para uma aula rápida reduz passos (criação de registry local, push/pull). Se quiser escalar, adicione um registry compartilhado.

---

## 🏷️ Passo 4 — Aplicar Objetos Fundamentais de Scheduling

Entrar na pasta `k8s`:

```bash
cd ../k8s
kubectl apply -f 00-namespace.yaml -f 01-priorityclasses.yaml
```

### Justificativas

| Recurso | Papel no cenário |
| ------- | ---------------- |
| Namespace `aula02` | Isolar recursos da demo (facilita limpeza). |
| PriorityClasses | Criam estrato de prioridades para observar preempção. |

### Script de Labels/Taints

```bash
./02-node-labels-and-taints.sh
```

Esse script:
 
1. Adiciona labels para permitir demonstrar `nodeAffinity` / `nodeSelector`.
2. Aplica taints em um ou mais nós para restringir agendamento.


### Verificar Resultado

```bash
kubectl get priorityclass
kubectl get nodes --show-labels
kubectl describe node <um-no> | grep -i taints -A1
```

> Se as PriorityClasses não aparecerem, confira a versão do cluster (Kind moderno suporta o objeto stable).

---

## 📦 Passo 5 — Aplicar Workloads Demonstrativos

Aplicar todos os manifests (ordem não é crítica após a criação das PriorityClasses):

```bash
kubectl apply -f 10-stable-app-deploy.yaml \
               -f 11-noisy-canary.yaml \
               -f 13-lowprio-filler.yaml \
               -f 12-critical-api.yaml \
               -f 14-besteffort-worker.yaml \
               -f 15-burstable-worker.yaml \
               -f 20-affinity-demo.yaml
```

### O que observar

| Manifesto | Enfatiza | Dica de inspeção |
| --------- | -------- | ---------------- |
| `10-stable-app-deploy` | Base/serviço estável | `kubectl get pods -n aula02 -l app=stable` |
| `11-noisy-canary` | Workload ruidoso (pode competir por CPU) | Top de uso / prioridade menor |
| `13-lowprio-filler` | Preencher cluster para pressionar scheduler | Ver preempção após entrada de crítico |
| `12-critical-api` | Maior prioridade (pode desalojar) | Eventos: `kubectl describe pod` |
| `14-besteffort-worker` | QoS BestEffort (sem requests/limits) | `kubectl get pods -o custom-columns=NAME:.metadata.name,QOS:.status.qosClass` |
| `15-burstable-worker` | QoS Burstable | Comparar com BestEffort/Guaranteed |
| `20-affinity-demo` | Afinidade por label de nó | `kubectl describe pod \| grep -i affinity -A8` |

### Verificação geral

```bash
kubectl -n aula02 get pods -o wide
kubectl -n aula02 get pods --sort-by=.status.qosClass
```

> Pode haver pods em `Pending` inicialmente — isso é útil para explicar restrições (taints, afinidade, falta de recursos).

---

## 🔄 Passo 6 — Demonstrar Preempção e Ajustes

1. Garanta que os pods de filler e workloads de baixa prioridade estejam ocupando nós.
2. Reaplique (ou escale) um deployment de alta prioridade para forçar desalojamento:

```bash
kubectl -n aula02 scale deploy critical-api --replicas=3
```

 
1. Observe eventos de preempção:

```bash
kubectl -n aula02 describe pod <pod-novo-ou-evicted> | grep -i preempt -C3 || true
kubectl -n aula02 get events --sort-by=.lastTimestamp | tail -20
```

1. Consultar QoS e distribuição:

```bash
kubectl -n aula02 get pods -o custom-columns=NAME:.metadata.name,PRIO:.spec.priorityClassName,QOS:.status.qosClass,NODE:.spec.nodeName
```

> Explique como o scheduler considera: (1) filtros (taints/tolerations, nodeSelector), (2) scoring (afinidade), (3) preempção se não houver encaixe.

---

## 🧭 Passo 7 — Explorando Afinidade

Edite (se desejar) `20-affinity-demo.yaml` para trocar labels de afinidade e reaplique:

```bash
kubectl apply -f 20-affinity-demo.yaml
kubectl -n aula02 describe pod $(kubectl -n aula02 get pods -l app=affinity-demo -o name | head -1)
```

Mostre como alterar `requiredDuringSchedulingIgnoredDuringExecution` para `preferredDuringScheduling...` muda o comportamento (preferência vs obrigação).

---

## 🧹 Passo 8 — Limpeza

```bash
kind delete cluster --name aula02
```

Se quiser apenas remover a namespace durante a aula e recriar:

```bash
kubectl delete ns aula02
```

---

## 🛠️ Troubleshooting Rápido

| Sintoma | Possível Causa | Ação Sugerida |
| ------- | -------------- | ------------- |
| Pods `ImagePullBackOff` | Imagem não carregada no Kind | Repetir `kind load docker-image ...` |
| Pods `Pending` | Taint sem toleration | `kubectl describe pod` e revisar tolerations |
| Preempção não ocorre | Pouca pressão de recursos | Escale fillers ou reduza requests dos críticos |
| Afinidade ignorada | Usou preferred (não required) | Verificar spec e labels dos nós |
| QoS inesperado | Requests/limits inconsistentes | Conferir spec: ambos = Guaranteed; só limits => Burstable |

### Comandos Úteis de Diagnóstico

```bash
kubectl -n aula02 describe pod <nome>
kubectl top pods -n aula02            # Se metrics-server estiver instalado
kubectl get events -n aula02 --sort-by=.lastTimestamp | tail -30
```

---

## ⚡ TL;DR (Execução Rápida)

```bash
cd kind && ./create-cluster.sh aula02
cd ../rust && docker build -t aula02-scheduler-demo:demo .
kind load docker-image aula02-scheduler-demo:demo --name aula02
cd ../k8s
kubectl apply -f 00-namespace.yaml -f 01-priorityclasses.yaml && ./02-node-labels-and-taints.sh
kubectl apply -f 10-stable-app-deploy.yaml -f 11-noisy-canary.yaml -f 13-lowprio-filler.yaml -f 12-critical-api.yaml -f 14-besteffort-worker.yaml -f 15-burstable-worker.yaml -f 20-affinity-demo.yaml
kubectl -n aula02 get pods -o wide
```

---

## 🚀 Próximas Extensões (Opcional)

| Ideia | Valor Didático |
| ----- | -------------- |
| Instalar `metrics-server` | Visualizar consumo real e reforçar impacto de requests/limits |
| Adicionar `PodDisruptionBudget` | Explicar manutenção + disponibilidade |
| Simular pressão de CPU | Mostrar scheduler + throttling cgroups |
| Usar `kubectl cordon/drain` | Demonstrar evacuação controlada |

---

Se quiser refinar ainda mais (ex.: adicionar diagramas ou scripts de medição), basta pedir!
