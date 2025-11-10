# Kubernetes Avançado — Aula 02 (Agendamento & Nós)

Este pacote entrega **código em Rust**, **YAMLs** e **roteiros** para demonstrar:
- **Taints & Tolerations**
- **Node/Pod Affinity & Anti-Affinity**
- **Topology Spread Constraints**
- **Priority & Preemption**
- **QoS (BestEffort/Burstable) e Evictions**

## Passo-a-passo rápido
```bash
cd kind && ./create-cluster.sh aula02
cd ../rust && docker build -t aula02-scheduler-demo:demo .
kind load docker-image aula02-scheduler-demo:demo --name aula02
cd ../k8s && kubectl apply -f 00-namespace.yaml -f 01-priorityclasses.yaml && ./02-node-labels-and-taints.sh
kubectl apply -f 10-stable-app-deploy.yaml -f 11-noisy-canary.yaml -f 13-lowprio-filler.yaml -f 12-critical-api.yaml -f 14-besteffort-worker.yaml -f 15-burstable-worker.yaml -f 20-affinity-demo.yaml
```

## Preparação do ambiente

- Instale e mantenha ativos Docker, `kubectl` e `kind`. Esses binários garantem que você consiga construir a imagem local e orquestrar um cluster Kubernetes dentro do Docker. O arquivo `docs/INSTALL.md` traz instruções específicas para Windows (via `choco`), macOS (via `brew`) e Linux.
- Abra um PowerShell apontando para `\aulas-k8s-avancado\advanced-scheduler`. Trabalhar a partir da raiz do projeto evita caminhos quebrados nos scripts.
- Rode `kubectl config get-contexts` para confirmar que o contexto atual não é um cluster de produção. O laboratório cria e deleta clusters; fazer essa conferência previne acidentes.

## Criar o cluster Kind

1. Execute `Set-Location kind` para entrar na pasta onde ficam os manifestos do Kind.
2. Lance `./create-cluster.sh aula02`. O script invoca `kind create cluster` usando `kind/kind-cluster.yaml`, que define quantidade de nós, roles e mapeamento de portas. Essa topologia múltipla é essencial para testar distribuição de pods e preempção.
3. Valide o resultado com `kubectl get nodes -o wide`. Anote os nomes dos trabalhadores: o script de labels/taints depende dessa listagem para aplicar personalizações consistentes.

## Build da aplicação de carga

1. `Set-Location ../rust` coloca você na pasta do serviço de suporte. Esse serviço expõe endpoints que simulam diferentes padrões de consumo (CPU, memória) para provocar o scheduler.
2. Compile a imagem com `docker build -t aula02-scheduler-demo:demo .`. O Dockerfile empacota a aplicação Rust e produz uma imagem local que não existe em registries públicos, então o build local é obrigatório.
3. Retorne à raiz (`Set-Location ..`) e carregue a imagem no cluster usando `kind load docker-image aula02-scheduler-demo:demo --name aula02`. Sem esse passo o cluster Kind não conhece a imagem, e os pods ficariam presos em `ImagePullBackOff`.

## Recursos base no cluster

1. Entre na pasta dos manifestos Kubernetes: `Set-Location k8s`.
2. Aplique `kubectl apply -f 00-namespace.yaml -f 01-priorityclasses.yaml` para criar o namespace isolado `aula02`, uma quota simples e as PriorityClasses que serão usadas na etapa de preempção.
3. Execute `./02-node-labels-and-taints.sh`. O script etiqueta cada worker com zona (`topology.kubernetes.io/zone`) para suportar topology spread, adiciona `disktype=ssd` a um nó (para a demo de affinity) e insere o taint `workload=noisy:NoSchedule` em outro nó (para testar tolerations). Confirme com `kubectl get nodes --show-labels`.
4. Abra um novo terminal e rode `kubectl -n aula02 get pods -o wide -w`. Esse watch contínuo permite observar, em tempo real, como o scheduler posiciona ou reprovisiona pods conforme as regras aplicadas.

## Demonstrações do scheduler

### Distribuição e resiliência (`10-stable-app-deploy.yaml`)

