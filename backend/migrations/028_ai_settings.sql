-- Per-user BYOK configuration for the optional AI assistant.
CREATE TABLE IF NOT EXISTS ai_settings (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT false,
    provider TEXT NOT NULL DEFAULT 'openai',
    base_url TEXT NOT NULL DEFAULT 'https://api.openai.com/v1',
    model TEXT NOT NULL DEFAULT 'gpt-4o-mini',
    api_key_encrypted BYTEA,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
