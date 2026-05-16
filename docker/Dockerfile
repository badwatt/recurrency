# ── Recurrency All-in-One ──────────────────────────────────
# Single image: PostgreSQL + Backend + Frontend (nginx)
# Build targets: backend binary + frontend static files

# ── Backend builder ──────────────────────────────────────
FROM rust:1-bookworm AS backend-builder

ARG APP_VERSION=dev
ENV APP_VERSION=${APP_VERSION}
ENV RUSTFLAGS="-C target-cpu=generic"

WORKDIR /build

COPY Cargo.toml Cargo.lock* ./
COPY backend/Cargo.toml backend/Cargo.toml
COPY frontend/Cargo.toml frontend/Cargo.toml
COPY VERSION /build/VERSION
RUN mkdir -p backend/src && echo 'fn main() {}' > backend/src/main.rs
RUN mkdir -p frontend/src && echo '' > frontend/src/lib.rs
RUN cargo build --release -p recurrency-api -j 4 2>/dev/null || true
RUN rm -rf backend/src frontend/src

COPY backend/ ./backend/
RUN mkdir -p frontend/src && echo '' > frontend/src/lib.rs
RUN touch backend/src/main.rs && cargo build --release -p recurrency-api -j 4

# ── Frontend builder ─────────────────────────────────────
FROM ubuntu:24.04 AS frontend-builder

ARG APP_VERSION=dev
ENV APP_VERSION=${APP_VERSION}
ENV RUSTFLAGS="-C target-cpu=generic" \
    DEBIAN_FRONTEND=noninteractive

RUN for i in 1 2 3; do apt-get update && apt-get install -y --no-install-recommends \
    build-essential curl pkg-config libssl-dev ca-certificates npm && break || sleep 5; done && \
    rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash && \
    cargo binstall dioxus-cli --no-confirm && \
    rustup target add wasm32-unknown-unknown

WORKDIR /build

COPY Cargo.toml Cargo.lock* ./
COPY backend/Cargo.toml backend/Cargo.toml
COPY frontend/Cargo.toml frontend/Cargo.toml
RUN mkdir -p backend/src && echo 'fn main() {}' > backend/src/main.rs
RUN mkdir -p frontend/src && echo 'fn main() {}' > frontend/src/main.rs
RUN cargo build --target wasm32-unknown-unknown -p recurrency --features web --release 2>/dev/null || true
RUN rm -rf frontend/src backend/src

COPY frontend/ ./frontend/
RUN mkdir -p backend/src && echo 'fn main() {}' > backend/src/main.rs

RUN cd frontend && npm install tailwindcss tailwindcss-animate && \
    npx tailwindcss -i assets/styles/tailwind.src.css -o assets/styles/tailwind.css --minify

RUN cd frontend && dx bundle --platform web --release

# ── All-in-One Runtime ───────────────────────────────────
FROM debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive

# Install PostgreSQL 15, nginx, and ca-certificates
RUN apt-get update && apt-get install -y --no-install-recommends \
    postgresql-15 \
    nginx \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Backend binary
COPY --from=backend-builder /build/target/release/recurrency-api /usr/local/bin/recurrency-api
RUN chmod +x /usr/local/bin/recurrency-api

# Frontend static files
COPY --from=frontend-builder /build/frontend/dist/public /usr/share/nginx/html

# Nginx config
COPY docker/nginx.conf /etc/nginx/sites-available/default

# Entrypoint
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# PostgreSQL data directory
RUN mkdir -p /var/lib/postgresql/15/main && chown postgres:postgres /var/lib/postgresql/15/main
RUN mkdir -p /var/run/postgresql && chown postgres:postgres /var/run/postgresql

ENV APP_HOST=0.0.0.0 \
    APP_PORT=80 \
    RUST_LOG=info \
    DATABASE_URL="" \
    JWT_SECRET=change-me-in-production \
    ENABLE_DEMO=false

EXPOSE 80

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]