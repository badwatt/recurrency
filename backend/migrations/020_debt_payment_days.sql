-- Automatic debts can be charged on several days of the month.
ALTER TABLE debts ADD COLUMN IF NOT EXISTS payment_days JSONB NOT NULL DEFAULT '[]'::jsonb;
ALTER TABLE debts ADD COLUMN IF NOT EXISTS start_date DATE;
UPDATE debts SET start_date = COALESCE(next_payment_date, created_at::date) WHERE start_date IS NULL;
ALTER TABLE debts ALTER COLUMN start_date SET NOT NULL;
