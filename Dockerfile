# ── Stage 1: Build frontend ────────────────────────────────
FROM node:22-alpine AS frontend-build

RUN npm install -g bun@1.3.11

WORKDIR /build/frontend
COPY frontend/package.json frontend/bun.lock* ./
RUN bun install --frozen-lockfile
COPY frontend/ ./
RUN bun run build --mode production

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
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Setup PostgreSQL on port 5432
RUN pg_dropcluster 15 main || true && pg_createcluster 15 main -p 5432 -- --auth-local=trust --auth-host=trust

# Copy backend binary
COPY --from=backend-build /build/backend/target/release/recur-api /usr/local/bin/recur-api

# Copy frontend dist (backend serves it directly)
COPY --from=frontend-build /build/frontend/dist /frontend/dist

# Copy entrypoint
COPY docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Environment defaults
ENV DATABASE_URL="postgres://recur:recur@localhost:5432/recur" \
    APP_HOST=0.0.0.0 \
    APP_PORT=80 \
    JWT_SECRET=change-me-in-production \
    ENABLE_DEMO=false \
    FRONTEND_DIST=/frontend/dist \
    PGUSER=recur \
    PGPASSWORD=recur \
    PGDATABASE=recur

EXPOSE 80

ENTRYPOINT ["/entrypoint.sh"]