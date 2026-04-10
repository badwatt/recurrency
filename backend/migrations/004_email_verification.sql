-- Email verification
ALTER TABLE users ADD COLUMN IF NOT EXISTS verified BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE users ADD COLUMN IF NOT EXISTS verification_code VARCHAR(6);

CREATE INDEX IF NOT EXISTS idx_users_verified ON users(verified);

-- Mark demo user as verified
UPDATE users SET verified = true WHERE email = 'demo@recurrency.app';

-- Ensure demo user exists and is verified (in case db started fresh)
INSERT INTO users (id, email, password_hash, name, currency, verified, created_at, updated_at)
VALUES (
    gen_random_uuid(),
    'demo@recurrency.app',
    '$argon2id$v=19$m=65536,t=3,p=4$1DHP6Y+8Y+KZ8pETQhgWmw$NubAUCWr1x65aiJKpq2bZshbL483/reR1hZ3qedOjZ8',
    'Demo User',
    'USD',
    true,
    NOW(),
    NOW()
)
ON CONFLICT (email) DO UPDATE SET password_hash = EXCLUDED.password_hash, verified = true;
