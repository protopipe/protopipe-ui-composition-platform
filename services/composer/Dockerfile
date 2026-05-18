FROM rust:1.95-bookworm AS source

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

FROM source AS builder

RUN cargo build --release

FROM rust:1.95-bookworm AS dev

WORKDIR /app

RUN cargo install bacon --locked

CMD ["bacon", "run-long"]

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/protopipe-composer /usr/local/bin/protopipe-composer

EXPOSE 8080 9000

HEALTHCHECK --interval=5s --timeout=2s --retries=12 \
    CMD curl -fsS http://127.0.0.1:9000/admin/health >/dev/null || exit 1

CMD ["protopipe-composer"]
