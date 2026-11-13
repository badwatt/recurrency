-- Allow "0" as the monthly summary day, meaning the last day of the month.
ALTER TABLE summary_notification_settings
    DROP CONSTRAINT IF EXISTS summary_day_check;
ALTER TABLE summary_notification_settings
    ADD CONSTRAINT summary_day_check CHECK (day_of_month IS NULL OR day_of_month BETWEEN 0 AND 31);
