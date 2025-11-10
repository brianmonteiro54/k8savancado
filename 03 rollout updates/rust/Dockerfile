## Multi-stage build for myapp (Rust) targeting Debian runtime
## Build context: this 'rust' workspace directory

ARG RUST_VERSION=1.75
FROM rust:${RUST_VERSION}-bookworm AS builder
WORKDIR /app

# Copia manifests primeiro para aproveitar cache de dependências
COPY Cargo.toml Cargo.lock ./
COPY myapp/Cargo.toml myapp/Cargo.toml
COPY simulator/Cargo.toml simulator/Cargo.toml

# Garante que o cache de crates seja baixado sem precisar do código completo
RUN mkdir -p myapp/src simulator/src \
 && echo 'fn main() {}' > myapp/src/main.rs \
 && echo 'fn main() {}' > simulator/src/main.rs \
 && cargo fetch

# Agora copia o código real, garante alvo GNU e compila apenas o pacote myapp
RUN rm -rf myapp/src simulator/src
COPY myapp/src myapp/src
COPY simulator/src simulator/src
RUN rustup target add x86_64-unknown-linux-gnu \
 && cargo build --release --locked --target x86_64-unknown-linux-gnu --package myapp

FROM debian:bookworm-slim AS runtime
ARG APP_VERSION=0.0.0
ENV APP_VERSION=${APP_VERSION} \
    PORT=8080 \
    RUST_BACKTRACE=1
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/* \
    && useradd -m app
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/myapp /usr/local/bin/myapp
RUN chmod 0755 /usr/local/bin/myapp
USER app
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/myapp"]
