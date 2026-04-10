#!/bin/bash
set -e

echo "=== Recurrency - All-in-One ==="

# ── Configure PostgreSQL ─────────────────────────────────
echo "Configuring PostgreSQL..."

PG_HBA="/etc/postgresql/15/main/pg_hba.conf"
PG_CONF="/etc/postgresql/15/main/postgresql.conf"

cat > "$PG_HBA" << EOF
local   all   all   trust
host    all   all   127.0.0.1/32   trust
host    all   all   ::1/128        trust
EOF

sed -i "s/^#\?listen_addresses.*$/listen_addresses = 'localhost'/" "$PG_CONF" 2>/dev/null || true
grep -q "^listen_addresses" "$PG_CONF" || echo "listen_addresses = 'localhost'" >> "$PG_CONF"

# ── Bootstrap PostgreSQL ────────────────────────────────
echo "Starting PostgreSQL for setup..."
su - postgres -c "/usr/lib/postgresql/15/bin/pg_ctl start -D /var/lib/postgresql/15/main -o '-c config_file=/etc/postgresql/15/main/postgresql.conf' -l /var/log/postgresql/startup.log" || true

echo "Waiting for PostgreSQL..."
for i in $(seq 1 30); do
    if su - postgres -c "psql -h localhost -U recurrency -d recurrency -c 'SELECT 1'" >/dev/null 2>&1; then
        echo "PostgreSQL is ready."
        break
    fi
    su - postgres -c "psql -c \"CREATE USER recurrency WITH PASSWORD '${PGPASSWORD:-recurrency}' SUPERUSER;\"" 2>/dev/null || true
    su - postgres -c "psql -c \"CREATE DATABASE recurrency OWNER recurrency;\"" 2>/dev/null || true
    sleep 1
done

# Stop — we'll restart it in background
su - postgres -c "/usr/lib/postgresql/15/bin/pg_ctl stop -D /var/lib/postgresql/15/main -m fast" 2>/dev/null || true

# ── Start PostgreSQL in background ──────────────────────
echo "Starting PostgreSQL..."
su - postgres -c "/usr/lib/postgresql/15/bin/pg_ctl start -D /var/lib/postgresql/15/main -o '-c config_file=/etc/postgresql/15/main/postgresql.conf' -l /var/log/postgresql/postgresql.log" || true

# Wait for PostgreSQL to be ready
for i in $(seq 1 15); do
    if su - postgres -c "pg_isready -h localhost -p 5432" >/dev/null 2>&1; then
        break
    fi
    sleep 1
done

# ── Set environment for backend ────────────────────────
export DATABASE_URL="postgres://recurrency:${PGPASSWORD:-recurrency}@localhost:5432/recurrency"
export APP_HOST="${APP_HOST:-0.0.0.0}"
export APP_PORT="${APP_PORT:-80}"
export JWT_SECRET="${JWT_SECRET:-change-me-in-production}"
export ENABLE_DEMO="${ENABLE_DEMO:-false}"

echo "Starting Recurrency on port ${APP_PORT}..."
echo "Frontend: http://localhost:${APP_PORT}"
echo "API:      http://localhost:${APP_PORT}/api"
echo ""
exec /usr/local/bin/recurrency-api