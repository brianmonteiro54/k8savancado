# App Health Demo (Rust + Axum) for Kubernetes

A minimal, cloud-agnostic demo that turns the theory from **“Saúde da Aplicação e Gerenciamento de Recursos – Teoria e Fundamentos”** into practice.

It shows, end‑to‑end:
- Golden Signals instrumentation (`/metrics` Prometheus endpoint).
- Liveness, Readiness and Startup probes (`/health/*`).
- Failure modes (CPU throttling, memory OOM/leak, latency and error injection).
- Resource **Requests/Limits** and **QoS** tiers (Burstable vs Guaranteed) via Kustomize overlays.
- Horizontal Pod Autoscaler (HPA) driven by CPU utilization.
- Graceful shutdown and disruption protection (PDB).

> All steps are Kubernetes‑native and **vendor‑agnostic**. You can use Kind, Minikube, k3d, AKS, EKS, GKE… you name it.

---

## 1) Prerequisites

- Docker or another OCI image builder (e.g., `podman`, `nerdctl`).
- `kubectl` (v1.24+), `kustomize` (optional, but recommended).
- A cluster: **Kind** or **Minikube** are simplest for local tests.
- Optional: `hey` or `vegeta` to generate load.

## 2) Run locally (no Kubernetes)

```bash
# in the project root
cargo run

# Try it
curl -s localhost:8080/info | jq
curl -s localhost:8080/health/startup -i
curl -s localhost:8080/health/ready   -i
curl -s localhost:8080/health/live    -i
curl -s localhost:8080/metrics | head -n 20
```

Environment knobs:
```bash
export STARTUP_DELAY_MS=3000   # How long until startupProbe passes
export READY_AFTER_MS=5000     # When readiness flips to true after startup
export PORT=8080               # Listener port
```

## 3) Container image

```bash
# Build
docker build -t app-health-demo:local .

# (Optional) Push to your registry
# docker tag app-health-demo:local ghcr.io/<user>/app-health-demo:0.1.0
# docker push ghcr.io/<user>/app-health-demo:0.1.0
```

## 4) Kubernetes deploy (Kustomize)

Base manifests are under `k8s/base`, with two overlays:

- **burstable**: lower requests than limits — more efficient but can throttle.
- **guaranteed**: requests == limits — more predictable latency; avoids evictions under memory pressure.

### Apply Burstable

```bash
kubectl apply -k k8s/overlays/burstable
kubectl -n app-health-demo get pods -w
kubectl -n app-health-demo port-forward svc/app-health-demo 8080:8080
```

### Switch to Guaranteed

```bash
kubectl apply -k k8s/overlays/guaranteed
```

### HPA & PDB

```bash
kubectl -n app-health-demo get hpa
kubectl -n app-health-demo get pdb
```

## 5) Probes in action

- `startupProbe`: waits `STARTUP_DELAY_MS` and then flips `started=true`.
- `readinessProbe`: only returns 200 **after** `started=true` **and** a subsequent delay (`READY_AFTER_MS`). You may also flip it manually:

```bash
curl "http://localhost:8080/admin/flip_ready?value=false"
curl "http://localhost:8080/admin/flip_ready?value=true"
```

- `livenessProbe`: set/unset via `admin/flip_live`:

```bash
curl "http://localhost:8080/admin/flip_live?value=false"
```

## 6) Failure modes (connect to the Service or port-forward first)

```bash
# Latency injection (Golden Signals: latency)
curl "/simulate/latency?ms=800"

# Error injection (Golden Signals: errors)
curl -i "/simulate/error?rate=0.4"

# CPU saturation (watch HPA react under CPU metrics)
curl "/simulate/cpu?seconds=25"

# Memory pressure / OOM (leak true by default)
curl "/simulate/mem?mb=256&leak=true"
```

Then observe Prometheus metrics at `/metrics` and watch your pod’s restarts (`kubectl get pods`).

## 7) Clean up

```bash
kubectl delete ns app-health-demo
```

## Notes

- **Outlier ejection** (from the theory) is typically enforced by a service mesh (e.g., Istio/Linkerd) or advanced gateways, not by core Kubernetes Service. This demo keeps the cluster **agnostic** but you can layer a mesh on top and point it to `/unstable` to see ejection at work.
- The app ships with **graceful shutdown**. K8s will send SIGTERM → readiness=false → in-flight drain before pod deletion. The PDB minimizes concurrent evictions.
