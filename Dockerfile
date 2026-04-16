# ── Multi-stage Dockerfile ──────────────────────────────────
# Dioxus frontend + Rust backend, all-in-one with PostgreSQL

# ── Stage 1: Build Dioxus frontend ────────────────────────
FROM rust:bookworm AS frontend-build

ARG APP_VERSION=dev
ENV RUSTFLAGS="-C target-cpu=generic"

# Install dx CLI + wasm target
RUN cargo install dioxus-cli && rustup target add wasm32-unknown-unknown

# Install npm for tailwind
RUN apt-get update && apt-get install -y --no-install-recommends npm && rm -rf /var/lib/apt/lists/*

WORKDIR /build/frontend
COPY frontend/Cargo.toml frontend/Cargo.lock* ./

# Cache dependencies
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --target wasm32-unknown-unknown --features web 2>/dev/null || true
RUN rm -rf src

COPY frontend/ ./

# Build Tailwind CSS
RUN npm install --prefix . tailwindcss tailwindcss-animate && npx tailwindcss -i assets/tailwind.src.css -o assets/tailwind.css --minify

# Build Dioxus web bundle
RUN dx bundle --platform web --release 2>&1 || true

# ── Stage 2: Build backend ──────────────────────────────────
FROM rust:bookworm AS backend-build

ENV RUSTFLAGS="-C target-cpu=generic"

WORKDIR /build/backend
COPY backend/Cargo.toml backend/Cargo.lock* ./

# Cache dependencies
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release -j 4 && rm -rf src

COPY backend/ ./
RUN touch src/main.rs && cargo build --release -j 4

# ── Stage 3: All-in-one runtime ────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    postgresql-15 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Setup PostgreSQL
RUN pg_dropcluster 15 main || true && pg_createcluster 15 main -p 5432 -- --auth-local=trust --auth-host=trust

# Copy backend binary
COPY --from=backend-build /build/backend/target/release/recurrency-api /usr/local/bin/recurrency-api
RUN chmod +x /usr/local/bin/recurrency-api

# Copy frontend dist
COPY --from=frontend-build /build/frontend/dist /frontend/dist

# Copy entrypoint
COPY docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Environment defaults
ENV APP_HOST=0.0.0.0 \
    APP_PORT=80 \
    ENABLE_DEMO=false \
    FRONTEND_DIST=/frontend/dist

EXPOSE 80

ENTRYPOINT ["/entrypoint.sh"]