
    use super::*;

    #[test]
    fn dialog_form_default() {
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
    fn dialog_form_debug() {
        let form = DialogForm::default();
        let debug = format!("{:?}", form);
        assert!(debug.contains("EntryForm"));
    }

    #[test]
    fn dialog_form_clone() {
        let form = DialogForm::default();
        let cloned = form.clone();
        assert_eq!(form.name, cloned.name);
    }

    #[test]
    fn dialog_form_with_values() {
        let form = DialogForm {
            name: "Netflix".into(),
            amount: "15.99".into(),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            schedule_day: None,
            schedule_month: None,
            start_date: "2024-01-01".into(),
            end_date: String::new(),
            category_id: "cat-123".into(),
            notes: "Family plan".into(),
            logo_url: "netflix.png".into(),
        };
        assert_eq!(form.name, "Netflix");
        assert_eq!(form.amount, "15.99");
        assert_eq!(form.frequency, "monthly");
        assert!(form.notes.contains("Family"));
    }

    #[test]
    fn dialog_form_empty_values() {
        let form = DialogForm {
            name: String::new(),
            amount: String::new(),
            currency: String::new(),
            frequency: String::new(),
            schedule_day: None,
            schedule_month: None,
            start_date: String::new(),
            end_date: String::new(),
            category_id: String::new(),
            notes: String::new(),
            logo_url: String::new(),
        };
        assert!(form.name.is_empty());
        assert!(form.amount.is_empty());
        assert!(form.currency.is_empty());
    }

    // ── DialogForm additional tests ──
    #[test]
    fn dialog_form_debug_extended() {
        let form = DialogForm::default();
        let debug = format!("{:?}", form);
        assert!(debug.contains("EntryForm"));
    }

    #[test]
    fn dialog_form_name_set() {
        let mut form = DialogForm::default();
        form.name = "Spotify".into();
        assert_eq!(form.name, "Spotify");
    }

    #[test]
    fn dialog_form_amount_set() {
        let mut form = DialogForm::default();
        form.amount = "9.99".into();
        assert_eq!(form.amount, "9.99");
    }

    #[test]
    fn dialog_form_currency_set() {
        let mut form = DialogForm::default();
        form.currency = "USD".into();
        assert_eq!(form.currency, "USD");
    }

    #[test]
    fn dialog_form_frequency_set() {
        let mut form = DialogForm::default();
        form.frequency = "yearly".into();
        assert_eq!(form.frequency, "yearly");
    }

    #[test]
    fn dialog_form_start_date_set() {
        let mut form = DialogForm::default();
        form.start_date = "2024-06-15".into();
        assert_eq!(form.start_date, "2024-06-15");
    }

    #[test]
    fn dialog_form_category_id_set() {
        let mut form = DialogForm::default();
        form.category_id = "cat-id-123".into();
        assert_eq!(form.category_id, "cat-id-123");
    }

    #[test]
    fn dialog_form_notes_set() {
        let mut form = DialogForm::default();
        form.notes = "Monthly payment".into();
        assert_eq!(form.notes, "Monthly payment");
    }

    #[test]
    fn dialog_form_logo_url_set() {
        let mut form = DialogForm::default();
        form.logo_url = "https://logo.com/spotify.png".into();
        assert_eq!(form.logo_url, "https://logo.com/spotify.png");
    }

    #[test]
    fn dialog_form_multiple_fields() {
        let form = DialogForm {
            name: "Netflix".into(),
            amount: "15.99".into(),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            schedule_day: None,
            schedule_month: None,
            start_date: "2024-01-01".into(),
            end_date: String::new(),
            category_id: "cat-123".into(),
            notes: "Family plan".into(),
            logo_url: "netflix.png".into(),
        };
        assert_eq!(form.name, "Netflix");
        assert_eq!(form.amount, "15.99");
        assert_eq!(form.currency, "EUR");
        assert_eq!(form.frequency, "monthly");
        assert_eq!(form.start_date, "2024-01-01");
        assert_eq!(form.category_id, "cat-123");
        assert_eq!(form.notes, "Family plan");
        assert_eq!(form.logo_url, "netflix.png");
    }

    #[test]
    fn dialog_form_empty_notes() {
        let form = DialogForm::default();
        assert!(form.notes.is_empty());
    }

    #[test]
    fn dialog_form_empty_logo_url() {
        let form = DialogForm::default();
        assert!(form.logo_url.is_empty());
    }

    #[test]
    fn dialog_form_clone_independent() {
        let mut form = DialogForm::default();
        form.name = "Spotify".into();
        let cloned = form.clone();
        assert_eq!(cloned.name, "Spotify");
        // Modify original
        form.name = "Netflix".into();
        // Cloned should still have old value
        assert_eq!(cloned.name, "Spotify");
    }

    // ── SortConfig additional tests ──
    #[test]
    fn sort_config_default_field() {
        let s = SortConfig::default();
        assert_eq!(s.field, "next_billing_date");
        assert_eq!(s.direction, "asc");
    }

    #[test]
    fn sort_config_custom() {
        let s = SortConfig {
            field: "amount".into(),
            direction: "desc".into(),
        };
        assert_eq!(s.field, "amount");
        assert_eq!(s.direction, "desc");
    }

    #[test]
    fn sort_config_debug() {
        let s = SortConfig::default();
        let debug = format!("{:?}", s);
        assert!(debug.contains("SortConfig"));
    }

    #[test]
    fn sort_config_partial_eq() {
        let a = SortConfig {
            field: "name".into(),
            direction: "asc".into(),
        };
        let b = SortConfig {
            field: "name".into(),
            direction: "asc".into(),
        };
        assert_eq!(a.field, b.field);
        assert_eq!(a.direction, b.direction);
    }

    // ── FilterConfig additional tests ──
    #[test]
    fn filter_config_default_all_none() {
        let f = FilterConfig::default();
        assert!(f.status.is_none());
        assert!(f.frequency.is_none());
        assert!(f.category_id.is_none());
        assert!(f.price_min.is_none());
        assert!(f.price_max.is_none());
    }

    #[test]
    fn filter_config_debug() {
        let f = FilterConfig::default();
        let debug = format!("{:?}", f);
        assert!(debug.contains("FilterConfig"));
    }

    #[test]
    fn filter_config_clone() {
        let f = FilterConfig {
            status: Some("active".into()),
            ..Default::default()
        };
        let cloned = f.clone();
        assert_eq!(f.status, cloned.status);
    }

    // ── ValidationResult additional tests ──
    #[test]
    fn validation_result_all_errors() {
        let v = ValidationResult {
            name_error: Some("a".into()),
            amount_error: Some("b".into()),
            currency_error: Some("c".into()),
            frequency_error: Some("d".into()),
            start_date_error: Some("e".into()),
            category_error: Some("f".into()),
        };
        assert!(!v.is_valid());
        assert!(v.name_error.is_some());
        assert!(v.amount_error.is_some());
        assert!(v.currency_error.is_some());
        assert!(v.frequency_error.is_some());
        assert!(v.start_date_error.is_some());
        assert!(v.category_error.is_some());
    }

    #[test]
    fn validation_result_partial_errors() {
        let v = ValidationResult {
            amount_error: Some("bad".into()),
            ..Default::default()
        };
        assert!(!v.is_valid());
        assert!(v.name_error.is_none());
        assert!(v.amount_error.is_some());
    }

    // ── Card display helpers tests ──
    #[test]
    fn subscription_card_opacity_active() {
        assert_eq!(subscription_card_opacity(true, false), "");
    }

    #[test]
    fn subscription_card_opacity_inactive() {
        assert_eq!(subscription_card_opacity(false, false), "opacity-60");
        assert_eq!(subscription_card_opacity(true, true), "opacity-60");
    }

    #[test]
    fn subscription_next_text_active_with_date() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = subscription_next_text(true, Some(date), "DD/MM/YYYY");
        assert_eq!(result, "Next: 15/06/2024");
    }

    #[test]
    fn subscription_next_text_active_with_date_yyyy_mm_dd() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = subscription_next_text(true, Some(date), "YYYY/MM/DD");
        assert_eq!(result, "Next: 2024/06/15");
    }

    #[test]
    fn subscription_next_text_active_no_date() {
        let result = subscription_next_text(true, None, "DD/MM/YYYY");
        assert_eq!(result, "No billing date");
    }

    #[test]
    fn subscription_next_text_inactive() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = subscription_next_text(false, Some(date), "DD/MM/YYYY");
        assert_eq!(result, "Inactive");
    }

    #[test]
    fn subscription_next_text_inactive_no_date() {
        let result = subscription_next_text(false, None, "DD/MM/YYYY");
        assert_eq!(result, "Inactive");
    }

    #[test]
    fn subscription_has_notes_some() {
        assert!(subscription_has_notes(&Some("Some notes".into())));
    }

    #[test]
    fn subscription_has_notes_empty() {
        assert!(!subscription_has_notes(&Some("".into())));
    }

    #[test]
    fn subscription_has_notes_none() {
        assert!(!subscription_has_notes(&None));
    }

    // ── Lucide icon tests ──
    #[test]
    fn credit_card_icon_renders() {
        let el: Element = rsx! { CreditCard { size: 16 } };
        assert!(el.is_ok());
    }

    #[test]
    fn file_text_icon_renders() {
        let el: Element = rsx! { FileText { size: 16 } };
        assert!(el.is_ok());
    }

    #[test]
    fn pause_icon_renders() {
        let el: Element = rsx! { Pause { size: 14 } };
        assert!(el.is_ok());
    }

    #[test]
    fn play_icon_renders() {
        let el: Element = rsx! { Play { size: 14 } };
        assert!(el.is_ok());
    }

    #[test]
    fn pencil_icon_renders() {
        let el: Element = rsx! { Pencil { size: 14 } };
        assert!(el.is_ok());
    }

    #[test]
    fn trash2_icon_renders() {
        let el: Element = rsx! { Trash2 { size: 14 } };
        assert!(el.is_ok());
    }

    #[test]
    fn x_icon_renders() {
        let el: Element = rsx! { X { size: 14 } };
        assert!(el.is_ok());
    }

    #[test]
    fn arrow_up_down_icon_renders() {
        let el: Element = rsx! { ArrowUpDown { size: 16 } };
        assert!(el.is_ok());
    }

    #[test]
    fn subscriptions_use_compact_icon_toolbar() {
        let prod = include_str!("../subscriptions_page.rs").split("#[cfg(test)]").next().unwrap_or("");
        assert!(prod.contains("ListToolbar"), "subscriptions must use the shared list toolbar");
        assert!(prod.contains("aria_label: \"Sort\""), "sort must be an icon button");
        assert!(prod.contains("aria_label: \"Filters\""), "filters must be an icon button");
    }

#[test]
fn billing_schedule_ui_exposes_recurrence_controls() {
    let src = include_str!("dialog.rs");
    assert!(src.contains("Billing schedule"));
    assert!(src.contains("Charge on"));
    assert!(src.contains("Charge on these days"));
    assert!(src.contains("Annual dates"));
    assert!(src.contains("Start date"));
    assert!(src.contains("End date"));
    assert!(src.contains("Leave empty to start from today."));
    assert!(src.contains("Last billing date (inclusive)."));
}

#[test]
fn dialog_submits_schedule_and_end_date() {
    let src = include_str!("dialog.rs");
    assert!(src.contains("schedule_day: rec.legacy_day()"));
    assert!(src.contains("schedule_month: rec.legacy_month()"));
    assert!(src.contains("recurrence: Some(rec.clone())"));
    assert!(src.contains("clear_end_date: f.end_date.is_empty()"));
    assert!(!src.contains("recurrence: None"));
}
