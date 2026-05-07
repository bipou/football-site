# ── Build ────────────────────────────────────────────────────────────────
FROM rust:nightly-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev cmake \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos --locked

WORKDIR /build
COPY . .
RUN cargo leptos build --release -vv

# ── Runtime ──────────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/site /app
COPY --from=builder /build/target/release/football_site /app/

WORKDIR /app
EXPOSE 7600
ENV LEPTOS_SITE_ADDR=0.0.0.0:7600
CMD ["./football_site"]
