#!/bin/bash
set -e

echo "=== Recurrency - All-in-One ==="

# ── Configure PostgreSQL ─────────────────────────────────
echo "Configuring PostgreSQL..."

PG_HBA="/etc/postgresql/15/main/pg_hba.conf"
PG_CONF="/etc/postgresql/15/main/postgresql.conf"

# Trust all local connections (all-in-one container, no external access)
cat > "$PG_HBA" << EOF
local   all   all   trust
host    all   all   127.0.0.1/32   trust
host    all   all   ::1/128        trust
EOF

# Listen on localhost (backend connects via TCP)
sed -i "s/^#\?listen_addresses.*$/listen_addresses = 'localhost'/" "$PG_CONF" 2>/dev/null || true
grep -q "^listen_addresses" "$PG_CONF" || echo "listen_addresses = 'localhost'" >> "$PG_CONF"

# ── Start PostgreSQL ────────────────────────────────────
echo "Starting PostgreSQL..."
su - postgres -c "/usr/lib/postgresql/15/bin/pg_ctl start -D /var/lib/postgresql/15/main -o '-c config_file=/etc/postgresql/15/main/postgresql.conf' -l /var/log/postgresql/postgresql.log" || true

# ── Wait for PostgreSQL to accept real queries ─────────
# pg_isready only checks TCP — we need actual DB connectivity
echo "Waiting for PostgreSQL to be ready..."
PGPASSWORD="${PGPASSWORD:-recurrency}"
export PGPASSWORD
for i in $(seq 1 60); do
    if psql -h localhost -U postgres -c "SELECT 1" >/dev/null 2>&1; then
        echo "PostgreSQL is ready."
        break
    fi
    sleep 1
done

# ── Create user/database if absent ────────────────────
DB_USER="recurrency"
DB_PASS="${PGPASSWORD:-recurrency}"
DB_NAME="recurrency"

# Try to connect as recurrency first — if it fails, create user
if ! psql "postgres://${DB_USER}:${DB_PASS}@localhost:5432/${DB_NAME}" -c "SELECT 1" >/dev/null 2>&1; then
    echo "Creating user and database..."
    psql -h localhost -U postgres -c "CREATE USER ${DB_USER} WITH PASSWORD '${DB_PASS}' SUPERUSER;" 2>/dev/null || true
    psql -h localhost -U postgres -c "CREATE DATABASE ${DB_NAME} OWNER ${DB_USER};" 2>/dev/null || true
    psql -h localhost -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE ${DB_NAME} TO ${DB_USER};" 2>/dev/null || true
    echo "Database configured."
fi

# ── Verify connectivity ─────────────────────────────────
# Run a real query with the exact connection string backend will use
echo "Verifying database connectivity..."
for i in $(seq 1 30); do
    if psql "postgres://${DB_USER}:${DB_PASS}@localhost:5432/${DB_NAME}" -c "SELECT 1" >/dev/null 2>&1; then
        echo "Database reachable."
        break
    fi
    sleep 1
done

# ── Set environment for backend ───────────────────────────
export DATABASE_URL="postgres://${DB_USER}:${DB_PASS}@localhost:5432/${DB_NAME}"
export APP_HOST="${APP_HOST:-0.0.0.0}"
export APP_PORT="${APP_PORT:-80}"
export JWT_SECRET="${JWT_SECRET:-change-me-in-production}"
export ENABLE_DEMO="${ENABLE_DEMO:-false}"

echo "Starting Recurrency on port ${APP_PORT}..."
echo "Frontend: http://localhost:${APP_PORT}"
echo "API:      http://localhost:${APP_PORT}/api"
echo ""
exec /usr/local/bin/recurrency-api
