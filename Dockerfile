# ── Stage 1: Build frontend ────────────────────────────────
FROM node:22-alpine AS frontend-build

RUN npm install -g bun@1.3.11

WORKDIR /build/frontend
COPY frontend/package.json frontend/bun.lock* ./
RUN bun install --frozen-lockfile
COPY frontend/ ./
RUN bun run build

# ── Stage 2: Build backend ────────────────────────────────
FROM rust:bookworm AS backend-build

WORKDIR /build/backend
COPY backend/Cargo.toml backend/Cargo.lock* ./

# Cache dependencies by building a dummy main
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src

COPY backend/ ./
RUN touch src/main.rs && cargo build --release

# ── Stage 3: All-in-one runtime ──────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    postgresql-15 \
    nginx \
    supervisor \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Setup PostgreSQL on port 5432
RUN pg_dropcluster 15 main || true && pg_createcluster 15 main -p 5432 -- --auth-local=trust --auth-host=trust

# Setup process manager
RUN mkdir -p /var/log/supervisor /var/log/nginx
COPY docker/supervisord.conf /etc/supervisor/conf.d/recur.conf

# Copy backend binary
COPY --from=backend-build /build/backend/target/release/recur-api /usr/local/bin/recur-api

# Copy frontend dist
COPY --from=frontend-build /build/frontend/dist /opt/recur/frontend

# Copy config
COPY docker/nginx.conf /opt/recur/nginx.conf
COPY docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Environment defaults
ENV DATABASE_URL="postgres://recur:recur@localhost:5432/recur" \
    APP_HOST=0.0.0.0 \
    APP_PORT=8080 \
    JWT_SECRET=change-me-in-production \
    ENABLE_DEMO=false \
    PGUSER=recur \
    PGPASSWORD=recur \
    PGDATABASE=recur

EXPOSE 80

ENTRYPOINT ["/entrypoint.sh"]