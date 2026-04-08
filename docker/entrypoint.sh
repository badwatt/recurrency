#!/bin/bash
set -e

echo "=== Recur - All-in-One ==="

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

# ── Configure nginx ──────────────────────────────────────
echo "Configuring nginx..."
rm -f /etc/nginx/sites-enabled/default 2>/dev/null || true
cp /opt/recur/nginx.conf /etc/nginx/nginx.conf

# ── Bootstrap PostgreSQL ────────────────────────────────
echo "Starting PostgreSQL for setup..."
su - postgres -c "/usr/lib/postgresql/15/bin/pg_ctl start -D /var/lib/postgresql/15/main -o '-c config_file=/etc/postgresql/15/main/postgresql.conf' -l /var/log/postgresql/startup.log" || true

echo "Waiting for PostgreSQL..."
for i in $(seq 1 30); do
    if su - postgres -c "psql -h localhost -U recur -d recur -c 'SELECT 1'" >/dev/null 2>&1; then
        echo "PostgreSQL is ready."
        break
    fi
    su - postgres -c "psql -c \"CREATE USER recur WITH PASSWORD '${PGPASSWORD:-recur}' SUPERUSER;\"" 2>/dev/null || true
    su - postgres -c "psql -c \"CREATE DATABASE recur OWNER recur;\"" 2>/dev/null || true
    sleep 1
done

# Stop — supervisord will manage it
su - postgres -c "/usr/lib/postgresql/15/bin/pg_ctl stop -D /var/lib/postgresql/15/main -m fast" 2>/dev/null || true

# ── Export env for backend ───────────────────────────────
export DATABASE_URL="postgres://recur:${PGPASSWORD:-recur}@localhost:5432/recur"
export APP_HOST="${APP_HOST:-0.0.0.0}"
export APP_PORT="${APP_PORT:-8080}"
export JWT_SECRET="${JWT_SECRET:-change-me-in-production}"
export ENABLE_DEMO="${ENABLE_DEMO:-false}"

echo "Starting all services (nginx:80, backend:8080, postgresql:5432)..."
exec /usr/bin/supervisord -c /etc/supervisor/conf.d/recur.conf