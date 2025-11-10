# ---- Build stage ----
FROM rust:1.81-slim AS builder
WORKDIR /app
# Cache deps
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock* ./
RUN mkdir -p src && echo 'fn main() {println!("building...")}' > src/main.rs
RUN cargo build --release || true
# real sources
COPY src ./src
RUN cargo build --release

# ---- Runtime stage ----
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/myrustapp /usr/local/bin/myrustapp
ENV RUST_LOG=info     PORT=8080
EXPOSE 8080
CMD ["myrustapp"]
