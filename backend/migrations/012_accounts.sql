-- Accounts are the mandatory budgeting buckets. Categories remain optional.
-- Existing categories were being used as accounts, so copy them over and
-- backfill every subscription/transaction before requiring account_id.

CREATE TABLE accounts (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR NOT NULL,
    icon VARCHAR,
    color VARCHAR,
    is_default BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_accounts_user_id ON accounts(user_id);

INSERT INTO accounts (id, user_id, name, icon, color, is_default, created_at, updated_at)
SELECT id, user_id, name, icon, color, is_default, created_at, updated_at
FROM categories;

ALTER TABLE subscriptions ADD COLUMN account_id UUID;
ALTER TABLE transactions ADD COLUMN account_id UUID;

UPDATE subscriptions SET account_id = category_id;
UPDATE transactions SET account_id = category_id;

-- Users without categories still need a default account.
INSERT INTO accounts (id, user_id, name, icon, color, is_default, created_at, updated_at)
SELECT gen_random_uuid(), u.id, 'General', 'Wallet', '#6b7280', true, NOW(), NOW()
FROM users u
WHERE NOT EXISTS (SELECT 1 FROM accounts a WHERE a.user_id = u.id);

-- Guarantee one default account per user.
UPDATE accounts a
SET is_default = true
WHERE a.id = (
    SELECT a2.id
    FROM accounts a2
    WHERE a2.user_id = a.user_id
    ORDER BY a2.created_at ASC
    LIMIT 1
)
AND NOT EXISTS (
    SELECT 1 FROM accounts a3
    WHERE a3.user_id = a.user_id AND a3.is_default = true
);

-- Backfill any remaining rows from the default account.
UPDATE subscriptions s
SET account_id = a.id
FROM accounts a
WHERE s.account_id IS NULL AND a.user_id = s.user_id AND a.is_default = true;

UPDATE transactions t
SET account_id = a.id
FROM accounts a
WHERE t.account_id IS NULL AND a.user_id = t.user_id AND a.is_default = true;

ALTER TABLE subscriptions ALTER COLUMN account_id SET NOT NULL;
ALTER TABLE transactions ALTER COLUMN account_id SET NOT NULL;

ALTER TABLE subscriptions ADD CONSTRAINT subscriptions_account_id_fkey
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE RESTRICT;
ALTER TABLE transactions ADD CONSTRAINT transactions_account_id_fkey
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE RESTRICT;

CREATE INDEX idx_subscriptions_account_id ON subscriptions(account_id);
CREATE INDEX idx_transactions_account_id ON transactions(account_id);

-- Categories stay required in storage for report compatibility; the UI may
-- omit them and the service assigns the user's default category.