- Aplique `kubectl apply -f 10-stable-app-deploy.yaml`. O deployment cria seis réplicas com `topologySpreadConstraints` exigindo equilíbrio entre nós (`kubernetes.io/hostname`). Isso demonstra como o scheduler distribui pods para evitar hotspots.
- Em seguida, teste `kubectl scale deploy stable-app -n aula02 --replicas=9`. O objetivo é verificar que mesmo com novas réplicas o skew máximo continua respeitado; se algum nó não comportar, os pods ficam pendentes até que uma redistribuição seja possível.

### Taints & tolerations (`11-noisy-canary.yaml`)

- `kubectl apply -f 11-noisy-canary.yaml` agenda pods que precisam tolerar o taint `workload=noisy`. A combinação de `tolerations` com `nodeSelector` garante que apenas workloads barulhentos vão para o nó isolado.
- Use `kubectl describe pod noisy-canary-... -n aula02` para inspecionar os eventos. O `postStart` dispara o endpoint `/busy`, simulando ruído real e mostrando por que manter esse workload segregado é importante.

### Preempção com PriorityClasses (`13-lowprio-filler.yaml` + `12-critical-api.yaml`)

- Aplique `kubectl apply -f 13-lowprio-filler.yaml` para saturar o cluster com cargas de baixa prioridade, cada uma solicitando uma fatia generosa de CPU e memória. Isso cria um cenário de contensão intencional.
- Em seguida, `kubectl apply -f 12-critical-api.yaml` cria um pod crítico usando PriorityClass alta. Se não houver recursos disponíveis, o scheduler preempta pods de menor prioridade. Verifique os eventos `Preempted` para entender quais pods foram desalojados e por quê.

### QoS e limites (`14-besteffort-worker.yaml` e `15-burstable-worker.yaml`)

- `kubectl apply -f 14-besteffort-worker.yaml` cria pods sem requests/limits, recebendo a classe QoS `BestEffort`. Já `kubectl apply -f 15-burstable-worker.yaml` define requests baixos e limites mais altos, resultando em QoS `Burstable`.
- Gere carga com `kubectl -n aula02 exec deploy/burstable-worker -- curl -s 'http://localhost:8080/busy?ms=60000&threads=4'`. Com metrics-server habilitado, compare `kubectl top pod` e observe que, em cenários de pressão, Kubernetes sempre sacrifica pod BestEffort primeiro, preservando workloads com garantias explícitas.

### Afinidades e anti-afinidades (`20-affinity-demo.yaml`)

- `kubectl apply -f 20-affinity-demo.yaml` provisiona dois deployments complementares. O `store` possui `nodeAffinity` rígido exigindo `disktype=ssd`, enquanto `client` prefere estar co-localizado com `store` (podAffinity) e evita compartilhar nó com outros `client` (podAntiAffinity).
- Valide o comportamento com `kubectl get pods -n aula02 -l tier=data -o wide` e `kubectl describe pod client-...`. Esses comandos mostram como o scheduler equilibra preferências suaves (preferred) versus requisitos rígidos (required).

## Observabilidade durante os testes

- Descubra a topologia exata de cada pod executando `kubectl -n aula02 exec deploy/stable-app -- curl -s localhost:8080/info`. O endpoint retorna `pod_name` e `node_name`, facilitando o entendimento dos movimentos do scheduler.
- Simule pressão de memória com `kubectl -n aula02 exec deploy/stable-app -- curl -s -X POST 'http://localhost:8080/alloc?mb=128&chunks=2'`. Em seguida, chame `/free` para liberar. Essa dinâmica ajuda a observar como QoS afeta decisões de eviction.
- Para testar reprogramações, remova temporariamente o taint com `kubectl taint nodes <node> workload=noisy:NoSchedule-` e observe o comportamento dos pods. Reaplique a restrição ao final (`kubectl taint nodes <node> workload=noisy:NoSchedule`).

## Limpeza

- `kubectl delete namespace aula02` remove todos os recursos criados para a aula, permitindo repetir o laboratório em estado limpo.
- `kind delete cluster --name aula02` encerra o cluster descartável, liberando CPU, memória e portas locais.

Mais detalhes e o roteiro completo de apresentação estão em `docs/INSTALL.md` e `docs/roteiro_apresentacao.md`.
