FROM rust:1.95-bookworm AS build

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/protopipe-message-worker-mock /usr/local/bin/protopipe-message-worker-mock

ENV RUST_LOG=info
EXPOSE 9100

ENTRYPOINT ["protopipe-message-worker-mock"]
