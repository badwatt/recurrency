#!/bin/bash
set -e

echo "=== Recurrency ==="

export APP_PORT="${APP_PORT:-80}"
export ENABLE_DEMO="${ENABLE_DEMO:-false}"

# Validate runtime configuration before doing any database work.
if [ -z "${JWT_SECRET:-}" ]; then
    echo "ERROR: JWT_SECRET must be set to a strong random value" >&2
    exit 1
fi
export JWT_SECRET

# ── External PostgreSQL ─────────────────────────────────
# An explicit DATABASE_URL always wins. Do not start or initialize the
# bundled PostgreSQL when the application is configured to use an external
# server.
if [ -n "${DATABASE_URL:-}" ]; then
    echo "Using external PostgreSQL from DATABASE_URL."
    exec /usr/local/bin/recurrency serve
fi

# ── Configure bundled PostgreSQL ────────────────────────
PG_HBA="/etc/postgresql/18/main/pg_hba.conf"
PG_CONF="/etc/postgresql/18/main/postgresql.conf"
PG_DATA="/var/lib/postgresql/18/main"

cat > "$PG_HBA" << PGEOF
local   all   all   trust
host    all   all   127.0.0.1/32   trust
host    all   all   ::1/128        trust
PGEOF

sed -i "s/^#\?listen_addresses.*$/listen_addresses = 'localhost'/" "$PG_CONF" 2>/dev/null || true
grep -q "^listen_addresses" "$PG_CONF" || echo "listen_addresses = 'localhost'" >> "$PG_CONF"

# Self-heal ownership for volumes created by older images.
chown -R postgres:postgres "$PG_DATA"
rm -f "$PG_DATA/postmaster.pid"
mkdir -p /var/log/postgresql
chown -R postgres:postgres /var/log/postgresql

echo "Starting bundled PostgreSQL 18..."
su - postgres -c "/usr/lib/postgresql/18/bin/pg_ctl start -D $PG_DATA -o '-c config_file=$PG_CONF' -l /var/log/postgresql/postgresql.log"

# ── Wait for PostgreSQL and create the database ────────
echo "Waiting for PostgreSQL..."
export PGPASSWORD="${PGPASSWORD:-recurrency}"
for _ in $(seq 1 60); do
    if psql -h localhost -U postgres -c "SELECT 1" >/dev/null 2>&1; then
        break
    fi
    sleep 1
done

DB_USER="recurrency"
DB_PASS="${PGPASSWORD:-recurrency}"
DB_NAME="recurrency"

if ! psql "postgres://${DB_USER}:${DB_PASS}@localhost:5432/${DB_NAME}" -c "SELECT 1" >/dev/null 2>&1; then
    echo "Creating database..."
    psql -h localhost -U postgres -c "CREATE USER ${DB_USER} WITH PASSWORD '${DB_PASS}' SUPERUSER;" 2>/dev/null || true
    psql -h localhost -U postgres -c "CREATE DATABASE ${DB_NAME} OWNER ${DB_USER};" 2>/dev/null || true
    psql -h localhost -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE ${DB_NAME} TO ${DB_USER};" 2>/dev/null || true
fi

export DATABASE_URL="postgres://${DB_USER}:${DB_PASS}@localhost:5432/${DB_NAME}"

echo "Starting Recurrency on port ${APP_PORT}..."
exec /usr/local/bin/recurrency serve
