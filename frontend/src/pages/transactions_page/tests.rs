
    use super::*;
use std::str::FromStr;
use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    fn make_txn(name: &str, amount: Decimal, date: NaiveDate, cat_id: Uuid) -> Transaction {
        Transaction {
            account_id: Uuid::nil(),
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: cat_id,
            name: name.into(),
            amount,
            currency: "EUR".into(),
            date,
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        }
    }

    // ── FilterConfig tests ──
    #[test]
    fn filter_config_default() {
        let f = FilterConfig::default();
        assert!(f.category_id.is_none());
        assert!(f.price_min.is_none());
        assert!(f.price_max.is_none());
    }

    #[test]
    fn filter_config_partial_eq() {
        let f1 = FilterConfig {
            category_id: Some(Uuid::new_v4()),
            ..Default::default()
        };
        let f2 = FilterConfig {
            category_id: f1.category_id,
            ..Default::default()
        };
        assert_eq!(f1.category_id, f2.category_id);
    }

    #[test]
    fn filter_config_clone() {
        let f = FilterConfig::default();
        let cloned = f.clone();
        assert_eq!(f, cloned);
    }

    // ── SortConfig tests ──
    #[test]
    fn sort_config_debug() {
        let s = SortConfig {
            field: "amount".into(),
            direction: "asc".into(),
        };
        let debug = format!("{:?}", s);
        assert!(debug.contains("SortConfig"));
    }

    // ── MonthGroup tests ──
    #[test]
    fn month_group_debug() {
        let c = Uuid::new_v4();
        let group = MonthGroup {
            key: "2024-01".into(),
            label: "January 2024".into(),
            transactions: vec![make_txn(
                "Test",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            )],
            total: dec!(10),
        };
        let debug = format!("{:?}", group);
        assert!(debug.contains("MonthGroup"));
    }

    // ── TransactionValidationResult tests ──
    #[test]
    fn validation_result_default_is_valid() {
        let result = TransactionValidationResult::default();
        assert!(result.is_valid());
    }

    #[test]
    fn validation_result_with_name_error_invalid() {
        let mut result = TransactionValidationResult::default();
        result.name_error = Some("Name required".into());
        assert!(!result.is_valid());
    }

    #[test]
    fn validation_result_with_amount_error_invalid() {
        let mut result = TransactionValidationResult::default();
        result.amount_error = Some("Amount required".into());
        assert!(!result.is_valid());
    }

    #[test]
    fn validation_result_with_currency_error_invalid() {
        let mut result = TransactionValidationResult::default();
        result.currency_error = Some("Currency required".into());
        assert!(!result.is_valid());
    }

    #[test]
    fn validation_result_with_date_error_invalid() {
        let mut result = TransactionValidationResult::default();
        result.date_error = Some("Date required".into());
        assert!(!result.is_valid());
    }

    #[test]
    fn validation_result_with_category_error_invalid() {
        let mut result = TransactionValidationResult::default();
        result.category_error = Some("Category required".into());
        assert!(!result.is_valid());
    }

    #[test]
    fn validation_result_multiple_errors() {
        let mut result = TransactionValidationResult::default();
        result.name_error = Some("Name required".into());
        result.amount_error = Some("Amount required".into());
        assert!(!result.is_valid());
        assert!(result.name_error.is_some());
        assert!(result.amount_error.is_some());
    }

    #[test]
    fn validation_result_clone() {
        let mut result = TransactionValidationResult::default();
        result.name_error = Some("err".into());
        let cloned = result.clone();
        assert_eq!(result.name_error, cloned.name_error);
    }

    // ── validate_transaction_form tests ──
    #[test]
    fn validate_form_all_valid() {
        let result = validate_transaction_form("Netflix", "15.99", "EUR", "2024-01-01", "cat-id");
        assert!(result.is_valid());
    }

    #[test]
    fn validate_form_empty_name() {
        let result = validate_transaction_form("", "15.99", "EUR", "2024-01-01", "cat-id");
        assert!(!result.is_valid());
        assert!(result.name_error.is_some());
    }

    #[test]
    fn validate_form_whitespace_name() {
        let result = validate_transaction_form("   ", "15.99", "EUR", "2024-01-01", "cat-id");
        assert!(!result.is_valid());
        assert!(result.name_error.is_some());
    }

    #[test]
    fn validate_form_short_name() {
        let result = validate_transaction_form("A", "15.99", "EUR", "2024-01-01", "cat-id");
        assert!(!result.is_valid());
        assert!(result.name_error.is_some());
    }

    #[test]
    fn validate_form_empty_amount() {
        let result = validate_transaction_form("Netflix", "", "EUR", "2024-01-01", "cat-id");
        assert!(!result.is_valid());
        assert!(result.amount_error.is_some());
    }

    #[test]
    fn validate_form_zero_amount() {
        let result = validate_transaction_form("Netflix", "0", "EUR", "2024-01-01", "cat-id");
        assert!(!result.is_valid());
        assert!(result.amount_error.is_some());
    }

    #[test]
    fn validate_form_negative_amount() {
        let result = validate_transaction_form("Netflix", "-5", "EUR", "2024-01-01", "cat-id");
        assert!(!result.is_valid());
        assert!(result.amount_error.is_some());
    }

    #[test]
    fn validate_form_invalid_amount() {
        let result = validate_transaction_form("Netflix", "abc", "EUR", "2024-01-01", "cat-id");
        assert!(!result.is_valid());
        assert!(result.amount_error.is_some());
    }

    #[test]
    fn validate_form_empty_currency() {
        let result = validate_transaction_form("Netflix", "15.99", "", "2024-01-01", "cat-id");
        assert!(!result.is_valid());
        assert!(result.currency_error.is_some());
    }

    #[test]
    fn validate_form_empty_date() {
        let result = validate_transaction_form("Netflix", "15.99", "EUR", "", "cat-id");
        assert!(!result.is_valid());
        assert!(result.date_error.is_some());
    }

    #[test]
    fn validate_form_invalid_date() {
        let result = validate_transaction_form("Netflix", "15.99", "EUR", "not-a-date", "cat-id");
        assert!(!result.is_valid());
        assert!(result.date_error.is_some());
    }

    #[test]
    fn validate_form_empty_category() {
        let result = validate_transaction_form("Netflix", "15.99", "EUR", "2024-01-01", "");
        assert!(!result.is_valid());
        assert!(result.category_error.is_some());
    }

    #[test]
    fn validate_form_decimal_amount() {
        let result = validate_transaction_form("Netflix", "9.99", "EUR", "2024-01-01", "cat-id");
        assert!(result.is_valid());
    }

    #[test]
    fn validate_form_valid_dates() {
        for date in &["2024-01-01", "2024-12-31", "2025-06-15"] {
            let result = validate_transaction_form("Test", "10", "EUR", date, "cat-id");
            assert!(result.date_error.is_none(), "Date {} should be valid", date);
        }
    }

    // ── filter_sort_group tests ──
    #[test]
    fn filter_by_category() {
        let c1 = Uuid::new_v4();
        let c2 = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c1,
            ),
            make_txn(
                "B",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c2,
            ),
        ];
        let filters = FilterConfig {
            category_id: Some(c1),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].transactions.len(), 1);
        assert_eq!(groups[0].transactions[0].name, "A");
    }

    #[test]
    fn filter_by_price_range() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "Cheap",
                dec!(5),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "Mid",
                dec!(15),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
            make_txn(
                "Expensive",
                dec!(50),
                NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                c,
            ),
        ];
        let filters = FilterConfig {
            price_min: Some(dec!(10)),
            price_max: Some(dec!(30)),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert_eq!(groups[0].transactions.len(), 1);
        assert_eq!(groups[0].transactions[0].name, "Mid");
    }

    #[test]
    fn sort_by_amount_desc() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(30),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
            make_txn(
                "C",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                c,
            ),
        ];
        let sort = SortConfig {
            field: "amount".into(),
            direction: "desc".into(),
        };
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
        assert_eq!(groups[0].transactions[0].name, "B");
        assert_eq!(groups[0].transactions[1].name, "C");
        assert_eq!(groups[0].transactions[2].name, "A");
    }

    #[test]
    fn group_by_month_descending() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "Jan",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                c,
            ),
            make_txn(
                "Mar",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
                c,
            ),
            make_txn(
                "Feb",
                dec!(15),
                NaiveDate::from_ymd_opt(2024, 2, 10).unwrap(),
                c,
            ),
        ];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].key, "2024-03");
        assert_eq!(groups[1].key, "2024-02");
        assert_eq!(groups[2].key, "2024-01");
    }

    #[test]
    fn month_group_total() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                c,
            ),
        ];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups[0].total, dec!(30));
    }

    #[test]
    fn empty_filters_return_all() {
        let c = Uuid::new_v4();
        let txns = vec![make_txn(
            "X",
            dec!(10),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            c,
        )];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups[0].transactions.len(), 1);
    }

    #[test]
    fn format_month_labels() {
        assert_eq!(format_month(2024, 1), "January 2024");
        assert_eq!(format_month(2024, 6), "June 2024");
        assert_eq!(format_month(2024, 12), "December 2024");
    }

    // ── Additional filter/sort tests ──
    #[test]
    fn filter_by_price_min_only() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(5),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(15),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
        ];
        let filters = FilterConfig {
            price_min: Some(dec!(10)),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert_eq!(groups[0].transactions.len(), 1);
        assert_eq!(groups[0].transactions[0].name, "B");
    }

    #[test]
    fn filter_by_price_max_only() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(5),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(15),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
        ];
        let filters = FilterConfig {
            price_max: Some(dec!(10)),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert_eq!(groups[0].transactions.len(), 1);
        assert_eq!(groups[0].transactions[0].name, "A");
    }

    #[test]
    fn filter_by_category_and_price() {
        let c1 = Uuid::new_v4();
        let c2 = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(5),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c1,
            ),
            make_txn(
                "B",
                dec!(15),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c1,
            ),
            make_txn(
                "C",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                c2,
            ),
        ];
        let filters = FilterConfig {
            category_id: Some(c1),
            price_min: Some(dec!(10)),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert_eq!(groups[0].transactions.len(), 1);
        assert_eq!(groups[0].transactions[0].name, "B");
    }

    #[test]
    fn sort_by_name_ascending() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "Charlie",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "Alpha",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
        ];
        let sort = SortConfig {
            field: "name".into(),
            direction: "asc".into(),
        };
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
        assert_eq!(groups[0].transactions[0].name, "Alpha");
        assert_eq!(groups[0].transactions[1].name, "Charlie");
    }

    #[test]
    fn sort_by_amount_ascending() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "C",
                dec!(30),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                c,
            ),
        ];
        let sort = SortConfig {
            field: "amount".into(),
            direction: "asc".into(),
        };
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
        assert_eq!(groups[0].transactions[0].name, "A");
        assert_eq!(groups[0].transactions[1].name, "B");
        assert_eq!(groups[0].transactions[2].name, "C");
    }

    #[test]
    fn sort_by_date_descending() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                c,
            ),
        ];
        let sort = SortConfig {
            field: "date".into(),
            direction: "desc".into(),
        };
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
        assert_eq!(groups[0].transactions[0].name, "B");
        assert_eq!(groups[0].transactions[1].name, "A");
    }

    #[test]
    fn sort_unknown_field_defaults_to_date() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                c,
            ),
        ];
        let sort = SortConfig {
            field: "unknown".into(),
            direction: "asc".into(),
        };
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
        assert_eq!(groups[0].transactions[0].name, "A"); // sorted by date asc as default
    }

    #[test]
    fn groups_across_multiple_months() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "Jan1",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "Jan2",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                c,
            ),
            make_txn(
                "Feb",
                dec!(15),
                NaiveDate::from_ymd_opt(2024, 2, 10).unwrap(),
                c,
            ),
            make_txn(
                "Mar",
                dec!(25),
                NaiveDate::from_ymd_opt(2024, 3, 5).unwrap(),
                c,
            ),
        ];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups.len(), 3);
        // Descending month order
        assert_eq!(groups[0].key, "2024-03");
        assert_eq!(groups[1].key, "2024-02");
        assert_eq!(groups[2].key, "2024-01");
    }

    #[test]
    fn month_group_total_multiple_txns() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                c,
            ),
            make_txn(
                "C",
                dec!(30),
                NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
                c,
            ),
        ];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups[0].total, dec!(60));
    }

    #[test]
    fn no_transactions_returns_empty_groups() {
        let txns: Vec<Transaction> = vec![];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert!(groups.is_empty());
    }

    #[test]
    fn filter_removes_all_returns_empty_groups() {
        let c = Uuid::new_v4();
        let txns = vec![make_txn(
            "A",
            dec!(5),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            c,
        )];
        let filters = FilterConfig {
            price_min: Some(dec!(100)),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert!(groups.is_empty());
    }

    // ── Validation logic (mirrors dialog save) ──
    #[test]
    fn validate_name_empty() {
        let name = "";
        assert!(name.trim().is_empty());
    }

    #[test]
    fn validate_name_too_short() {
        let name = "A";
        assert!(name.trim().len() < 2);
    }

    #[test]
    fn validate_amount_zero_invalid() {
        let amount_str = "0";
        let parsed = Decimal::from_str(amount_str);
        match parsed {
            Ok(d) => assert!(d <= Decimal::ZERO),
            Err(_) => panic!("Should parse"),
        }
    }

    #[test]
    fn validate_amount_negative_invalid() {
        let amount_str = "-5";
        let parsed = Decimal::from_str(amount_str);
        match parsed {
            Ok(d) => assert!(d <= Decimal::ZERO),
            Err(_) => panic!("Should parse"),
        }
    }

    #[test]
    fn validate_amount_positive_valid() {
        let amount_str = "9.99";
        let parsed = Decimal::from_str(amount_str);
        match parsed {
            Ok(d) => assert!(d > Decimal::ZERO),
            Err(_) => panic!("Should parse"),
        }
    }

    #[test]
    fn validate_amount_non_numeric_invalid() {
        let amount_str = "abc";
        let parsed = Decimal::from_str(amount_str);
        assert!(parsed.is_err());
    }

    #[test]
    fn validate_date_valid_format() {
        let date_str = "2024-06-15";
        assert!(NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_ok());
    }

    #[test]
    fn validate_date_invalid_format() {
        let date_str = "06/15/2024";
        assert!(NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_err());
    }

    #[test]
    fn validate_date_empty_invalid() {
        let date_str = "";
        assert!(date_str.is_empty());
    }

    #[test]
    fn month_label_all_months() {
        for m in 1..=12u32 {
            let label = format_month(2024, m);
            assert!(!label.is_empty(), "Month {} should have a label", m);
        }
    }

    #[test]
    fn filter_config_default_is_empty() {
        let f = FilterConfig::default();
        assert!(f.category_id.is_none());
        assert!(f.price_min.is_none());
        assert!(f.price_max.is_none());
    }

    #[test]
    fn sort_config_default() {
        let s = SortConfig::default();
        assert_eq!(s.field, "");
        assert_eq!(s.direction, "");
    }

    #[test]
    fn transaction_has_notes() {
        let c = Uuid::new_v4();
        let mut txn = make_txn(
            "A",
            dec!(10),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            c,
        );
        txn.notes = Some("Important note".into());
        assert!(txn.notes.as_ref().is_some_and(|n| !n.is_empty()));
    }

    #[test]
    fn transaction_no_notes() {
        let c = Uuid::new_v4();
        let txn = make_txn(
            "A",
            dec!(10),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            c,
        );
        assert!(!txn.notes.as_ref().is_some_and(|n| !n.is_empty()));
    }

    // ── TDD: Transaction dialog click-outside ──
    #[test]
    fn transaction_dialog_card_has_stop_propagation() {
        // Add/Edit transaction modal now uses the shared Dialog component,
        // which owns the backdrop and click-outside handling.
        let src = include_str!("../transactions_page/dialog.rs");
        assert!(
            src.contains("Dialog {"),
            "Transaction Add/Edit dialog must use the Dialog component"
        );
    }

    // ── filter_sort_group additional edge cases ──
    #[test]
    fn filter_sort_group_empty_input() {
        let txns: Vec<Transaction> = vec![];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert!(groups.is_empty());
    }

    #[test]
    fn filter_sort_group_no_category() {
        let c = Uuid::new_v4();
        let txns = vec![make_txn(
            "A",
            dec!(10),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            c,
        )];
        let filters = FilterConfig {
            category_id: Some(Uuid::new_v4()),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert!(groups.is_empty());
    }

    #[test]
    fn filter_sort_group_price_min_above_all() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(5),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
        ];
        let filters = FilterConfig {
            price_min: Some(dec!(100)),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert!(groups.is_empty());
    }

    #[test]
    fn filter_sort_group_price_max_below_all() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(50),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(100),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
        ];
        let filters = FilterConfig {
            price_max: Some(dec!(10)),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert!(groups.is_empty());
    }

    #[test]
    fn filter_sort_group_combined_all_filters() {
        let c1 = Uuid::new_v4();
        let c2 = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c1,
            ),
            make_txn(
                "B",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c1,
            ),
            make_txn(
                "C",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                c2,
            ),
        ];
        let filters = FilterConfig {
            category_id: Some(c1),
            price_min: Some(dec!(15)),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert_eq!(groups[0].transactions.len(), 1);
        assert_eq!(groups[0].transactions[0].name, "B");
    }

    #[test]
    fn month_group_label_format() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
                c,
            ),
        ];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups.len(), 2);
        // Groups are sorted descending by key
        assert_eq!(groups[0].key, "2024-02");
        assert_eq!(groups[0].label, "February 2024");
        assert_eq!(groups[1].key, "2024-01");
        assert_eq!(groups[1].label, "January 2024");
    }

    #[test]
    fn month_group_total_sum() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10.5),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(20.25),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
            make_txn(
                "C",
                dec!(5.25),
                NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                c,
            ),
        ];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        // Sum = 10.5 + 20.25 + 5.25 = 36.0
        assert_eq!(groups[0].total, dec!(36.0));
    }

    #[test]
    fn sort_by_name_descending() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "Apple",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "Zebra",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
        ];
        let sort = SortConfig {
            field: "name".into(),
            direction: "desc".into(),
        };
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
        assert_eq!(groups[0].transactions[0].name, "Zebra");
        assert_eq!(groups[0].transactions[1].name, "Apple");
    }

    #[test]
    fn sort_by_date_ascending() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "Later",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                c,
            ),
            make_txn(
                "Earlier",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
                c,
            ),
        ];
        let sort = SortConfig {
            field: "date".into(),
            direction: "asc".into(),
        };
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
        assert_eq!(groups[0].transactions[0].name, "Earlier");
        assert_eq!(groups[0].transactions[1].name, "Later");
    }

    #[test]
    fn multiple_groups_each_with_total() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "JanA",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "JanB",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                c,
            ),
            make_txn(
                "FebA",
                dec!(5),
                NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
                c,
            ),
        ];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        // Jan: 30, Feb: 5
        assert_eq!(groups[0].key, "2024-02");
        assert_eq!(groups[0].total, dec!(5));
        assert_eq!(groups[1].key, "2024-01");
        assert_eq!(groups[1].total, dec!(30));
    }

    // ── MonthGroup struct tests ──
    #[test]
    fn month_group_key_format() {
        let c = Uuid::new_v4();
        let txns = vec![make_txn(
            "A",
            dec!(10),
            NaiveDate::from_ymd_opt(2024, 3, 5).unwrap(),
            c,
        )];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups[0].key, "2024-03");
    }

    #[test]
    fn month_group_transactions_preserved() {
        let c = Uuid::new_v4();
        let t1 = make_txn(
            "A",
            dec!(10),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            c,
        );
        let t2 = make_txn(
            "B",
            dec!(20),
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            c,
        );
        let txns = vec![t1.clone(), t2.clone()];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups[0].transactions.len(), 2);
    }

    // ── format_month boundary months ──
    #[test]
    fn format_month_january() {
        assert_eq!(format_month(2024, 1), "January 2024");
    }

    #[test]
    fn format_month_december() {
        assert_eq!(format_month(2024, 12), "December 2024");
    }

    #[test]
    fn format_month_single_digit_month() {
        assert_eq!(format_month(2024, 6), "June 2024");
    }

    #[test]
    fn format_month_year_boundary() {
        assert_eq!(format_month(2023, 12), "December 2023");
        assert_eq!(format_month(2025, 1), "January 2025");
    }

    // ── TransactionValidationResult tests ──
    #[test]
    fn txn_validation_result_default() {
        let v = TransactionValidationResult::default();
        assert!(v.is_valid());
        assert!(v.name_error.is_none());
    }

    #[test]
    fn txn_validation_result_with_errors() {
        let v = TransactionValidationResult {
            name_error: Some("Name required".into()),
            amount_error: Some("Amount required".into()),
            ..Default::default()
        };
        assert!(!v.is_valid());
        assert!(v.name_error.is_some());
        assert!(v.amount_error.is_some());
    }

    #[test]
    fn txn_validation_result_clone() {
        let v = TransactionValidationResult {
            name_error: Some("err".into()),
            ..Default::default()
        };
        let c = v.clone();
        assert_eq!(v.name_error, c.name_error);
    }

    // ── validate_transaction_form tests ──
    #[test]
    fn validate_txn_form_valid() {
        let r = validate_transaction_form("Coffee", "4.50", "EUR", "2024-01-15", "cat-id-123");
        assert!(r.is_valid());
    }

    #[test]
    fn validate_txn_form_empty_name() {
        let r = validate_transaction_form("", "4.50", "EUR", "2024-01-15", "cat-id-123");
        assert_eq!(r.name_error.as_deref(), Some("Name is required"));
        assert!(!r.is_valid());
    }

    #[test]
    fn validate_txn_form_whitespace_name() {
        let r = validate_transaction_form("   ", "4.50", "EUR", "2024-01-15", "cat-id-123");
        assert_eq!(r.name_error.as_deref(), Some("Name is required"));
    }

    #[test]
    fn validate_txn_form_short_name() {
        let r = validate_transaction_form("A", "4.50", "EUR", "2024-01-15", "cat-id-123");
        assert_eq!(
            r.name_error.as_deref(),
            Some("Name must be at least 2 characters")
        );
    }

    #[test]
    fn validate_txn_form_empty_amount() {
        let r = validate_transaction_form("Coffee", "", "EUR", "2024-01-15", "cat-id-123");
        assert_eq!(r.amount_error.as_deref(), Some("Amount is required"));
    }

    #[test]
    fn validate_txn_form_zero_amount() {
        let r = validate_transaction_form("Coffee", "0", "EUR", "2024-01-15", "cat-id-123");
        assert_eq!(
            r.amount_error.as_deref(),
            Some("Amount must be greater than 0")
        );
    }

    #[test]
    fn validate_txn_form_negative_amount() {
        let r = validate_transaction_form("Coffee", "-3", "EUR", "2024-01-15", "cat-id-123");
        assert_eq!(
            r.amount_error.as_deref(),
            Some("Amount must be greater than 0")
        );
    }

    #[test]
    fn validate_txn_form_invalid_amount() {
        let r = validate_transaction_form("Coffee", "xyz", "EUR", "2024-01-15", "cat-id-123");
        assert_eq!(r.amount_error.as_deref(), Some("Invalid amount format"));
    }

    #[test]
    fn validate_txn_form_empty_currency() {
        let r = validate_transaction_form("Coffee", "4.50", "", "2024-01-15", "cat-id-123");
        assert_eq!(r.currency_error.as_deref(), Some("Currency is required"));
    }

    #[test]
    fn validate_txn_form_empty_date() {
        let r = validate_transaction_form("Coffee", "4.50", "EUR", "", "cat-id-123");
        assert_eq!(r.date_error.as_deref(), Some("Date is required"));
    }

    #[test]
    fn validate_txn_form_invalid_date() {
        let r = validate_transaction_form("Coffee", "4.50", "EUR", "not-a-date", "cat-id-123");
        assert_eq!(r.date_error.as_deref(), Some("Invalid date format"));
    }

    #[test]
    fn validate_txn_form_empty_category() {
        let r = validate_transaction_form("Coffee", "4.50", "EUR", "2024-01-15", "");
        assert_eq!(r.category_error.as_deref(), Some("Category is required"));
    }

    #[test]
    fn validate_txn_form_multiple_errors() {
        let r = validate_transaction_form("", "", "", "", "");
        assert!(r.name_error.is_some());
        assert!(r.amount_error.is_some());
        assert!(r.currency_error.is_some());
        assert!(r.date_error.is_some());
        assert!(r.category_error.is_some());
    }

    #[test]
    fn validate_txn_form_decimal_amount() {
        let r = validate_transaction_form("Coffee", "9.99", "EUR", "2024-01-15", "cat-id-123");
        assert!(r.is_valid());

        // ── TransactionPage additional tests ─────────────────────────────────
        #[cfg(test)]
        mod transaction_page_tests {
            use super::*;

            #[test]
            fn filter_config_txn_default() {
                let f = FilterConfig::default();
                assert!(f.category_id.is_none());
                assert!(f.price_min.is_none());
                assert!(f.price_max.is_none());
            }

            #[test]
            fn sort_config_txn_default() {
                let s = SortConfig::default();
                assert_eq!(s.field, "date");
                assert_eq!(s.direction, "desc");
            }

            #[test]
            fn sort_config_txn_partial_eq() {
                let s1 = SortConfig {
                    field: "amount".into(),
                    direction: "asc".into(),
                };
                let s2 = SortConfig {
                    field: "amount".into(),
                    direction: "asc".into(),
                };
                assert_eq!(s1.field, s2.field);
                assert_eq!(s1.direction, s2.direction);
            }

            #[test]
            fn month_group_transactions_empty() {
                let _cat = Uuid::new_v4();
                let txns: Vec<Transaction> = vec![];
                let groups =
                    filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
                assert!(groups.is_empty());
            }

            #[test]
            fn month_group_single_transaction() {
                let cat = Uuid::new_v4();
                let txns = vec![make_txn(
                    "Coffee",
                    dec!(4.50),
                    NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
                    cat,
                )];
                let groups =
                    filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
                assert_eq!(groups.len(), 1);
                assert_eq!(groups[0].key, "2024-06");
                assert_eq!(groups[0].total, dec!(4.50));
                assert_eq!(groups[0].transactions.len(), 1);
            }

            #[test]
            fn month_group_key_format_various() {
                let cat = Uuid::new_v4();
                for (year, month, expected) in &[
                    (2024, 1, "2024-01"),
                    (2024, 12, "2024-12"),
                    (2025, 6, "2025-06"),
                ] {
                    let txns = vec![make_txn(
                        "T",
                        dec!(1),
                        NaiveDate::from_ymd_opt(*year, *month, 1).unwrap(),
                        cat,
                    )];
                    let groups =
                        filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
                    assert_eq!(
                        groups[0].key, *expected,
                        "key for {}/{} should be {}",
                        year, month, expected
                    );
                }
            }

            #[test]
            fn month_group_year_spans() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "Jan",
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "Dec",
                        dec!(20),
                        NaiveDate::from_ymd_opt(2023, 12, 1).unwrap(),
                        cat,
                    ),
                ];
                let groups =
                    filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
                assert_eq!(groups.len(), 2);
            }

            #[test]
            fn month_group_preserves_sort_within_group() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "B",
                        dec!(20),
                        NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "A",
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                        cat,
                    ),
                ];
                let sort = SortConfig {
                    field: "amount".into(),
                    direction: "asc".into(),
                };
                let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
                let names: Vec<_> = groups[0]
                    .transactions
                    .iter()
                    .map(|t| t.name.clone())
                    .collect();
                // Sorted by amount ASC: A (10) then B (20)
                assert_eq!(names[0], "A");
                assert_eq!(names[1], "B");
            }

            #[test]
            fn filter_sort_group_price_min_only() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "A",
                        dec!(5),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "B",
                        dec!(15),
                        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "C",
                        dec!(25),
                        NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                        cat,
                    ),
                ];
                let filters = FilterConfig {
                    price_min: Some(dec!(15)),
                    ..Default::default()
                };
                let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
                assert_eq!(groups[0].transactions.len(), 2);
            }

            #[test]
            fn filter_sort_group_price_max_only() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "A",
                        dec!(5),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "B",
                        dec!(15),
                        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "C",
                        dec!(25),
                        NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                        cat,
                    ),
                ];
                let filters = FilterConfig {
                    price_max: Some(dec!(15)),
                    ..Default::default()
                };
                let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
                assert_eq!(groups[0].transactions.len(), 2);
            }

            #[test]
            fn filter_sort_group_combined() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "A",
                        dec!(5),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "B",
                        dec!(15),
                        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                        cat,
                    ),
                ];
                let filters = FilterConfig {
                    price_min: Some(dec!(5)),
                    price_max: Some(dec!(15)),
                    ..Default::default()
                };
                let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
                assert_eq!(groups[0].transactions.len(), 2);
            }

            #[test]
            fn month_group_total_accumulation() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "A",
                        dec!(10.50),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "B",
                        dec!(20.25),
                        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "C",
                        dec!(15.25),
                        NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                        cat,
                    ),
                ];
                let groups =
                    filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
                assert_eq!(groups[0].total, dec!(46.00));
            }

            #[test]
            fn month_group_many_months() {
                let cat = Uuid::new_v4();
                let mut txns = vec![];
                for m in 1..=12 {
                    txns.push(make_txn(
                        &format!("M{}", m),
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, m, 1).unwrap(),
                        cat,
                    ));
                }
                let groups =
                    filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
                assert_eq!(groups.len(), 12);
            }

            #[test]
            fn sort_by_amount_ascending() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "C",
                        dec!(30),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "A",
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "B",
                        dec!(20),
                        NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                        cat,
                    ),
                ];
                let sort = SortConfig {
                    field: "amount".into(),
                    direction: "asc".into(),
                };
                let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
                let names: Vec<_> = groups[0]
                    .transactions
                    .iter()
                    .map(|t| t.name.clone())
                    .collect();
                assert_eq!(names, vec!["A", "B", "C"]);
            }

            #[test]
            fn sort_by_amount_descending() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "A",
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "B",
                        dec!(20),
                        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "C",
                        dec!(30),
                        NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                        cat,
                    ),
                ];
                let sort = SortConfig {
                    field: "amount".into(),
                    direction: "desc".into(),
                };
                let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
                let names: Vec<_> = groups[0]
                    .transactions
                    .iter()
                    .map(|t| t.name.clone())
                    .collect();
                assert_eq!(names, vec!["C", "B", "A"]);
            }

            #[test]
            fn sort_by_name() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "Zebra",
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "Apple",
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "Mango",
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                        cat,
                    ),
                ];
                let sort = SortConfig {
                    field: "name".into(),
                    direction: "asc".into(),
                };
                let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
                let names: Vec<_> = groups[0]
                    .transactions
                    .iter()
                    .map(|t| t.name.clone())
                    .collect();
                assert_eq!(names, vec!["Apple", "Mango", "Zebra"]);
            }

            #[test]
            fn sort_by_unknown_field() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "A",
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "B",
                        dec!(20),
                        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                        cat,
                    ),
                ];
                let sort = SortConfig {
                    field: "unknown_field".into(),
                    direction: "asc".into(),
                };
                let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
                // Unknown field: preserves original order
                assert_eq!(groups[0].transactions.len(), 2);
            }

            #[test]
            fn validate_txn_form_large_amount() {
                let r = validate_transaction_form(
                    "Big purchase",
                    "999999.99",
                    "EUR",
                    "2024-01-15",
                    "cat-id-123",
                );
                assert!(r.is_valid());
            }

            #[test]
            fn validate_txn_form_small_amount() {
                let r =
                    validate_transaction_form("Micro", "0.01", "EUR", "2024-01-15", "cat-id-123");
                assert!(r.is_valid());
            }

            #[test]
            fn validate_txn_form_valid_date_formats() {
                for date in &["2024-01-01", "2024-06-30", "2025-12-31"] {
                    let r = validate_transaction_form("Test", "10", "EUR", date, "cat-id-123");
                    assert!(r.date_error.is_none(), "date {} should be valid", date);
                }
            }
                }
    }

    // ── Lucide icon tests ──
    #[test]
    fn receipt_icon_renders() {
        let el: Element = rsx! { Receipt { size: 16 } };
        assert!(el.is_ok());
    }

    #[test]
    fn file_text_icon_renders() {
        let el: Element = rsx! { FileText { size: 16 } };
        assert!(el.is_ok());
    }

    #[test]
    fn pencil_icon_renders() {
        let el: Element = rsx! { Pencil { size: 16 } };
        assert!(el.is_ok());
    }

    #[test]
    fn trash2_icon_renders() {
        let el: Element = rsx! { Trash2 { size: 16 } };
        assert!(el.is_ok());
    }

    #[test]
    fn plus_icon_renders() {
        let el: Element = rsx! { Plus { size: 16 } };
        assert!(el.is_ok());
    }

    #[test]
    fn transaction_filter_sort_bar_responsive() {
        let prod = include_str!("../transactions_page.rs").split("#[cfg(test)]").next().unwrap_or("");
        assert!(prod.contains("flex-col sm:flex-row"),
            "Transaction filter/sort bar must stack vertically on mobile");
    }

    #[test]
    fn transaction_sort_uses_sm_ml_auto() {
        let prod = include_str!("../transactions_page/filter_sort_bar.rs");
        assert!(prod.contains("sm:ml-auto"),
            "Transaction sort section must push to right on desktop");
    }

    #[test]
    fn save_refreshes_list_before_closing_dialog() {
        let src = include_str!("dialog.rs");
        let save_block = src.split("match result {").nth(1).unwrap_or("");
        let refresh = save_block.find("list_transactions").unwrap_or(usize::MAX);
        let close = save_block.find("open.set(false);").unwrap_or(usize::MAX);
        assert!(
            refresh < close,
            "the save task must refresh the list before closing: unmounting the dialog cancels its spawned tasks"
        );
    }
