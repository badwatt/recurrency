-- Administrative users, full backups and registration control.
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_admin BOOLEAN NOT NULL DEFAULT false;

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO app_settings (key, value)
VALUES ('registration_enabled', 'true')
ON CONFLICT (key) DO NOTHING;

-- The demo account is not an administrator. Bootstrap the oldest real account
-- as administrator on existing installations.
UPDATE users SET is_admin = false WHERE email = 'demo@recurrency.app';

UPDATE users
SET is_admin = true
WHERE id = (
    SELECT id FROM users
    WHERE email <> 'demo@recurrency.app'
    ORDER BY created_at ASC, id ASC LIMIT 1
)
AND NOT EXISTS (SELECT 1 FROM users WHERE is_admin = true);

CREATE INDEX IF NOT EXISTS idx_users_is_admin ON users (is_admin);
