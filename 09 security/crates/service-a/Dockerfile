# Build
FROM rust:1.81 as build
WORKDIR /app
# Cache deps
COPY Cargo.toml ./Cargo.toml
COPY crates/service-a/Cargo.toml ./crates/service-a/Cargo.toml
COPY crates/orchestrator/Cargo.toml ./crates/orchestrator/Cargo.toml
RUN mkdir -p crates/service-a/src crates/orchestrator/src &&     echo "fn main(){}" > crates/service-a/src/main.rs &&     echo "fn main(){}" > crates/orchestrator/src/main.rs &&     cargo build -p service-a --release

# Real source
COPY . .
RUN cargo build -p service-a --release

# Runtime (distroless-ish)
FROM gcr.io/distroless/cc-debian12
WORKDIR /
USER 65532:65532
COPY --from=build /app/target/release/service-a /usr/local/bin/service-a
EXPOSE 8443
ENV TLS_CERT_PATH=/tls/tls.crt
ENV TLS_KEY_PATH=/tls/tls.key
ENTRYPOINT ["/usr/local/bin/service-a"]
