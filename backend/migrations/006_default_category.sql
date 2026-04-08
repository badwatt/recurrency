-- Add is_default column to categories
ALTER TABLE categories ADD COLUMN IF NOT EXISTS is_default BOOLEAN NOT NULL DEFAULT false;

-- Set the existing Uncategorized category as default for all users that have one
UPDATE categories SET is_default = true WHERE name = 'Uncategorized' AND is_system = true;

-- Ensure each user has exactly one default category
-- If a user doesn't have a default, set their first category as default
WITH users_without_default AS (
  SELECT u.id as user_id
  FROM users u
  WHERE NOT EXISTS (
    SELECT 1 FROM categories c WHERE c.user_id = u.id AND c.is_default = true
  )
)
UPDATE categories c
SET is_default = true
FROM users_without_default uwd
WHERE c.user_id = uwd.user_id
AND c.id = (
  SELECT id FROM categories 
  WHERE user_id = uwd.user_id 
  ORDER BY created_at ASC 
  LIMIT 1
);

-- Prevent deleting the default category (will be enforced in application code)
