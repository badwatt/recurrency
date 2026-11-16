-- The demo account is created on demand by demo mode. Older releases seeded
-- demo@recurrency.app on every installation, which made it appear as a real
-- account in the admin panel. Remove that seed and its demo-owned data.
DELETE FROM debt_payments
WHERE user_id IN (SELECT id FROM users WHERE email = 'demo@recurrency.app');
DELETE FROM debts
WHERE user_id IN (SELECT id FROM users WHERE email = 'demo@recurrency.app');
DELETE FROM subscriptions
WHERE user_id IN (SELECT id FROM users WHERE email = 'demo@recurrency.app');
DELETE FROM transactions
WHERE user_id IN (SELECT id FROM users WHERE email = 'demo@recurrency.app');
DELETE FROM notifications
WHERE user_id IN (SELECT id FROM users WHERE email = 'demo@recurrency.app');
DELETE FROM notification_preferences
WHERE user_id IN (SELECT id FROM users WHERE email = 'demo@recurrency.app');
DELETE FROM notification_channels
WHERE user_id IN (SELECT id FROM users WHERE email = 'demo@recurrency.app');
DELETE FROM summary_notification_settings
WHERE user_id IN (SELECT id FROM users WHERE email = 'demo@recurrency.app');
DELETE FROM user_preferences
WHERE user_id IN (SELECT id FROM users WHERE email = 'demo@recurrency.app');
DELETE FROM currency_settings
WHERE user_id IN (SELECT id FROM users WHERE email = 'demo@recurrency.app');
DELETE FROM categories
WHERE user_id IN (SELECT id FROM users WHERE email = 'demo@recurrency.app');
DELETE FROM accounts
WHERE user_id IN (SELECT id FROM users WHERE email = 'demo@recurrency.app');
DELETE FROM users WHERE email = 'demo@recurrency.app';
