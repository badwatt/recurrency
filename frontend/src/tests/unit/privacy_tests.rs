//! Privacy mode wiring.

#[test]
fn converted_amounts_are_marked_sensitive() {
    let source = include_str!("../../components/converted_amount.rs");
    assert!(source.contains("amount-sensitive"));
}

#[test]
fn privacy_toggle_lives_in_the_app_header() {
    let source = include_str!("../../layout/app_layout.rs");
    assert!(source.contains("PrivacyToggle"));
    assert!(source.contains("privacy-mode"));
}

#[test]
fn privacy_css_blurs_sensitive_amounts() {
    let css = include_str!("../../../assets/styles/tailwind.src.css");
    assert!(css.contains(".privacy-mode .amount-sensitive"));
    assert!(css.contains("filter: blur(6px)"));
}

#[test]
fn frameless_is_a_native_window_menu_item() {
    let source = include_str!("../../layout/app_layout.rs");
    assert!(source.contains("use_muda_event_handler"));
    assert!(source.contains("set_decorations"));
    assert!(source.contains("MENU_ID"));
}
