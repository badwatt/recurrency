-- Per-user notification channels and a dashboard-summary schedule.
-- Channel secrets are encrypted by the application before they reach this table.

CREATE TABLE IF NOT EXISTS notification_channels (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(32) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT false,
    config BYTEA NOT NULL DEFAULT '\x',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, provider)
);

CREATE INDEX IF NOT EXISTS idx_notification_channels_user_id
    ON notification_channels(user_id);

CREATE TABLE IF NOT EXISTS summary_notification_settings (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT false,
    frequency VARCHAR(16) NOT NULL DEFAULT 'monthly',
    weekday SMALLINT,
    day_of_month SMALLINT,
    hour SMALLINT NOT NULL DEFAULT 9,
    minute SMALLINT NOT NULL DEFAULT 0,
    timezone VARCHAR(64) NOT NULL DEFAULT 'UTC',
    next_run_at TIMESTAMPTZ,
    last_sent_at TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT summary_frequency_check CHECK (frequency IN ('daily', 'weekly', 'monthly')),
    CONSTRAINT summary_weekday_check CHECK (weekday IS NULL OR weekday BETWEEN 1 AND 7),
    CONSTRAINT summary_day_check CHECK (day_of_month IS NULL OR day_of_month BETWEEN 1 AND 31),
    CONSTRAINT summary_hour_check CHECK (hour BETWEEN 0 AND 23),
    CONSTRAINT summary_minute_check CHECK (minute BETWEEN 0 AND 59)
);
