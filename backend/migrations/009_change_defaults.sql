-- Change defaults: EUR currency, notifications OFF
ALTER TABLE users ALTER COLUMN currency SET DEFAULT 'EUR';
ALTER TABLE subscriptions ALTER COLUMN currency SET DEFAULT 'EUR';
ALTER TABLE transactions ALTER COLUMN currency SET DEFAULT 'EUR';
ALTER TABLE notification_preferences ALTER COLUMN enabled SET DEFAULT false;

-- Migrate existing data
UPDATE notification_preferences SET enabled = false WHERE enabled = true;
UPDATE users SET currency = 'EUR' WHERE currency = 'USD';