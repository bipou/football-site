FROM rustlang/rust:nightly-slim AS builder

ARG HTTP_PROXY
ARG HTTPS_PROXY
ARG NO_PROXY

ENV http_proxy=$HTTP_PROXY \
    https_proxy=$HTTPS_PROXY \
    no_proxy=$NO_PROXY \
    HTTP_PROXY=$HTTP_PROXY \
    HTTPS_PROXY=$HTTPS_PROXY \
    NO_PROXY=$NO_PROXY

RUN rustup target add wasm32-unknown-unknown \
    && apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev cmake make perl \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-leptos

WORKDIR /build
COPY . .
RUN cargo leptos build --release \
    && rm -rf /usr/local/cargo/registry \
    && rm -rf target/wasm32-unknown-unknown target/release/build target/release/deps target/release/incremental

FROM debian:trixie-slim

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
