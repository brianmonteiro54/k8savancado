# k8s/README — Blue/Green Manifestos (passo‑a‑passo)

## 1) Namespace
```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: aula05
```
Cria uma **fronteira lógica** para isolar os recursos da aula.

## 2) Deployments (Blue e Green)
Ambos referenciam **a mesma imagem** (`myapp:latest`), mudando apenas:
- **labels** (`env: blue|green`)
- **env vars** (`VERSION` e `COLOR`) — retornadas pelo serviço para identificação.

Pontos‑chave:
- `readinessProbe` e `livenessProbe` garantem **cutover seguro** (só entra tráfego quando pronto).
- `resources` com *requests/limits* evitam ruídos por falta de CPU/Mem.

## 3) Service
```yaml
spec:
  selector:
    app: myapp
    env: blue   # muda para 'green' no cutover
```
O **Service** é o endereço estável. **Trocar o tráfego** = **alterar o `selector`**.

### Cutover por YAML (patch declarativo)
```bash
kubectl -n aula05 patch svc myapp -p '{"spec":{"selector":{"app":"myapp","env":"green"}}}'
# rollback
kubectl -n aula05 patch svc myapp -p '{"spec":{"selector":{"app":"myapp","env":"blue"}}}'
```

> Em ambientes GitOps, essa alteração vai como **commit/PR** — audível, idempotente e reversível.
