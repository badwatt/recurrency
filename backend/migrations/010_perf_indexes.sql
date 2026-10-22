-- Composite indexes for the hot report/dashboard/notification queries.
-- Reports filter transactions by user + date range and subscriptions by
-- user + active; notifications list unread by user + recency.
CREATE INDEX IF NOT EXISTS idx_transactions_user_date
    ON transactions(user_id, date);

CREATE INDEX IF NOT EXISTS idx_subscriptions_user_active
    ON subscriptions(user_id, is_active);

CREATE INDEX IF NOT EXISTS idx_subscriptions_user_next_billing
    ON subscriptions(user_id, next_billing_date);

CREATE INDEX IF NOT EXISTS idx_notifications_user_read_created
    ON notifications(user_id, is_read, created_at DESC);
