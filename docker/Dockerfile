# ── Unified Dockerfile: backend / frontend / all-in-one ───
# Build with --target backend, --target frontend, or --target all-in-one

# ═══════════════════════════════════════════════════════════
# Shared builder stages (cached across all targets)
# ═══════════════════════════════════════════════════════════

# ── Backend builder ──────────────────────────────────────
FROM rust:1-bookworm AS backend-builder

ENV RUSTFLAGS="-C target-cpu=generic"

WORKDIR /build

COPY Cargo.toml Cargo.lock* ./
COPY backend/Cargo.toml backend/Cargo.toml
COPY frontend/Cargo.toml frontend/Cargo.toml
RUN mkdir -p backend/src && echo 'fn main() {}' > backend/src/main.rs
RUN mkdir -p frontend/src && echo '' > frontend/src/lib.rs
RUN --mount=type=cache,target=/build/target cargo build --release -p recurrency_backend -j 4 2>/dev/null || true

COPY backend/ ./backend/
RUN mkdir -p frontend/src && echo '' > frontend/src/lib.rs
RUN --mount=type=cache,target=/build/target touch backend/src/main.rs && cargo build --release -p recurrency_backend -j 4 && cp /build/target/release/recurrency_backend /build/recurrency_backend

# ── Frontend builder ─────────────────────────────────────
FROM ubuntu:24.04 AS frontend-builder

ENV RUSTFLAGS="-C target-cpu=generic" \
    DEBIAN_FRONTEND=noninteractive

WORKDIR /build

# Install Rust + system deps + dx CLI + wasm tooling
# binaryen provides wasm-opt (needed by `dx bundle --release`).
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential curl pkg-config libssl-dev ca-certificates npm binaryen && \
    rm -rf /var/lib/apt/lists/* && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    /root/.cargo/bin/rustup target add wasm32-unknown-unknown && \
    curl -L https://github.com/DioxusLabs/dioxus/releases/download/v0.7.10/dx-x86_64-unknown-linux-gnu.tar.gz | tar xz -C /usr/local/bin && \
    chmod +x /usr/local/bin/dx
ENV PATH="/root/.cargo/bin:${PATH}"

COPY Cargo.toml Cargo.lock* ./
COPY backend/Cargo.toml backend/Cargo.toml
COPY frontend/Cargo.toml frontend/Cargo.toml
RUN mkdir -p backend/src && echo 'fn main() {}' > backend/src/main.rs
RUN mkdir -p frontend/src && echo 'fn main() {}' > frontend/src/main.rs
RUN --mount=type=cache,target=/build/target cargo build -p recurrency_frontend --features web --release 2>/dev/null || true

COPY frontend/ ./frontend/
RUN mkdir -p backend/src && echo 'fn main() {}' > backend/src/main.rs

RUN cd frontend && npm install tailwindcss tailwindcss-animate && \
    npx tailwindcss -i assets/styles/tailwind.src.css -o assets/styles/tailwind.css --minify

RUN cd frontend && dx bundle --platform web --release && \
    ls -laR dist/ 2>/dev/null; \
    find dist/ -name 'index.html' -type f 2>/dev/null

# ═══════════════════════════════════════════════════════════
# Runtime targets
# ═══════════════════════════════════════════════════════════

# ── Target: backend only ─────────────────────────────────
FROM debian:bookworm-slim AS backend

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=backend-builder /build/recurrency_backend /usr/local/bin/recurrency_backend
RUN chmod +x /usr/local/bin/recurrency_backend

ENV APP_HOST=0.0.0.0 \
    APP_PORT=8080 \
    RUST_LOG=info

EXPOSE 8080

CMD ["recurrency_backend"]

# ── Target: frontend only ────────────────────────────────
FROM nginx:alpine AS frontend

COPY --from=frontend-builder /build/frontend/dist/public /usr/share/nginx/html
RUN rm -f /etc/nginx/conf.d/default.conf
COPY docker/nginx-standalone.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]

# ── Target: all-in-one ───────────────────────────────────
FROM debian:bookworm-slim AS all-in-one

ENV DEBIAN_FRONTEND=noninteractive

# Prevent services from auto-starting during build
RUN echo '#!/bin/sh\nexit 101' > /usr/sbin/policy-rc.d && chmod +x /usr/sbin/policy-rc.d
RUN apt-get update && apt-get install -y --no-install-recommends \
    postgresql-15 \
    nginx \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && nginx -s stop 2>/dev/null || true \
    && rm -f /etc/nginx/sites-enabled/default

# Backend binary
COPY --from=backend-builder /build/recurrency_backend /usr/local/bin/recurrency_backend
RUN chmod +x /usr/local/bin/recurrency_backend

# Frontend static files
COPY --from=frontend-builder /build/frontend/dist/public /usr/share/nginx/html

# Nginx config
COPY docker/nginx.conf /etc/nginx/sites-available/recurrency
RUN ln -sf /etc/nginx/sites-available/recurrency /etc/nginx/sites-enabled/recurrency

# Entrypoint
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# PostgreSQL data directory
RUN mkdir -p /var/lib/postgresql/15/main && chown postgres:postgres /var/lib/postgresql/15/main
RUN mkdir -p /var/run/postgresql && chown postgres:postgres /var/run/postgresql

# SECURITY: Use ARG for secrets (only available at build time, not in final image)
ARG DATABASE_URL=
ARG ENABLE_DEMO=false

# Environment variables (safe, non-sensitive)
ENV APP_HOST=0.0.0.0 \
    APP_PORT=8080 \
    RUST_LOG=info \
    ENABLE_DEMO=${ENABLE_DEMO}

# Pass secrets as runtime configuration (via entrypoint or volume)
# Do NOT set JWT_SECRET as ENV - it should come from secrets at runtime

EXPOSE 80

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
