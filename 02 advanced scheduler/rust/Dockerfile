FROM rust:1.81 as builder
WORKDIR /app

COPY Cargo.toml ./
RUN mkdir src && echo 'fn main() { println!("placeholder"); }' > src/main.rs
RUN cargo build --release || true

RUN rm -rf src
COPY src ./src
RUN cargo build --release

# ---------- Runtime stage ----------
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
RUN useradd -r -s /usr/sbin/nologin appuser
COPY --from=builder /app/target/release/aula02-scheduler-demo /usr/local/bin/app
ENV PORT=8080
EXPOSE 8080
USER appuser
CMD ["/usr/local/bin/app"]
