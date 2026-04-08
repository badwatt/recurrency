-- Add last_demo_reset column to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_demo_reset TIMESTAMPTZ;
