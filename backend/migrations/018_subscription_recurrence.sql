-- Unified recurrence model. Existing schedules are migrated losslessly.
ALTER TABLE subscriptions ADD COLUMN IF NOT EXISTS recurrence JSONB;

UPDATE subscriptions
SET recurrence = CASE frequency
    WHEN 'daily' THEN jsonb_build_object('interval', 1, 'unit', 'day')
    WHEN 'weekly' THEN jsonb_build_object(
        'interval', 1,
        'unit', 'week',
        'weekdays', jsonb_build_array(
            COALESCE(schedule_day, EXTRACT(ISODOW FROM start_date)::smallint)
        )
    )
    WHEN 'monthly' THEN jsonb_build_object(
        'interval', 1,
        'unit', 'month',
        'days', jsonb_build_array(
            COALESCE(schedule_day, EXTRACT(DAY FROM start_date)::smallint)
        )
    )
    WHEN 'quarterly' THEN jsonb_build_object(
        'interval', 3,
        'unit', 'month',
        'days', jsonb_build_array(
            COALESCE(schedule_day, EXTRACT(DAY FROM start_date)::smallint)
        )
    )
    WHEN 'yearly' THEN jsonb_build_object(
        'interval', 1,
        'unit', 'year',
        'dates', jsonb_build_array(jsonb_build_object(
            'month', COALESCE(schedule_month, EXTRACT(MONTH FROM start_date)::smallint),
            'day', COALESCE(schedule_day, EXTRACT(DAY FROM start_date)::smallint)
        ))
    )
    ELSE jsonb_build_object(
        'interval', 1,
        'unit', 'month',
        'days', jsonb_build_array(EXTRACT(DAY FROM start_date)::smallint)
    )
END;

ALTER TABLE subscriptions
    ALTER COLUMN recurrence SET DEFAULT '{"interval":1,"unit":"month","days":[1]}'::jsonb,
    ALTER COLUMN recurrence SET NOT NULL;
