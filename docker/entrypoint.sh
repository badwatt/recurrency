#!/bin/bash
set -e

echo "=== Recurrency ==="

export APP_PORT="${APP_PORT:-80}"
export ENABLE_DEMO="${ENABLE_DEMO:-false}"

# JWT_SECRET is optional. When absent, the application generates it once and
# stores it in app_secrets, so sessions survive container recreation.

# Start the daily logical backup loop only after the API has completed
# migrations. The loop runs in the background while the API is PID 1's child.
start_backup_loop() {
    if [ "${BACKUP_ENABLED:-true}" = "false" ]; then
        return
    fi

    export BACKUP_DIR="${BACKUP_DIR:-/var/lib/recurrency/backups}"
    export BACKUP_RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-7}"
    export BACKUP_RESTORE_TEST="${BACKUP_RESTORE_TEST:-true}"
    mkdir -p "$BACKUP_DIR"

    (
        for _ in $(seq 1 60); do
            if curl -fsS "http://127.0.0.1:${APP_PORT}/api/ready" >/dev/null 2>&1; then
                break
            fi
            sleep 2
        done
        while true; do
            /usr/local/bin/recurrency backup || echo "Recurrency backup failed; will retry."
            sleep "${BACKUP_INTERVAL_SECONDS:-86400}"
        done
    ) &
}

# ── External PostgreSQL ─────────────────────────────────
# An explicit DATABASE_URL always wins. Do not start or initialize the
# bundled PostgreSQL when the application is configured to use an external
# server.
if [ -n "${DATABASE_URL:-}" ]; then
    echo "Using external PostgreSQL from DATABASE_URL."
    start_backup_loop
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
    psql -h localhost -U postgres -c "DO \$\$ BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = '${DB_USER}') THEN CREATE ROLE ${DB_USER} LOGIN PASSWORD '${DB_PASS}'; END IF; END \$\$;" 2>/dev/null || true
    psql -h localhost -U postgres -c "CREATE DATABASE ${DB_NAME} OWNER ${DB_USER};" 2>/dev/null || true
    psql -h localhost -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE ${DB_NAME} TO ${DB_USER};" 2>/dev/null || true
fi

# The application role only needs to own its database. Keep the extension in
# place for the historical 001 migration, which is a no-op once installed.
psql -h localhost -U postgres -c "ALTER ROLE ${DB_USER} NOSUPERUSER NOCREATEDB NOCREATEROLE;" >/dev/null
psql -h localhost -U postgres -d "${DB_NAME}" -c 'CREATE EXTENSION IF NOT EXISTS "uuid-ossp";' >/dev/null

export DATABASE_URL="postgres://${DB_USER}:${DB_PASS}@localhost:5432/${DB_NAME}"
export DATABASE_ADMIN_URL="postgres://postgres@localhost:5432/postgres"

start_backup_loop
echo "Starting Recurrency on port ${APP_PORT}..."
exec /usr/local/bin/recurrency serve
