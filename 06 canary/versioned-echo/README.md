# versioned-echo (Rust)

Pequeno serviço HTTP que expõe:
- `/` -> texto com a versão
- `/version` -> JSON com `{ "version": "vX" }`
- `/health` -> healthcheck simples
- `/metrics` -> métricas Prometheus

Use `APP_VERSION=v1` ou `APP_VERSION=v2` no container para distinguir versões.
