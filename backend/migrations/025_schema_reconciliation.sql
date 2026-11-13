-- Reconcile databases created from the old embedded schema snapshot with the
-- migration-managed schema. New databases already get these objects from
-- migrations 010/017; this migration is intentionally idempotent.

-- A legacy local database may contain more than one default category per
-- user. Keep the oldest one before enforcing the unique index.
UPDATE categories c
SET is_default = false
WHERE c.is_default = true
  AND c.id <> (
      SELECT c2.id
      FROM categories c2
      WHERE c2.user_id = c.user_id
        AND c2.is_default = true
      ORDER BY c2.created_at ASC, c2.id ASC
      LIMIT 1
  );

CREATE UNIQUE INDEX IF NOT EXISTS categories_one_default_per_user
    ON categories (user_id)
    WHERE is_default = true;

CREATE INDEX IF NOT EXISTS idx_users_verified ON users (verified);
