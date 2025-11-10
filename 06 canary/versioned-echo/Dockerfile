# Build
FROM rust:1.80 as builder
WORKDIR /app
COPY Cargo.toml .
COPY src ./src
RUN cargo build --release

# Runtime
FROM gcr.io/distroless/cc-debian12
ENV APP_VERSION=v1
COPY --from=builder /app/target/release/versioned-echo /usr/local/bin/versioned-echo
EXPOSE 8080
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/versioned-echo"]
