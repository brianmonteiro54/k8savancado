# Worker Dockerfile
FROM rust:1.83 AS builder
WORKDIR /app

# Pre-fetch dependencies without copying full sources
COPY Cargo.toml ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release && rm -rf src

# Build with real sources
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
	&& apt-get install -y --no-install-recommends ca-certificates \
	&& rm -rf /var/lib/apt/lists/*
WORKDIR /app
ENV RUST_LOG=info
COPY --from=builder /app/target/release/worker /usr/local/bin/worker
EXPOSE 8080
ENTRYPOINT ["worker"]
