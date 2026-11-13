-- Application-level secrets that must be shared by every instance using the
-- same database. The JWT signing secret is generated on first boot when
-- JWT_SECRET is not set.
CREATE TABLE IF NOT EXISTS app_secrets (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
