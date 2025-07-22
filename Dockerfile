FROM rust:1.88.0-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates && rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

COPY --from=builder /app/target/release/rust_ogc_features_server /usr/local/bin/rust_ogc_features_server

WORKDIR /app

COPY config.toml .

EXPOSE 3022

CMD ["rust_ogc_features_server"] 