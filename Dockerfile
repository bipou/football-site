FROM rust:slim AS builder

RUN rustup toolchain install nightly --profile minimal \
    && rustup default nightly \
    && rustup toolchain remove stable \
    && rustup target add wasm32-unknown-unknown \
    && apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev cmake \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-leptos --locked

WORKDIR /build
COPY . .
RUN cargo leptos build --release \
    && rm -rf /usr/local/cargo/registry \
    && rm -rf target/wasm32-unknown-unknown target/release/build target/release/deps target/release/incremental

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/site /app/site
COPY --from=builder /build/target/release/football_site /app/

WORKDIR /app
ENV LEPTOS_OUTPUT_NAME=football_site
ENV LEPTOS_SITE_ROOT=site
ENV LEPTOS_SITE_PKG_DIR=pkg
ENV LEPTOS_SITE_ADDR=0.0.0.0:7600
CMD ["./football_site"]
