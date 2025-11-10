# ---- Build stage ----
FROM rust:1.79-bullseye AS builder
WORKDIR /app
# Create a dummy project to cache dependencies
RUN USER=root cargo new --bin app-health-demo
WORKDIR /app/app-health-demo
COPY Cargo.toml Cargo.toml
RUN cargo build --release || true
# Now copy source
COPY src ./src
# Build
RUN cargo build --release

# ---- Runtime stage ----
FROM debian:bookworm-slim
ENV RUST_LOG=info
WORKDIR /app
COPY --from=builder /app/app-health-demo/target/release/app-health-demo /usr/local/bin/app-health-demo
EXPOSE 8080
# Non-root user
RUN useradd -m runner
USER runner
CMD ["/usr/local/bin/app-health-demo"]
