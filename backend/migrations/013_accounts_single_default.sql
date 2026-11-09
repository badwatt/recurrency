-- If legacy categories marked several accounts as default, keep only the
-- oldest one. This is idempotent: it does nothing when there is already a
-- single default.
UPDATE accounts a
SET is_default = false
WHERE a.is_default = true
  AND a.id <> (
      SELECT a2.id
      FROM accounts a2
      WHERE a2.user_id = a.user_id
      ORDER BY a2.created_at ASC
      LIMIT 1
  );
