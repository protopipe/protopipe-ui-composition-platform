FROM rust:1.95-bookworm AS build

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

FROM rust:1.95-bookworm AS dev
WORKDIR /app
RUN cargo install bacon --locked
CMD ["bacon", "run-long"]

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/protopipe-messagebridge /usr/local/bin/protopipe-messagebridge

ENV RUST_LOG=info
EXPOSE 8082

ENTRYPOINT ["protopipe-messagebridge"]
