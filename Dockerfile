# ── All-in-one runtime (no multi-stage build) ──────────────
# Artifacts (frontend dist + backend binary) are built in CI
# and passed via build context — no compilation in Docker.

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    postgresql-15 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Setup PostgreSQL
RUN pg_dropcluster 15 main || true && pg_createcluster 15 main -p 5432 -- --auth-local=trust --auth-host=trust

# Copy pre-built backend binary
COPY backend-binary/recur-api /usr/local/bin/recur-api
RUN chmod +x /usr/local/bin/recur-api

# Copy pre-built frontend dist
COPY frontend/dist /frontend/dist

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