-- Migration: Require category for subscriptions and transactions
-- Backfill NULL category_ids with the user's default category, then make NOT NULL
-- Change FK ON DELETE from SET NULL to RESTRICT (app handles reassignment before delete)

-- Step 1: Backfill subscriptions with NULL category_id to user's default category
UPDATE subscriptions s
SET category_id = c.id
FROM categories c
WHERE s.category_id IS NULL
  AND c.user_id = s.user_id
  AND c.is_default = true;

-- Step 2: Backfill transactions with NULL category_id to user's default category
UPDATE transactions t
SET category_id = c.id
FROM categories c
WHERE t.category_id IS NULL
  AND c.user_id = t.user_id
  AND c.is_default = true;

-- Step 3: Make category_id NOT NULL
ALTER TABLE subscriptions ALTER COLUMN category_id SET NOT NULL;
ALTER TABLE transactions ALTER COLUMN category_id SET NOT NULL;

-- Step 4: Drop existing FK constraints and recreate with ON DELETE RESTRICT
-- The auto-generated constraint names follow PostgreSQL's pattern: <table>_<column>_fkey
ALTER TABLE subscriptions DROP CONSTRAINT subscriptions_category_id_fkey;
ALTER TABLE subscriptions ADD CONSTRAINT subscriptions_category_id_fkey
  FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE RESTRICT;

ALTER TABLE transactions DROP CONSTRAINT transactions_category_id_fkey;
ALTER TABLE transactions ADD CONSTRAINT transactions_category_id_fkey
  FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE RESTRICT;