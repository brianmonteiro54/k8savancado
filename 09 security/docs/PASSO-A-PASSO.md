# PASSO‑A‑PASSO (agnóstico) — Aula 09

Este guia explica **exatamente** como executar a demonstração, sem depender de um provedor específico.

## 0) Ambiente

- Kubernetes habilitado no **Docker Desktop** (contexto `docker-desktop`).
- `kubectl`, `helm`, `docker`, `rustup` com toolchain estável.

> Se você optar por outro cluster local/remoto (Minikube, k3d, AKS, EKS, GKE), lembre-se de publicar as imagens em um registry acessível ou usar o script `scripts/kind-load-image.sh` para clusters que não compartilham o daemon Docker do host.

## 1) Instalar o cert-manager

Instale usando Helm (forma recomendada, traz CRDs e controladores):

```bash
helm repo add jetstack https://charts.jetstack.io
helm repo update
helm install cert-manager jetstack/cert-manager   --namespace cert-manager --create-namespace   --set crds.enabled=true
```

> Verifique:

```bash
kubectl -n cert-manager get pods
```

## 2) Criar **CA raiz** e emitir certificado para `service-a`

Aplique os manifests:

```bash
kubectl apply -f k8s-manifests/cert-manager/issuers-and-certs.yaml
```

- Isso cria um `Issuer` self-signed, um `Certificate` CA (`root-ca-secret`), um `Issuer` de CA e um `Certificate` para `service-a` (`service-a-tls`).

Valide o `Certificate` (aguarde `Ready=True`):

```bash
kubectl -n default get certificate service-a-cert -o wide
kubectl -n default get secret service-a-tls
```

## 3) Subir o serviço HTTPS em Rust

### 3.1 Build da imagem

Dentro do diretório raiz:

```bash
docker build -t service-a:local -f crates/service-a/Dockerfile .
```

### 3.2 Disponibilizar a imagem para o cluster

No Docker Desktop não é necessário nenhum passo extra: o cluster já enxerga as imagens locais. Em outros ambientes, utilize `./scripts/kind-load-image.sh service-a:local` ou publique a imagem em um registry compartilhado.

### 3.3 Deployment + Service

```bash
kubectl apply -f k8s-manifests/service-a-deployment.yaml
kubectl apply -f k8s-manifests/service-a-service.yaml
kubectl -n default get pods,svc -l app=service-a
```

## 4) Validar o TLS de dentro do cluster

Crie um `ConfigMap` com a CA raiz para o `curl` usá-la como confiança:

```bash
./scripts/create-ca-configmap.sh
kubectl -n default run tls-tester --image=alpine:3.19 -it --rm --   sh -lc "apk add --no-cache curl && curl --cacert /ca/ca.crt https://service-a.default.svc.cluster.local:8443/healthz"
```

Você deve ver `{"status":"ok"}`.

## 5) Identidade do workload e **RBAC**

### 5.1 Via CLI Rust (mesmo efeito dos YAMLs)

```bash
cargo run -p orchestrator -- bootstrap
```

### 5.2 Somente YAML

```bash
kubectl apply -f k8s-manifests/analytics/namespace.yaml
kubectl apply -f k8s-manifests/analytics/rbac.yaml
kubectl apply -f k8s-manifests/analytics/pod.yaml
```

### 5.3 Testes práticos de permissão

```bash
# Permite ler pods no namespace analytics
kubectl auth can-i --as=system:serviceaccount:analytics:analytics-sa get pods -n analytics

# Deve negar criação de pods
kubectl auth can-i --as=system:serviceaccount:analytics:analytics-sa create pods -n analytics
```

## 6) (Opcional) Federar a SA com identidade de cloud

> **Escolha apenas UMA** conforme seu provedor. O comando só adiciona a anotação na SA `analytics-sa`.

- **EKS/IRSA**

```bash
cargo run -p orchestrator -- annotate --provider eks --value arn:aws:iam::123456789012:role/S3Reader
```

- **GKE/Workload Identity**

```bash
cargo run -p orchestrator -- annotate --provider gke --value meu-servico@projeto.iam.gserviceaccount.com
```

- **AKS/Azure Workload Identity**

```bash
cargo run -p orchestrator -- annotate --provider aks --value <AZURE_CLIENT_ID_DA_MANAGED_IDENTITY>
```

## 7) Limpeza

```bash
cargo run -p orchestrator -- cleanup
kubectl delete -f k8s-manifests --ignore-not-found
helm uninstall cert-manager -n cert-manager || true
```
