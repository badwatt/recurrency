# ── Multi-stage Dockerfile ──────────────────────────────────
# Dioxus frontend + Rust backend, all-in-one with PostgreSQL
# Build stages: ubuntu:24.04 (glibc 2.39 = cargo-binstall compatible)
# Runtime: debian:bookworm-slim (PostgreSQL 15)

# ── Stage 1: Build Dioxus frontend ────────────────────────
FROM ubuntu:24.04 AS frontend-build

ARG APP_VERSION=dev
ENV APP_VERSION=${APP_VERSION}
ENV RUSTFLAGS="-C target-cpu=generic" \
    DEBIAN_FRONTEND=noninteractive

# Copy VERSION for build.rs fallback
COPY VERSION /build/VERSION

# Install system deps with retry on mirror sync
RUN for i in 1 2 3; do apt-get update && apt-get install -y --no-install-recommends \
    build-essential curl pkg-config libssl-dev ca-certificates npm && break || sleep 5; done && \
    rm -rf /var/lib/apt/lists/*

# Install Rust via rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install dx CLI via binstall + wasm target
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash && \
    cargo binstall dioxus-cli --no-confirm && \
    rustup target add wasm32-unknown-unknown

WORKDIR /build/frontend
COPY frontend/Cargo.toml frontend/Cargo.lock* ./

# Cache dependencies — separate layers for better Docker caching
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --target wasm32-unknown-unknown --features web --release 2>/dev/null || true
RUN rm -rf src

COPY frontend/ ./

# Build Tailwind CSS
RUN npm install --prefix . tailwindcss tailwindcss-animate \
    && npx tailwindcss -i assets/styles/tailwind.src.css -o assets/styles/tailwind.css --minify

# Build Dioxus web bundle (dx bundle outputs to out_dir = dist/)
RUN dx bundle --platform web --release && \
    echo "=== Frontend dist/ output ===" && \
    ls -laR dist/ 2>/dev/null; \
    echo "=== Searching for index.html ===" && \
    find dist/ target/dx/ -name 'index.html' -type f 2>/dev/null; \
    test -f dist/index.html || { echo "FATAL: index.html not in dist/, checking target/dx/"; find target/dx/ -name 'index.html' -type f; }

# ── Stage 2: Build backend ──────────────────────────────────
# rust:bookworm has glibc 2.36 — binary compatible with debian:bookworm-slim runtime
FROM rust:1-bookworm AS backend-build

ARG APP_VERSION=dev
ENV APP_VERSION=${APP_VERSION}
ENV RUSTFLAGS="-C target-cpu=generic"

# Copy VERSION for build.rs fallback
COPY VERSION /build/VERSION

# Copy frontend dist from frontend-build stage (needed by include_dir!)
COPY --from=frontend-build /build/frontend/dist /build/frontend/dist

WORKDIR /build/backend
COPY backend/Cargo.toml backend/Cargo.lock* ./

# Cache dependencies — separate layers
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release -j 4 --features embed-frontend 2>/dev/null || true
RUN rm -rf src

COPY backend/ ./
RUN touch src/main.rs && cargo build --release -j 4 --features embed-frontend

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

# Copy frontend dist (dx bundle output)
# dx bundle puts files in dist/public/
COPY --from=frontend-build /build/frontend/dist/public /frontend/dist

# Also copy dx build output as fallback (dx build uses target/dx/ instead of dist/)
COPY --from=frontend-build /build/frontend/target/dx /tmp/dx_output

# Ensure index.html exists — try both output locations
RUN if [ ! -f /frontend/dist/index.html ]; then \
        IDX=$(find /tmp/dx_output -name 'index.html' -type f | head -1); \
        if [ -n "$IDX" ]; then \
            WEBDIR=$(dirname "$IDX"); \
            echo "Frontend found at: $WEBDIR"; \
            cp -r "$WEBDIR"/* /frontend/dist/; \
        fi; \
    fi && \
    test -f /frontend/dist/index.html || (echo "FATAL: index.html not found in frontend build output!" && exit 1) && \
    echo "Frontend files:" && ls -la /frontend/dist/ && \
    rm -rf /tmp/dx_output

# Copy entrypoint
COPY docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Environment defaults
ENV APP_HOST=0.0.0.0 \
    APP_PORT=80 \
    APP_ENV=development \
    BUILD_UI=true \
    ENABLE_DEMO=false \
    FRONTEND_DIST=/frontend/dist

EXPOSE 80

ENTRYPOINT ["/entrypoint.sh"]