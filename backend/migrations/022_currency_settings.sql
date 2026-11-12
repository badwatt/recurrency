-- Per-user exchange-rate provider configuration.
CREATE TABLE IF NOT EXISTS currency_settings (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL DEFAULT 'frankfurter' CHECK (provider IN ('frankfurter', 'fixer')),
    fixer_api_key BYTEA,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
