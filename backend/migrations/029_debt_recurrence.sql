-- Automatic debts can repeat weekly, monthly or yearly. Keep the legacy
-- payment_days column in sync, but store the full recurrence as JSONB.
ALTER TABLE debts ADD COLUMN IF NOT EXISTS recurrence JSONB;

UPDATE debts
SET recurrence = jsonb_build_object(
    'interval', 1,
    'unit', 'month',
    'days', COALESCE(payment_days, '[]'::jsonb),
    'weekdays', '[]'::jsonb,
    'dates', '[]'::jsonb
)
WHERE recurrence IS NULL AND debt_type = 'automatic';

UPDATE debts
SET recurrence = '{"interval":1,"unit":"month","days":[],"weekdays":[],"dates":[]}'::jsonb
WHERE recurrence IS NULL;

ALTER TABLE debts ALTER COLUMN recurrence SET NOT NULL;
ALTER TABLE debts ALTER COLUMN recurrence
    SET DEFAULT '{"interval":1,"unit":"month","days":[],"weekdays":[],"dates":[]}'::jsonb;
