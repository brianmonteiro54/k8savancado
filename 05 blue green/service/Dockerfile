# syntax=docker/dockerfile:1

FROM rust:1.80 as builder
WORKDIR /app
COPY Cargo.toml Cargo.toml
RUN mkdir src && echo 'fn main(){}' > src/main.rs && cargo build --release || true
COPY src src
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN useradd -u 10001 -r -s /sbin/nologin appuser
COPY --from=builder /app/target/release/myapp-service /usr/local/bin/myapp-service
ENV PORT=8080
USER appuser
EXPOSE 8080
CMD ["/usr/local/bin/myapp-service"]
