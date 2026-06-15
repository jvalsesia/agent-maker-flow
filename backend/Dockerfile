# syntax=docker/dockerfile:1

# ---- Build stage ---------------------------------------------------------
# Pinned to the toolchain this repo builds with (rustc 1.95).
FROM rust:1.95-bookworm AS builder

# reqwest (0.12) uses native-tls by default -> needs OpenSSL + pkg-config to link.
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 1) Cache the dependency build: copy only the manifests, compile a dummy bin.
#    This layer is reused on every source-only change.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

# 2) Build the real binary. Migrations are embedded at compile time via
#    sqlx::migrate!("./migrations"), so they must be present here.
COPY src ./src
COPY migrations ./migrations
# Bust the dummy-main cache so the real sources are recompiled.
RUN touch src/main.rs && cargo build --release

# ---- Runtime stage -------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# ca-certificates: outbound HTTPS (Clerk JWKS, LiteLLM). libssl3: native-tls runtime.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --no-create-home --uid 10001 appuser

WORKDIR /app
COPY --from=builder /app/target/release/agent-maker-flow-backend /usr/local/bin/agent-maker-flow-backend

USER appuser

# Matches the default BIND_ADDR (0.0.0.0:8080) in config.rs.
EXPOSE 8080
ENV RUST_LOG=info

ENTRYPOINT ["agent-maker-flow-backend"]
