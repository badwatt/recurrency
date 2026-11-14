# ── Recurrency all-in-one ───────────────────────────────────
# One runtime process (the `recurrency` binary) plus the bundled PostgreSQL.
# The web frontend is embedded into the binary; there is no nginx.

# ── Frontend builder ─────────────────────────────────────
FROM ubuntu:24.04 AS frontend-builder

ENV RUSTFLAGS="-C target-cpu=generic" \
    DEBIAN_FRONTEND=noninteractive

WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential curl pkg-config libssl-dev ca-certificates npm binaryen && \
    rm -rf /var/lib/apt/lists/* && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    /root/.cargo/bin/rustup target add wasm32-unknown-unknown && \
    curl -L https://github.com/DioxusLabs/dioxus/releases/download/v0.7.10/dx-x86_64-unknown-linux-gnu.tar.gz | tar xz -C /usr/local/bin && \
    chmod +x /usr/local/bin/dx
ENV PATH="/root/.cargo/bin:${PATH}"

COPY Cargo.toml Cargo.lock* ./
COPY vendor/ ./vendor/
COPY backend/Cargo.toml backend/Cargo.toml
COPY frontend/Cargo.toml frontend/Cargo.toml
COPY tools/Cargo.toml tools/Cargo.toml
RUN mkdir -p backend/src && echo 'fn main() {}' > backend/src/main.rs
RUN mkdir -p frontend/src && echo 'fn main() {}' > frontend/src/main.rs
RUN mkdir -p tools/src && echo 'fn main() {}' > tools/src/main.rs
RUN --mount=type=cache,target=/build/target cargo build -p recurrency_frontend --features web --release 2>/dev/null || true
RUN cargo fetch

COPY frontend/ ./frontend/
RUN mkdir -p backend/src && echo 'fn main() {}' > backend/src/main.rs
RUN cd frontend && npm install tailwindcss tailwindcss-animate && \
    npx tailwindcss -i assets/styles/tailwind.src.css -o assets/styles/tailwind.css --minify
RUN cd frontend && dx bundle --platform web --release --offline
RUN test -f /build/frontend/dist/public/index.html && \
    test -n "$(find /build/frontend/dist/public/assets -name '*.js' -print -quit)"

# ── Application builder ──────────────────────────────────
FROM rust:1-bookworm AS app-builder

ENV RUSTFLAGS="-C target-cpu=generic"

WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
COPY vendor/ ./vendor/
COPY backend/Cargo.toml backend/Cargo.toml
COPY frontend/Cargo.toml frontend/Cargo.toml
COPY tools/Cargo.toml tools/Cargo.toml
RUN mkdir -p backend/src && echo 'fn main() {}' > backend/src/main.rs
RUN mkdir -p frontend/src && echo '' > frontend/src/lib.rs
RUN mkdir -p frontend/dist/public/assets && \
    printf '<!doctype html><html></html>\n' > frontend/dist/public/index.html && \
    printf 'cache\n' > frontend/dist/public/assets/placeholder.js
RUN mkdir -p tools/src && echo 'fn main() {}' > tools/src/main.rs
RUN --mount=type=cache,target=/build/target cargo build --release -p recurrency_backend --features embedded-web -j 4 2>/dev/null || true

COPY backend/ ./backend/
RUN rm -rf /build/frontend/dist
COPY --from=frontend-builder /build/frontend/dist ./frontend/dist
RUN --mount=type=cache,target=/build/target touch backend/src/main.rs && \
    cargo build --release -p recurrency_backend --features embedded-web -j 4 && \
    cp /build/target/release/recurrency_backend /build/recurrency

# ── Target: all-in-one ───────────────────────────────────
FROM debian:bookworm-slim AS all-in-one

ENV DEBIAN_FRONTEND=noninteractive

# Prevent services from auto-starting during build
RUN echo '#!/bin/sh\nexit 101' > /usr/sbin/policy-rc.d && chmod +x /usr/sbin/policy-rc.d
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    gnupg \
    && install -d /usr/share/postgresql-common/pgdg \
    && curl -fsSL -o /usr/share/postgresql-common/pgdg/apt.postgresql.org.asc \
        https://www.postgresql.org/media/keys/ACCC4CF8.asc \
    && echo "deb [signed-by=/usr/share/postgresql-common/pgdg/apt.postgresql.org.asc] https://apt.postgresql.org/pub/repos/apt bookworm-pgdg main" \
        > /etc/apt/sources.list.d/pgdg.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends postgresql-18 postgresql-contrib-18 \
    && rm -rf /var/lib/apt/lists/*

# Single application binary: backend + embedded frontend.
COPY --from=app-builder /build/recurrency /usr/local/bin/recurrency
RUN chmod +x /usr/local/bin/recurrency

# Entrypoint
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# PostgreSQL data directory (PG 18)
RUN mkdir -p /var/lib/postgresql/18/main && chown postgres:postgres /var/lib/postgresql/18/main
RUN mkdir -p /var/run/postgresql && chown postgres:postgres /var/run/postgresql
RUN mkdir -p /var/lib/recurrency/backups
VOLUME ["/var/lib/recurrency/backups"]

ENV APP_PORT=80 \
    RUST_LOG=info \
    ENABLE_WEB=true

HEALTHCHECK --interval=30s --timeout=5s --start-period=45s --retries=3 \
    CMD curl -fsS http://127.0.0.1:${APP_PORT:-80}/api/ready || exit 1

EXPOSE 80

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
