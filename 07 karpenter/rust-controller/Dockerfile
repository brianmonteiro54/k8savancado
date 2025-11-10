# Build stage
FROM rust:1.78 as builder
WORKDIR /app
COPY Cargo.toml .
RUN mkdir -p src && echo 'fn main(){}' > src/main.rs
RUN cargo build --release
COPY src ./src
RUN touch -a -m src/main.rs && cargo build --release

# Runtime (debian slim para simplicidade)
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/mini-karpenter-rs /usr/local/bin/mini-karpenter-rs
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/mini-karpenter-rs"]
