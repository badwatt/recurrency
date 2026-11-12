-- Uncategorized is the fixed system fallback for every user.
INSERT INTO categories (id, user_id, name, is_system, is_default, created_at, updated_at)
SELECT gen_random_uuid(), u.id, 'Uncategorized', true, true, NOW(), NOW()
FROM users u
WHERE NOT EXISTS (
    SELECT 1 FROM categories c WHERE c.user_id = u.id AND c.is_system = true
);

UPDATE categories
SET name = 'Uncategorized',
    icon = NULL,
    color = NULL,
    is_default = is_system,
    updated_at = NOW()
WHERE is_system = true;

UPDATE categories
SET is_default = false,
    updated_at = NOW()
WHERE is_system = false AND is_default = true;

-- Only the system category may be the default.
CREATE UNIQUE INDEX IF NOT EXISTS categories_one_default_per_user
    ON categories (user_id)
    WHERE is_default = true;
