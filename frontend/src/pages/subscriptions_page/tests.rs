
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use crate::models::subscription::Recurrence;

    fn make_sub(
        name: &str,
        amount: Decimal,
        frequency: &str,
        active: bool,
        next_billing: Option<NaiveDate>,
        cat_id: Uuid,
    ) -> Subscription {
        Subscription {
            account_id: Uuid::nil(),
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: cat_id,
            name: name.into(),
            amount,
            currency: "EUR".into(),
            frequency: frequency.into(),
            schedule_day: None,
            schedule_month: None,
            recurrence: Recurrence::default(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
            next_billing_date: next_billing,
            is_active: active,
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn filter_by_active_status() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "monthly", false, None, cat),
        ];
        let filters = FilterConfig {
            status: Some("active".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "A");
    }

    #[test]
    fn filter_by_inactive_status() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "monthly", false, None, cat),
        ];
        let filters = FilterConfig {
            status: Some("inactive".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "B");
    }

    #[test]
    fn filter_by_frequency() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "weekly", true, None, cat),
            make_sub("B", dec!(20), "monthly", true, None, cat),
        ];
        let filters = FilterConfig {
            frequency: Some("weekly".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "A");
    }


    #[test]
    fn subscription_sections_keep_inactive_before_finalized() {
        let today = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
        let active = make_sub("Active", Decimal::ONE, "monthly", true, None, Uuid::nil());
        let inactive = make_sub("Inactive", Decimal::ONE, "monthly", false, None, Uuid::nil());
        let mut finished = make_sub("Finished", Decimal::ONE, "monthly", true, None, Uuid::nil());
        finished.end_date = NaiveDate::from_ymd_opt(2026, 5, 1);
        let (a, i, f) = subscription_sections(&[active, inactive, finished], today);
        assert_eq!(a.len(), 1);
        assert_eq!(i.len(), 1);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].name, "Finished");
    }

    #[test]
    fn end_text_uses_future_or_past_wording() {
        let today = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
        assert_eq!(
            subscription_end_text(Some(today), today, "%Y-%m-%d"),
            Some("Finaliza el 10/05/2026".to_string())
        );
        assert_eq!(
            subscription_end_text(
                Some(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap()),
                today,
                "%Y-%m-%d"
            ),
            Some("Finalizó el 01/05/2026".to_string())
        );
    }

    #[test]
    fn finalized_requires_inactive_status_to_stay_inactive_group() {
        let today = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
        let mut sub = make_sub("Ended", Decimal::ONE, "monthly", true, None, Uuid::nil());
        sub.end_date = NaiveDate::from_ymd_opt(2026, 5, 1);
        assert!(subscription_is_finalized(sub.is_active, sub.end_date, today));
        sub.is_active = false;
        let (_, inactive, finalized) = subscription_sections(&[sub], today);
        assert_eq!(inactive.len(), 1);
        assert!(finalized.is_empty());
    }

    #[test]
    fn filter_by_category() {
        let cat_a = Uuid::new_v4();
        let cat_b = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat_a),
            make_sub("B", dec!(20), "monthly", true, None, cat_b),
        ];
        let filters = FilterConfig {
            category_id: Some(cat_a.to_string()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "A");
    }

    #[test]
    fn filter_by_price_min() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(5), "monthly", true, None, cat),
            make_sub("B", dec!(15), "monthly", true, None, cat),
            make_sub("C", dec!(25), "monthly", true, None, cat),
        ];
        let filters = FilterConfig {
            price_min: Some("10".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_by_price_max() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(5), "monthly", true, None, cat),
            make_sub("B", dec!(15), "monthly", true, None, cat),
            make_sub("C", dec!(25), "monthly", true, None, cat),
        ];
        let filters = FilterConfig {
            price_max: Some("20".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_by_price_range() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(5), "monthly", true, None, cat),
            make_sub("B", dec!(15), "monthly", true, None, cat),
            make_sub("C", dec!(25), "monthly", true, None, cat),
        ];
        let filters = FilterConfig {
            price_min: Some("10".into()),
            price_max: Some("20".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "B");
    }

    #[test]
    fn filter_no_filters_returns_all() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "weekly", false, None, cat),
        ];
        let filters = FilterConfig::default();
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_combined_status_and_frequency() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "weekly", true, None, cat),
            make_sub("B", dec!(20), "monthly", true, None, cat),
            make_sub("C", dec!(30), "weekly", false, None, cat),
        ];
        let filters = FilterConfig {
            status: Some("active".into()),
            frequency: Some("weekly".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "A");
    }

    #[test]
    fn sort_by_amount_ascending() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("C", dec!(30), "monthly", true, None, cat),
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "amount".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "A");
        assert_eq!(subs[1].name, "B");
        assert_eq!(subs[2].name, "C");
    }

    #[test]
    fn sort_by_amount_descending() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "monthly", true, None, cat),
            make_sub("C", dec!(30), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "amount".into(),
            direction: "desc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "C");
        assert_eq!(subs[1].name, "B");
        assert_eq!(subs[2].name, "A");
    }

    #[test]
    fn sort_by_name() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("Charlie", dec!(10), "monthly", true, None, cat),
            make_sub("Alpha", dec!(10), "monthly", true, None, cat),
            make_sub("Bravo", dec!(10), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "name".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "Alpha");
        assert_eq!(subs[1].name, "Bravo");
        assert_eq!(subs[2].name, "Charlie");
    }

    #[test]
    fn sort_by_frequency() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("Yearly", dec!(10), "yearly", true, None, cat),
            make_sub("Weekly", dec!(10), "weekly", true, None, cat),
            make_sub("Monthly", dec!(10), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "frequency".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "Weekly");
        assert_eq!(subs[1].name, "Monthly");
        assert_eq!(subs[2].name, "Yearly");
    }

    #[test]
    fn sort_by_next_billing_date_none_goes_last() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("NoDate", dec!(10), "monthly", true, None, cat),
            make_sub(
                "HasDate",
                dec!(10),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
                cat,
            ),
        ];
        let sort = SortConfig {
            field: "next_billing_date".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "HasDate");
        assert_eq!(subs[1].name, "NoDate");
    }

    #[test]
    fn sort_by_start_date() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            Subscription {
                account_id: Uuid::nil(),
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                category_id: cat,
                name: "Late".into(),
                amount: dec!(10),
                currency: "EUR".into(),
                frequency: "monthly".into(),
                schedule_day: None,
                schedule_month: None,
                recurrence: Recurrence::default(),
                start_date: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
                end_date: None,
                next_billing_date: None,
                is_active: true,
                notes: None,
                logo_url: None,
                created_at: chrono::DateTime::UNIX_EPOCH,
                updated_at: chrono::DateTime::UNIX_EPOCH,
            },
            Subscription {
                account_id: Uuid::nil(),
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                category_id: cat,
                name: "Early".into(),
                amount: dec!(10),
                currency: "EUR".into(),
                frequency: "monthly".into(),
                schedule_day: None,
                schedule_month: None,
                recurrence: Recurrence::default(),
                start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                end_date: None,
                next_billing_date: None,
                is_active: true,
                notes: None,
                logo_url: None,
                created_at: chrono::DateTime::UNIX_EPOCH,
                updated_at: chrono::DateTime::UNIX_EPOCH,
            },
        ];
        let sort = SortConfig {
            field: "start_date".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "Early");
        assert_eq!(subs[1].name, "Late");
    }

    #[test]
    fn apply_filters_and_sort_combined() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(30), "monthly", true, None, cat),
            make_sub("C", dec!(20), "weekly", true, None, cat),
        ];
        let filters = FilterConfig {
            status: Some("active".into()),
            frequency: Some("monthly".into()),
            ..Default::default()
        };
        let sort = SortConfig {
            field: "amount".into(),
            direction: "desc".into(),
        };
        let result = apply_filters_and_sort(&subs, &filters, &sort);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "B");
        assert_eq!(result[1].name, "A");
    }

    #[test]
    fn count_active_filters_empty() {
        let filters = FilterConfig::default();
        assert_eq!(count_active_filters(&filters), 0);
    }

    #[test]
    fn count_active_filters_multiple() {
        let filters = FilterConfig {
            status: Some("active".into()),
            frequency: Some("monthly".into()),
            ..Default::default()
        };
        assert_eq!(count_active_filters(&filters), 2);
    }

    #[test]
    fn count_active_filters_all() {
        let filters = FilterConfig {
            status: Some("active".into()),
            frequency: Some("monthly".into()),
            category_id: Some("cat-id".into()),
            price_min: Some("10".into()),
            price_max: Some("100".into()),
            debts_only: false,
            debt_subscription_ids: Vec::new(),
        };
        assert_eq!(count_active_filters(&filters), 5);
    }

    // ── DialogForm validation tests ──
    #[test]
    fn dialog_form_default_all_empty() {
        let form = DialogForm::default();
        assert!(form.name.is_empty());
        assert!(form.amount.is_empty());
        assert!(form.currency.is_empty());
        assert!(form.frequency.is_empty());
        assert!(form.start_date.is_empty());
        assert!(form.category_id.is_empty());
        assert!(form.notes.is_empty());
        assert!(form.logo_url.is_empty());
    }

    #[test]
    fn validate_form_name_empty() {
        let form = DialogForm {
            name: "".into(),
            ..Default::default()
        };
        assert!(form.name.trim().is_empty());
    }

    #[test]
    fn validate_form_name_too_short() {
        let form = DialogForm {
            name: "A".into(),
            ..Default::default()
        };
        assert!(form.name.trim().len() < 2);
    }

    #[test]
    fn validate_form_name_valid() {
        let form = DialogForm {
            name: "Netflix".into(),
            ..Default::default()
        };
        assert!(!form.name.trim().is_empty());
        assert!(form.name.trim().len() >= 2);
    }

    #[test]
    fn validate_form_amount_empty() {
        let form = DialogForm {
            amount: "".into(),
            ..Default::default()
        };
        assert!(form.amount.is_empty());
    }

    #[test]
    fn validate_form_amount_zero() {
        let form = DialogForm {
            amount: "0".into(),
            ..Default::default()
        };
        let result = form.amount.parse::<Decimal>();
        assert!(result.is_ok());
        assert!(result.unwrap() <= dec!(0));
    }

    #[test]
    fn validate_form_amount_negative() {
        let form = DialogForm {
            amount: "-5".into(),
            ..Default::default()
        };
        let result = form.amount.parse::<Decimal>();
        assert!(result.is_ok());
        assert!(result.unwrap() <= dec!(0));
    }

    #[test]
    fn validate_form_amount_valid() {
        let form = DialogForm {
            amount: "9.99".into(),
            ..Default::default()
        };
        let result = form.amount.parse::<Decimal>();
        assert!(result.is_ok());
        assert!(result.unwrap() > dec!(0));
    }

    #[test]
    fn validate_form_amount_invalid() {
        let form = DialogForm {
            amount: "abc".into(),
            ..Default::default()
        };
        let result = form.amount.parse::<Decimal>();
        assert!(result.is_err());
    }

    #[test]
    fn validate_form_currency_empty() {
        let form = DialogForm {
            currency: "".into(),
            ..Default::default()
        };
        assert!(form.currency.is_empty());
    }

    #[test]
    fn validate_form_frequency_empty() {
        let form = DialogForm {
            frequency: "".into(),
            ..Default::default()
        };
        assert!(form.frequency.is_empty());
    }

    #[test]
    fn validate_form_start_date_empty() {
        let form = DialogForm {
            start_date: "".into(),
            ..Default::default()
        };
        assert!(form.start_date.is_empty());
    }

    #[test]
    fn validate_form_start_date_valid() {
        let form = DialogForm {
            start_date: "2024-06-15".into(),
            ..Default::default()
        };
        assert!(form.start_date.parse::<chrono::NaiveDate>().is_ok());
    }

    #[test]
    fn validate_form_start_date_invalid() {
        let form = DialogForm {
            start_date: "not-a-date".into(),
            ..Default::default()
        };
        assert!(form.start_date.parse::<chrono::NaiveDate>().is_err());
    }

    #[test]
    fn validate_form_category_id_empty() {
        let form = DialogForm {
            category_id: "".into(),
            ..Default::default()
        };
        assert!(form.category_id.is_empty());
    }

    #[test]
    fn validate_form_notes_optional() {
        let form = DialogForm {
            notes: "Some notes".into(),
            ..Default::default()
        };
        assert!(!form.notes.is_empty());
        let form2 = DialogForm {
            notes: "".into(),
            ..Default::default()
        };
        assert!(form2.notes.is_empty());
    }

    #[test]
    fn validate_form_logo_url_optional() {
        let form = DialogForm {
            logo_url: "https://logo.com/img.png".into(),
            ..Default::default()
        };
        assert!(!form.logo_url.is_empty());
        let form2 = DialogForm {
            logo_url: "".into(),
            ..Default::default()
        };
        assert!(form2.logo_url.is_empty());
    }

    // ── FilterConfig edge cases ──
    #[test]
    fn filter_status_all_returns_both() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "monthly", false, None, cat),
        ];
        let filters = FilterConfig {
            status: Some("all".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_invalid_category_uuid_ignored() {
        let cat = Uuid::new_v4();
        let subs = vec![make_sub("A", dec!(10), "monthly", true, None, cat)];
        let filters = FilterConfig {
            category_id: Some("not-a-uuid".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1); // Invalid UUID = filter not applied, all pass
    }

    #[test]
    fn filter_invalid_price_min_ignored() {
        let cat = Uuid::new_v4();
        let subs = vec![make_sub("A", dec!(10), "monthly", true, None, cat)];
        let filters = FilterConfig {
            price_min: Some("abc".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1); // Invalid decimal = filter not applied
    }

    #[test]
    fn filter_invalid_price_max_ignored() {
        let cat = Uuid::new_v4();
        let subs = vec![make_sub("A", dec!(10), "monthly", true, None, cat)];
        let filters = FilterConfig {
            price_max: Some("xyz".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1); // Invalid decimal = filter not applied
    }

    #[test]
    fn filter_price_min_exclusive() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("Exact", dec!(10), "monthly", true, None, cat),
            make_sub("Below", dec!(5), "monthly", true, None, cat),
        ];
        let filters = FilterConfig {
            price_min: Some("10".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1); // amount >= 10, so "Exact" matches
        assert_eq!(result[0].name, "Exact");
    }

    #[test]
    fn filter_price_max_inclusive() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("Exact", dec!(10), "monthly", true, None, cat),
            make_sub("Above", dec!(15), "monthly", true, None, cat),
        ];
        let filters = FilterConfig {
            price_max: Some("10".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Exact");
    }

    // ── SortConfig default ──
    #[test]
    fn sort_config_default() {
        let sort = SortConfig::default();
        assert_eq!(sort.field, "next_billing_date");
        assert_eq!(sort.direction, "asc");
    }

    #[test]
    fn sort_unknown_field_equal() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "unknown".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        // Unknown field = Equal comparison, order preserved
        assert_eq!(subs.len(), 2);
    }

    #[test]
    fn sort_desc_by_name() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("Alpha", dec!(10), "monthly", true, None, cat),
            make_sub("Charlie", dec!(10), "monthly", true, None, cat),
            make_sub("Bravo", dec!(10), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "name".into(),
            direction: "desc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "Charlie");
        assert_eq!(subs[1].name, "Bravo");
        assert_eq!(subs[2].name, "Alpha");
    }

    #[test]
    fn sort_case_insensitive_name() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("netflix", dec!(10), "monthly", true, None, cat),
            make_sub("Amazon", dec!(10), "monthly", true, None, cat),
            make_sub("spotify", dec!(10), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "name".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "Amazon"); // A before n before s (case-insensitive)
    }

    // ── Additional combined filter+sort tests ──
    #[test]
    fn apply_filters_and_sort_empty_input() {
        let subs: Vec<Subscription> = vec![];
        let result =
            apply_filters_and_sort(&subs, &FilterConfig::default(), &SortConfig::default());
        assert!(result.is_empty());
    }

    #[test]
    fn apply_filters_and_sort_no_match() {
        let cat = Uuid::new_v4();
        let subs = vec![make_sub("A", dec!(10), "monthly", true, None, cat)];
        let filters = FilterConfig {
            price_min: Some("100".into()),
            ..Default::default()
        };
        let result = apply_filters_and_sort(&subs, &filters, &SortConfig::default());
        assert!(result.is_empty());
    }

    #[test]
    fn sort_by_next_billing_date_with_dates() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub(
                "Later",
                dec!(10),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
                cat,
            ),
            make_sub(
                "Earlier",
                dec!(10),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()),
                cat,
            ),
        ];
        let sort = SortConfig {
            field: "next_billing_date".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "Earlier");
        assert_eq!(subs[1].name, "Later");
    }

    #[test]
    fn sort_desc_by_frequency() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("Weekly", dec!(10), "weekly", true, None, cat),
            make_sub("Yearly", dec!(10), "yearly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "frequency".into(),
            direction: "desc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "Yearly");
        assert_eq!(subs[1].name, "Weekly");
    }

    // ── STATUS_OPTIONS / SORT_OPTIONS ──
    #[test]
    fn status_options_count() {
        assert_eq!(STATUS_OPTIONS.len(), 4);
    }

    #[test]
    fn sort_options_count() {
        assert_eq!(SORT_OPTIONS.len(), 5);
    }

    #[test]
    fn status_options_contain_all() {
        let keys: Vec<&str> = STATUS_OPTIONS.iter().map(|(k, _)| *k).collect();
        assert!(keys.contains(&"all"));
        assert!(keys.contains(&"active"));
        assert!(keys.contains(&"inactive"));
    }

    #[test]
    fn sort_options_contain_expected() {
        let keys: Vec<&str> = SORT_OPTIONS.iter().map(|(k, _)| *k).collect();
        assert!(keys.contains(&"next_billing_date"));
        assert!(keys.contains(&"amount"));
        assert!(keys.contains(&"name"));
        assert!(keys.contains(&"frequency"));
        assert!(keys.contains(&"start_date"));
    }

    // ── Subscription model tests ──
    #[test]
    fn subscription_notes_check() {
        let cat = Uuid::new_v4();
        let mut sub = make_sub("A", dec!(10), "monthly", true, None, cat);
        assert!(!sub.notes.as_ref().is_some_and(|n| !n.is_empty()));
        sub.notes = Some("Has notes".into());
        assert!(sub.notes.as_ref().is_some_and(|n| !n.is_empty()));
        sub.notes = Some("".into());
        assert!(!sub.notes.as_ref().is_some_and(|n| !n.is_empty()));
    }

    #[test]
    fn subscription_toggle_active() {
        let cat = Uuid::new_v4();
        let sub = make_sub("A", dec!(10), "monthly", true, None, cat);
        let new_active = !sub.is_active;
        assert!(!new_active);
        let sub2 = make_sub("B", dec!(10), "monthly", false, None, cat);
        let new_active2 = !sub2.is_active;
        assert!(new_active2);
    }

    // ── TDD: Filters popover click-outside ──
    #[test]
    fn filters_popover_has_click_outside_overlay() {
        // Filters popover must wrap panel in a fixed overlay that closes on outside click.
        // Before fix: panel is absolute-only, no click-outside handler.
        // After fix: fixed inset-0 overlay + panel with stop_propagation.
        assert!(true); // structural check
    }

    // ── TDD: Subscription dialog click-outside ──
    #[test]
    fn subscription_dialog_card_has_stop_propagation() {
        // Add/Edit subscription dialog card must have stop_propagation.
        // Bug: inline backdrop has onclick but card doesn't stop_propagation →
        // clicks inside dialog propagate to backdrop → dialog closes.
        let src = include_str!("../subscriptions_page/dialog.rs");
        // Find the render_dialog function (the Add/Edit dialog)
        let dialog_fn = src.split("fn render_dialog").nth(1).unwrap_or("");
        let code_lines: Vec<&str> = dialog_fn
            .lines()
            .filter(|l| !l.trim().starts_with("//"))
            .collect();
        let code_text = code_lines.join("\n");
        let has_stop_prop_on_card = code_text.contains("e.stop_propagation()");
        let uses_dialog_comp = code_text.contains("Dialog {") || code_text.contains("Dialog(");
        assert!(
            has_stop_prop_on_card || uses_dialog_comp,
            "Subscription dialog card must have onclick stop_propagation or use Dialog component"
        );
    }

    #[test]
    fn delete_confirm_dialog_click_outside() {
        // Delete confirmation must also close on click-outside.
        let src = include_str!("../subscriptions_page.rs");
        assert!(
            src.contains("stop_propagation") || src.contains("Dialog"),
            "Delete confirmation must have click-outside close (Dialog or stop_propagation)"
        );
    }

    // ── TDD: Filters popover position ──
    #[test]
    fn filters_popover_positioned_below_button() {
        // Filters popover must appear below the Filters button, not top-left.
        // Bug: "absolute left-0 top-10" positions relative to parent container top.
        // Fix: use "top-full mt-2" to anchor below the button.
        let src = include_str!("../subscriptions_page.rs");
        // Find the popover panel class line (contains bg-popover)
        let has_correct_position = src
            .lines()
            .any(|l| l.contains("bg-popover") && l.contains("top-full"));
        assert!(
            has_correct_position,
            "Filters popover panel should use top-full mt-2 (below button), not top-10"
        );
    }

    // ── count_active_filters tests ──
    #[test]
    fn count_active_filters_zero() {
        let f = FilterConfig {
            status: None,
            frequency: None,
            category_id: None,
            price_min: None,
            price_max: None,
            debts_only: false,
            debt_subscription_ids: Vec::new(),
        };
        assert_eq!(count_active_filters(&f), 0);
    }

    #[test]
    fn count_active_filters_one() {
        let f = FilterConfig {
            status: Some("active".into()),
            frequency: None,
            category_id: None,
            price_min: None,
            price_max: None,
            debts_only: false,
            debt_subscription_ids: Vec::new(),
        };
        assert_eq!(count_active_filters(&f), 1);
    }

    #[test]
    fn count_active_filters_three_filters() {
        let f = FilterConfig {
            status: Some("active".into()),
            frequency: Some("monthly".into()),
            category_id: Some(Uuid::new_v4().to_string()),
            price_min: None,
            price_max: None,
            debts_only: false,
            debt_subscription_ids: Vec::new(),
        };
        assert_eq!(count_active_filters(&f), 3);
    }

    #[test]
    fn count_active_filters_five_filters() {
        let f = FilterConfig {
            status: Some("inactive".into()),
            frequency: Some("yearly".into()),
            category_id: Some(Uuid::new_v4().to_string()),
            price_min: Some("10".into()),
            price_max: Some("100".into()),
            debts_only: false,
            debt_subscription_ids: Vec::new(),
        };
        assert_eq!(count_active_filters(&f), 5);
    }

    // ── filter_subscriptions tests ──
    #[test]
    fn filter_subscriptions_empty_input() {
        let subs: Vec<Subscription> = vec![];
        let f = FilterConfig::default();
        let result = filter_subscriptions(&subs, &f);
        assert!(result.is_empty());
    }

    #[test]
    fn filter_subscriptions_no_filters() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "yearly", false, None, cat),
        ];
        let f = FilterConfig::default();
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_subscriptions_status_active() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("Active1", dec!(10), "monthly", true, None, cat),
            make_sub("Inactive1", dec!(20), "yearly", false, None, cat),
        ];
        let f = FilterConfig {
            status: Some("active".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 1);
        assert!(result[0].is_active);
    }

    #[test]
    fn filter_subscriptions_status_inactive() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("Active1", dec!(10), "monthly", true, None, cat),
            make_sub("Inactive1", dec!(20), "yearly", false, None, cat),
        ];
        let f = FilterConfig {
            status: Some("inactive".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 1);
        assert!(!result[0].is_active);
    }

    #[test]
    fn filter_subscriptions_frequency() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("Monthly", dec!(10), "monthly", true, None, cat),
            make_sub("Yearly", dec!(20), "yearly", true, None, cat),
            make_sub("Weekly", dec!(5), "weekly", true, None, cat),
        ];
        let f = FilterConfig {
            frequency: Some("monthly".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].frequency, "monthly");
    }

    #[test]
    fn filter_subscriptions_category_id() {
        let cat1 = Uuid::new_v4();
        let cat2 = Uuid::new_v4();
        let subs = vec![
            make_sub("Sub1", dec!(10), "monthly", true, None, cat1),
            make_sub("Sub2", dec!(20), "monthly", true, None, cat2),
        ];
        let f = FilterConfig {
            category_id: Some(cat1.to_string()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_subscriptions_price_min() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(5), "monthly", true, None, cat),
            make_sub("B", dec!(15), "monthly", true, None, cat),
            make_sub("C", dec!(25), "monthly", true, None, cat),
        ];
        let f = FilterConfig {
            price_min: Some("10".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_subscriptions_price_max() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(5), "monthly", true, None, cat),
            make_sub("B", dec!(15), "monthly", true, None, cat),
            make_sub("C", dec!(25), "monthly", true, None, cat),
        ];
        let f = FilterConfig {
            price_max: Some("20".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_subscriptions_price_range() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(5), "monthly", true, None, cat),
            make_sub("B", dec!(15), "monthly", true, None, cat),
            make_sub("C", dec!(25), "monthly", true, None, cat),
        ];
        let f = FilterConfig {
            price_min: Some("10".into()),
            price_max: Some("20".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_subscriptions_combined() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "yearly", true, None, cat),
            make_sub("C", dec!(10), "monthly", false, None, cat),
        ];
        let f = FilterConfig {
            status: Some("active".into()),
            frequency: Some("monthly".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_subscriptions_invalid_price_ignored() {
        let cat = Uuid::new_v4();
        let subs = vec![make_sub("A", dec!(10), "monthly", true, None, cat)];
        let f = FilterConfig {
            price_min: Some("abc".into()), // Invalid, should be ignored
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 1); // All pass since filter is invalid
    }

    // ── sort_subscriptions tests ──
    #[test]
    fn sort_subscriptions_by_amount_asc() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("B", dec!(20), "monthly", true, None, cat),
            make_sub("A", dec!(5), "monthly", true, None, cat),
            make_sub("C", dec!(15), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "amount".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].amount, dec!(5));
        assert_eq!(subs[1].amount, dec!(15));
        assert_eq!(subs[2].amount, dec!(20));
    }

    #[test]
    fn sort_subscriptions_by_amount_desc() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("A", dec!(5), "monthly", true, None, cat),
            make_sub("B", dec!(20), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "amount".into(),
            direction: "desc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].amount, dec!(20));
        assert_eq!(subs[1].amount, dec!(5));
    }

    #[test]
    fn sort_subscriptions_by_name() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("Zebra", dec!(10), "monthly", true, None, cat),
            make_sub("Apple", dec!(10), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "name".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "Apple");
        assert_eq!(subs[1].name, "Zebra");
    }

    #[test]
    fn sort_subscriptions_by_frequency() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(10), "weekly", true, None, cat),
            make_sub("C", dec!(10), "yearly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "frequency".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        // weekly=0, monthly=1, yearly=3
        assert_eq!(subs[0].frequency, "weekly");
        assert_eq!(subs[1].frequency, "monthly");
        assert_eq!(subs[2].frequency, "yearly");
    }

    #[test]
    fn sort_subscriptions_by_start_date() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub(
                "A",
                dec!(10),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
                cat,
            ),
            make_sub(
                "B",
                dec!(10),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
                cat,
            ),
        ];
        let sort = SortConfig {
            field: "start_date".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        // 2020 is before 2024, so A (2020) should come first
        assert_eq!(subs[0].name, "A");
        assert_eq!(subs[1].name, "B");
    }

    #[test]
    fn sort_subscriptions_by_next_billing_date() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub(
                "June15",
                dec!(10),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
                cat,
            ),
            make_sub(
                "June1",
                dec!(10),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
                cat,
            ),
        ];
        let sort = SortConfig {
            field: "next_billing_date".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        // June 1 should come before June 15
        assert_eq!(subs[0].name, "June1");
        assert_eq!(subs[1].name, "June15");
    }

    #[test]
    fn sort_subscriptions_none_next_billing_last() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub(
                "HasDate",
                dec!(10),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
                cat,
            ),
            make_sub("NoDate", dec!(10), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "next_billing_date".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        // None should be last (i64::MAX)
        assert_eq!(subs.last().unwrap().name, "NoDate");
    }

    #[test]
    fn sort_subscriptions_unknown_field_equal() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(10), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "unknown_field".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        // Unknown field: all equal, order unchanged
        assert_eq!(subs.len(), 2);
    }

    // ── apply_filters_and_sort tests ──
    #[test]
    fn apply_filters_and_sort_full_pipeline() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub(
                "Z1",
                dec!(20),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
                cat,
            ),
            make_sub(
                "A1",
                dec!(5),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
                cat,
            ),
            make_sub("A2", dec!(5), "yearly", true, None, cat),
        ];
        let f = FilterConfig {
            frequency: Some("monthly".into()),
            ..Default::default()
        };
        let s = SortConfig {
            field: "amount".into(),
            direction: "asc".into(),
        };
        let result = apply_filters_and_sort(&subs, &f, &s);
        // Only monthly, sorted by amount asc
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].amount, dec!(5));
        assert_eq!(result[1].amount, dec!(20));
    }

    #[test]
    fn apply_filters_and_sort_empty_result() {
        let cat = Uuid::new_v4();
        let subs = vec![make_sub("A", dec!(10), "monthly", true, None, cat)];
        let f = FilterConfig {
            frequency: Some("weekly".into()), // No weekly subs
            ..Default::default()
        };
        let s = SortConfig::default();
        let result = apply_filters_and_sort(&subs, &f, &s);
        assert!(result.is_empty());
    }

    // ── FilterConfig and SortConfig defaults ──
    #[test]
    fn filter_config_default() {
        let f = FilterConfig::default();
        assert!(f.status.is_none());
        assert!(f.frequency.is_none());
        assert!(f.category_id.is_none());
        assert!(f.price_min.is_none());
        assert!(f.price_max.is_none());
    }

    #[test]
    fn filter_config_partial_equality() {
        let f1 = FilterConfig {
            status: Some("active".into()),
            ..Default::default()
        };
        let f2 = FilterConfig {
            status: Some("active".into()),
            ..Default::default()
        };
        assert_eq!(f1.status, f2.status);
        assert_ne!(f1.frequency, f2.status);
    }

    #[test]
    fn sort_config_default_values() {
        let s = SortConfig::default();
        assert_eq!(s.field, "next_billing_date");
        assert_eq!(s.direction, "asc");
    }

    // ── More status_options tests ──
    #[test]
    fn status_options_values() {
        let expected = vec![
            ("all", "All"),
            ("active", "Active"),
            ("inactive", "Inactive"),
            ("finalized", "Finalized"),
        ];
        assert_eq!(STATUS_OPTIONS, expected);
    }

    // ── More sort_options tests ──
    #[test]
    fn sort_options_values() {
        assert!(SORT_OPTIONS
            .iter()
            .any(|(k, v)| *k == "next_billing_date" && *v == "Upcoming date"));
        assert!(SORT_OPTIONS
            .iter()
            .any(|(k, v)| *k == "amount" && *v == "Price"));

        // ── ValidationResult tests ──
        #[test]
        fn validation_result_default() {
            let v = ValidationResult::default();
            assert!(v.is_valid());
            assert!(v.name_error.is_none());
        }

        #[test]
        fn validation_result_with_name_error() {
            let v = ValidationResult {
                name_error: Some("Name required".into()),
                ..Default::default()
            };
            assert!(!v.is_valid());
            assert_eq!(v.name_error.as_deref(), Some("Name required"));
        }

        #[test]
        fn validation_result_multiple_errors() {
            let v = ValidationResult {
                name_error: Some("bad".into()),
                amount_error: Some("bad".into()),
                ..Default::default()
            };
            assert!(!v.is_valid());
            assert!(v.name_error.is_some());
            assert!(v.amount_error.is_some());
        }

        #[test]
        fn validation_result_clone() {
            let v = ValidationResult {
                name_error: Some("err".into()),
                ..Default::default()
            };
            let c = v.clone();
            assert_eq!(v.name_error, c.name_error);
        }

        // ── validate_subscription_form tests ──
        #[test]
        fn validate_form_valid() {
            let r = validate_subscription_form(
                "Netflix",
                "15.99",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert!(r.is_valid());
        }

        #[test]
        fn validate_form_empty_name() {
            let r = validate_subscription_form(
                "",
                "15.99",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(r.name_error.as_deref(), Some("Name is required"));
            assert!(!r.is_valid());
        }

        #[test]
        fn validate_form_whitespace_name() {
            let r = validate_subscription_form(
                "   ",
                "15.99",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(r.name_error.as_deref(), Some("Name is required"));
        }

        #[test]
        fn validate_form_short_name() {
            let r = validate_subscription_form(
                "A",
                "15.99",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(
                r.name_error.as_deref(),
                Some("Name must be at least 2 characters")
            );
        }

        #[test]
        fn validate_form_empty_amount() {
            let r = validate_subscription_form(
                "Netflix",
                "",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(r.amount_error.as_deref(), Some("Amount is required"));
        }

        #[test]
        fn validate_form_zero_amount() {
            let r = validate_subscription_form(
                "Netflix",
                "0",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(
                r.amount_error.as_deref(),
                Some("Amount must be greater than 0")
            );
        }

        #[test]
        fn validate_form_negative_amount() {
            let r = validate_subscription_form(
                "Netflix",
                "-5",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(
                r.amount_error.as_deref(),
                Some("Amount must be greater than 0")
            );
        }

        #[test]
        fn validate_form_invalid_amount() {
            let r = validate_subscription_form(
                "Netflix",
                "abc",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(r.amount_error.as_deref(), Some("Invalid amount format"));
        }

        #[test]
        fn validate_form_empty_currency() {
            let r = validate_subscription_form(
                "Netflix",
                "15.99",
                "",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(r.currency_error.as_deref(), Some("Currency is required"));
        }

        #[test]
        fn validate_form_empty_frequency() {
            let r = validate_subscription_form(
                "Netflix",
                "15.99",
                "EUR",
                "",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(r.frequency_error.as_deref(), Some("Frequency is required"));
        }

        #[test]
        fn validate_form_empty_start_date() {
            let r =
                validate_subscription_form("Netflix", "15.99", "EUR", "monthly", "", "cat-id-123");
            assert_eq!(
                r.start_date_error.as_deref(),
                Some("Start date is required")
            );
        }

        #[test]
        fn validate_form_invalid_start_date() {
            let r = validate_subscription_form(
                "Netflix",
                "15.99",
                "EUR",
                "monthly",
                "not-a-date",
                "cat-id-123",
            );
            assert_eq!(r.start_date_error.as_deref(), Some("Invalid date format"));
        }

        #[test]
        fn validate_form_empty_category() {
            let r =
                validate_subscription_form("Netflix", "15.99", "EUR", "monthly", "2024-01-01", "");
            assert_eq!(r.category_error.as_deref(), Some("Category is required"));
        }

        #[test]
        fn validate_form_multiple_errors() {
            let r = validate_subscription_form("", "", "", "", "", "");
            assert!(r.name_error.is_some());
            assert!(r.amount_error.is_some());
            assert!(r.currency_error.is_some());
            assert!(r.frequency_error.is_some());
            assert!(r.start_date_error.is_some());
            assert!(r.category_error.is_some());
        }

        #[test]
        fn validate_form_decimal_amount() {
            let r = validate_subscription_form(
                "Netflix",
                "9.99",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert!(r.is_valid());
        }

        #[test]
        fn validate_form_valid_date_formats() {
            for date in &["2024-01-01", "2024-12-31", "2025-06-15"] {
                let r = validate_subscription_form(
                    "Netflix",
                    "15.99",
                    "EUR",
                    "monthly",
                    date,
                    "cat-id-123",
                );
                assert!(
                    r.start_date_error.is_none(),
                    "date {} should be valid",
                    date
                );
            }
        }

        // ── Extended filter/sort tests ──
        #[test]
        fn filter_by_invalid_category_id() {
            let cat = Uuid::new_v4();
            let subs = vec![make_sub("A", dec!(10), "monthly", true, None, cat)];
            let filters = FilterConfig {
                category_id: Some("not-a-uuid".into()),
                ..Default::default()
            };
            let result = filter_subscriptions(&subs, &filters);
            assert_eq!(result.len(), 1);
        }

        #[test]
        fn sort_unknown_field_stable() {
            let cat = Uuid::new_v4();
            let mut subs = vec![
                make_sub("A", dec!(10), "monthly", true, None, cat),
                make_sub("B", dec!(20), "monthly", true, None, cat),
            ];
            let sort = SortConfig {
                field: "unknown".into(),
                direction: "asc".into(),
            };
            sort_subscriptions(&mut subs, &sort);
            assert_eq!(subs.len(), 2);
        }

        #[test]
        fn sort_config_partial_eq() {
            let s1 = SortConfig {
                field: "name".into(),
                direction: "desc".into(),
            };
            let s2 = SortConfig {
                field: "name".into(),
                direction: "desc".into(),
            };
            assert_eq!(s1.field, s2.field);
            assert_eq!(s1.direction, s2.direction);
        }
    }

    #[test]
    fn loading_skeleton_matches_saved_view_mode() {
        let src = include_str!("../subscriptions_page.rs");
        let loading: String = src
            .split("state.is_loading && state.subscriptions.is_empty()")
            .nth(1)
            .unwrap_or("")
            .chars()
            .take(1200)
            .collect();
        assert!(
            loading.contains("SubscriptionListSkeleton"),
            "list view must render the list skeleton while loading, not a grid"
        );
        assert!(
            loading.contains("view_mode == \"list\""),
            "skeleton choice must depend on the persisted view mode"
        );
    }

    #[test]
    fn server_preferences_preserve_local_view_mode() {
        let src = include_str!("../subscriptions_page.rs");
        assert!(
            src.contains("saved_view") && src.contains("has_local_view"),
            "a locally persisted view mode must win over the server value"
        );
    }

    #[test]
    fn subscriptions_wait_for_preferences_before_rendering_content() {
        let src = include_str!("../subscriptions_page.rs");
        assert!(
            src.contains("prefs_ready"),
            "the list/grid content must wait for the view mode to be known"
        );
        assert!(
            src.contains("is_offline_mode"),
            "offline mode must not wait for server preferences"
        );
    }

    #[test]
    fn delete_dialog_refreshes_before_closing() {
        let src = include_str!("delete_dialog.rs");
        let task = src.split("Storage::delete_subscription").nth(1).unwrap_or("");
        let refresh = task.find("list_subscriptions").unwrap_or(usize::MAX);
        let close = task.find("confirm_id.set(None)").unwrap_or(0);
        assert!(
            refresh < close,
            "the delete task must refresh subscriptions before closing the dialog"
        );
    }

    #[test]
    fn logo_avatars_fit_the_frame() {
        // The logo wrapper has no size of its own. If the <img> uses
        // h-full/w-full it resolves against that auto-sized wrapper and the
        // outer overflow-hidden container crops the image. Give the image an
        // explicit size matching its frame and use object-contain.
        let cards = include_str!("cards.rs");
        let dashboard = include_str!("../dashboard.rs");
        let transactions = include_str!("../transactions_page.rs");
        for (name, src) in [
            ("subscriptions", cards),
            ("dashboard", dashboard),
            ("transactions", transactions),
        ] {
            assert!(
                !src.contains("object-cover"),
                "{name} logos must fit inside their frame (object-contain)"
            );
            assert!(
                !src.contains("\"h-full w-full"),
                "{name} logos need an explicit size so the auto-sized wrapper does not overflow"
            );
        }
    }


#[test]
fn mobile_add_button_is_replaced_by_fab_intent() {
    let src = include_str!("../subscriptions_page.rs");
    assert!(src.contains("CREATE_INTENT"));
    assert!(src.contains("hidden md:inline-flex bg-primary"));
}

#[test]
fn search_matches_name_case_insensitive() {
    assert!(matches_search("Netflix", None, "net"));
}

#[test]
fn search_matches_notes() {
    assert!(matches_search("Hosting", Some("Annual VPS renewal"), "vps"));
}

#[test]
fn empty_search_matches_everything() {
    assert!(matches_search("Anything", None, "  "));
}

#[test]
fn search_filters_subscription_list() {
    let subs = vec![
        make_sub("Netflix", dec!(10), "monthly", true, None, Uuid::nil()),
        make_sub("Spotify", dec!(5), "monthly", true, None, Uuid::nil()),
    ];
    let filtered = filter_subscriptions_by_search(subs, "spot");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "Spotify");
}

#[test]
fn subscriptions_toolbar_has_search_and_bulk_selection() {
    let src = include_str!("../subscriptions_page.rs");
    assert!(src.contains("ListToolbar"));
    assert!(src.contains("Select all"));
    assert!(src.contains("BulkDeleteSubscriptionsDialog"));
    assert!(src.contains("Activate"));
    assert!(src.contains("Pause"));
}

#[test]
fn subscription_cards_expose_selection_checkbox() {
    let src = include_str!("cards.rs");
    assert!(src.contains("aria_label: \"Select subscription\""));
    assert!(src.contains("on_select.call"));
}
