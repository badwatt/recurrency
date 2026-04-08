-- Add is_system flag to categories
ALTER TABLE categories ADD COLUMN IF NOT EXISTS is_system BOOLEAN NOT NULL DEFAULT false;

-- Create uncategorized category for demo user
INSERT INTO categories (id, user_id, name, is_system, created_at, updated_at)
SELECT gen_random_uuid(), id, 'Uncategorized', true, NOW(), NOW()
FROM users WHERE email = 'demo@recur.app'
ON CONFLICT DO NOTHING;
