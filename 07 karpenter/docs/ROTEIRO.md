# ROTEIRO — Aula 07 (Karpenter, Escala de Nós) — 90 minutos

**Meta:** Mostrar, na prática, como o Karpenter decide **provisionar** e **consolidar** nós, com um mini‑controlador em **Rust** + **YAML declarativo**.

## Bloco 1 — Abertura & Contexto (15’)
- Dor do autoscaling tradicional: grupos fixos, lentidão, desperdício.
- Ideia central do Karpenter: **capacidade sob medida** (*just‑in‑time*), **rápida** e **barata** (Spot quando possível).
- Enquadrar como **bin packing multidimensional** + decisões **gulosas** bem informadas.

## Bloco 2 — Demo Parte 1: Pods Pending ⇒ Plano (25’)
1. Mostrar `pending-deploy.yaml` — por que ficará Pending (nodeSelector “impossível”).
2. Rodar controlador **localmente** (`cargo run --release`). Ver logs aparecendo.
3. Abrir o **ConfigMap `plan-*`** e explicar o `plan.yaml`:
   - `requirements` (zona, capacity-type, GPU/ARM etc. — adaptáveis).
   - Estratégia **least‑waste** (nó menor que comporta).
   - Como integrar com **Karpenter** real (aplicar o `plan.yaml`).

## Bloco 3 — Demo Parte 2: Consolidação (20’)
1. Aplicar `consolidation-workload.yaml` (várias cargas pequenas).
2. Acompanhar **ConfigMaps `consolidation-*`**:
   - Como calculamos utilização por *requests*.
   - Lista de Pods candidatos, PDB/Spread e passos para **drain + scale‑down**.

## Bloco 4 — Código (20’)
- `kube-rs`: Client, ListParams, field selectors.
- Parsers de CPU/Mem (millicores, Ki/Mi/Gi).
- Catálogo de instâncias (tamanhos) e heurística **least‑waste**.
- Escrita de ConfigMaps (**declaratividade** → GitOps).

## Bloco 5 — Encerramento (10’)
- Checklist de produção: PDB, Topology Spread, Prioridades, FinOps, GitOps.
- Multicloud & NodePools (governança).
- Próximos passos: ligar no Karpenter real; integrar custos; metrics‑server; Spot preemption.

**Q&A** (tempo remanescente)
