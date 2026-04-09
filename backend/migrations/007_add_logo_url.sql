-- Add logo_url column to subscriptions and transactions tables

ALTER TABLE subscriptions
ADD COLUMN logo_url TEXT;

ALTER TABLE transactions
ADD COLUMN logo_url TEXT;