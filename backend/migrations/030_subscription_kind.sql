-- Subscriptions and budgets share the same recurrence engine; only the
-- user-facing type differs.
ALTER TABLE subscriptions
    ADD COLUMN IF NOT EXISTS kind VARCHAR(16) NOT NULL DEFAULT 'subscription';

ALTER TABLE subscriptions
    ADD CONSTRAINT subscriptions_kind_check
    CHECK (kind IN ('subscription', 'budget'));
