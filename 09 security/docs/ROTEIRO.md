# ROTEIRO DA APRESENTAÇÃO — Aula 09 (Segurança no Cluster)

> Tempo total sugerido: 75–90 min (com labs).

## 0) Abertura (5 min)
- Contexto: superfícies de ataque em clusters modernos; princípio do menor privilégio.
- Objetivo do dia: sair com **RBAC mínimo funcionando** e **TLS automatizado** na prática.

## 1) Identidade de Workloads + RBAC (25 min)
- Conceito: **ServiceAccount ≅ identidade do pod**.
- Evitar `default`, criar **SA por aplicação**.
- **DEMO** (CLI Rust ou YAML):
  1. Criar `namespace analytics` + `ServiceAccount analytics-sa`.
  2. Criar `Role` mínima (pods: get,list; configmaps: get) e `RoleBinding`.
  3. Subir `Pod` usando `serviceAccountName: analytics-sa`.
  4. Validar com `kubectl auth can-i` (permitir leitura, negar criação).
- Mensagem-chave: **permissão mínima** por **namespace** e **por SA**.

## 2) TLS automatizado com cert-manager (25 min)
- Dor real: certificados manuais expiram e derrubam serviços.
- **DEMO**:
  1. Instalar cert-manager (Helm).
  2. Aplicar `Issuer` self-signed → gerar **Root CA** (`root-ca-secret`).
  3. Criar `Issuer` de CA e emitir `Certificate` para `service-a` (`service-a-tls`).
  4. Subir `service-a` (Rust, HTTPS) montando o Secret.
  5. Validar com `curl --cacert` dentro do cluster.
- Mensagem-chave: **automação de PKI** reduz falhas operacionais.

## 3) Integração com identidades de cloud (10 min)
- Mostrar a **anotação** da SA (IRSA/Workload Identity/Azure WI).
- Explicar conceito de **credenciais federadas e de curta duração**.
- *Sem hardcode de chaves* nos pods.

## 4) “Saiba Mais” (10–15 min)
- RBAC vs ABAC (motivações, auditabilidade).
- Política como código (OPA/Rego) — onde encaixa no admission.
- Segurança de supply chain: SBOM, assinatura de imagens, proveniência.

## 5) Encerramento (5 min)
- Recap do que foi feito.
- Próximos passos recomendados: NetworkPolicies, Pod Security, assinatura (Cosign).

---

## Perguntas de checagem rápidas
- O que acontece se o pod usar a `default` SA? Por que não é recomendável?
- Como verifico rapidamente se uma SA pode criar Pods no namespace?
- Onde o cert-manager grava chaves/certs? Como renova?
- Como federar a SA com uma identidade no seu provedor de cloud?

---

## Resultados esperados no final
- `kubectl auth can-i ...` **OK** para leitura e **DENY** para criação.
- `curl --cacert ... https://service-a:8443/healthz` → `{"status":"ok"}`.
- `Certificate` com `Ready=True` e `Secret` montado no pod.
