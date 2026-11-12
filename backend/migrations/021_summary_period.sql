ALTER TABLE summary_notification_settings
    ADD COLUMN IF NOT EXISTS period VARCHAR(8) NOT NULL DEFAULT 'current';
ALTER TABLE summary_notification_settings
    DROP CONSTRAINT IF EXISTS summary_period_check;
ALTER TABLE summary_notification_settings
    ADD CONSTRAINT summary_period_check CHECK (period IN ('current', 'next'));
