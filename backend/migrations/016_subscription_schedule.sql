-- Recurrence details separate from the start date, plus an optional end date.
ALTER TABLE subscriptions ADD COLUMN IF NOT EXISTS schedule_day SMALLINT;
ALTER TABLE subscriptions ADD COLUMN IF NOT EXISTS schedule_month SMALLINT;
ALTER TABLE subscriptions ADD COLUMN IF NOT EXISTS end_date DATE;

UPDATE subscriptions
SET schedule_day = CASE
        WHEN frequency = 'weekly' THEN EXTRACT(ISODOW FROM start_date)::SMALLINT
        WHEN frequency IN ('monthly', 'quarterly', 'yearly') THEN EXTRACT(DAY FROM start_date)::SMALLINT
        ELSE NULL
    END,
    schedule_month = CASE
        WHEN frequency = 'yearly' THEN EXTRACT(MONTH FROM start_date)::SMALLINT
        ELSE NULL
    END
WHERE schedule_day IS NULL;
