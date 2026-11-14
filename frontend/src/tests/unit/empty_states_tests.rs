//! Empty states use the same structure as Transactions.

#[test]
fn debts_empty_state_has_icon_title_and_add_button() {
    let source = include_str!("../../pages/debts_page.rs");
    assert!(source.contains("No debts yet"));
    assert!(source.contains("Add Debt"));
    assert!(source.contains("Plus { size: 16 }"));
}

#[test]
fn subscriptions_empty_state_has_icon_title_and_add_button() {
    let source = include_str!("../../pages/subscriptions_page.rs");
    assert!(source.contains("No subscriptions yet"));
    assert!(source.contains("Add Subscription"));
    assert!(source.contains("Plus { size: 16 }"));
}

#[test]
fn empty_states_share_the_transactions_card_layout() {
    for source in [
        include_str!("../../pages/transactions_page.rs"),
        include_str!("../../pages/debts_page.rs"),
        include_str!("../../pages/subscriptions_page.rs"),
    ] {
        assert!(source.contains("rounded-lg border border-border bg-card p-8 flex flex-col items-center justify-center gap-4"));
    }
}
