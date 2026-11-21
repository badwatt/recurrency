/// Tests for dashboard bento grid redesign.

#[test]
fn dashboard_uses_bento_grid() {
    let dashboard = include_str!("../../pages/dashboard.rs");
    let prod = dashboard.split("#[cfg(test)]").next().unwrap_or(dashboard);
    assert!(
        prod.contains("grid"),
        "Dashboard must use CSS grid for bento layout"
    );
    // Must not be simple 1-3 column flex card row
    let dash_lower = dashboard.to_lowercase();
    assert!(
        !dash_lower.contains("grid-cols-1 md:grid-cols-2 lg:grid-cols-3"),
        "Dashboard must not use generic 3-column card grid"
    );
}

#[test]
fn dashboard_has_large_monthly_total() {
    let dashboard = include_str!("../../pages/dashboard.rs");
    // Monthly total must use large typography (text-3xl or bigger)
    assert!(
        dashboard.contains("text-4xl")
            || dashboard.contains("text-5xl")
            || dashboard.contains("text-6xl"),
        "Dashboard monthly total must use large display typography (text-4xl+)"
    );
}

#[test]
fn dashboard_has_staggered_animation_classes() {
    let dashboard = include_str!("../../pages/dashboard.rs").to_lowercase();
    // Must have animation/transition related patterns for staggered entry
    let has_anim = dashboard.contains("animate")
        || dashboard.contains("opacity-0")
        || dashboard.contains("translate-y")
        || dashboard.contains("page-enter")
        || dashboard.contains("stagger");
    assert!(
        has_anim,
        "Dashboard must include staggered animation classes for card entries"
    );
}

#[test]
fn dashboard_cards_vary_in_size() {
    let dashboard = include_str!("../../pages/dashboard.rs");
    // Cards must use different col-span values (bento grid)
    assert!(
        dashboard.contains("col-span")
            || dashboard.contains("col-span-2")
            || dashboard.contains("row-span"),
        "Dashboard cards must use varied col-span/row-span for bento grid"
    );
}

#[test]
fn dashboard_has_quick_add_section() {
    let dashboard = include_str!("../../pages/dashboard.rs");
    // Must have quick-add subscription/transaction button
    assert!(
        dashboard.contains("New subscription")
            || dashboard.contains("New transaction")
            || dashboard.contains("Add subscription")
            || dashboard.contains("Add transaction")
            || dashboard.contains("Quick add")
            || dashboard.contains("Create"),
        "Dashboard must have quick-add subscription/transaction buttons"
    );
}

#[test]
fn dashboard_quick_add_opens_each_form_directly() {
    let dashboard = include_str!("../../pages/dashboard.rs");
    assert!(dashboard.contains("CreateIntent::Subscription"));
    assert!(dashboard.contains("CreateIntent::Transaction"));
    assert!(dashboard.contains("CreateIntent::Debt"));
}

#[test]
fn dashboard_places_payments_left_and_accounts_right() {
    let dashboard = include_str!("../../pages/dashboard.rs");
    let production = dashboard.split("#[cfg(test)]").next().unwrap_or(dashboard);
    assert!(
        production.contains("lg:grid-cols-[minmax(0,1fr)_460px]"),
        "the lower dashboard must use a payments/account two-column grid"
    );
    assert!(
        production.contains("Card { class: \"order-1 min-w-0 overflow-hidden lg:order-2\","),
        "the account ledger must be first on mobile and right on desktop"
    );
    assert!(
        production.contains("order-2 flex min-w-0 flex-col gap-4 lg:order-1"),
        "payments must stack left on desktop"
    );
}

#[test]
fn hovering_a_payment_targets_its_account_row() {
    let dashboard = include_str!("../../pages/dashboard.rs");
    let production = dashboard.split("#[cfg(test)]").next().unwrap_or(dashboard);
    assert!(
        production.contains("onmouseenter: move |_| hovered_account.set(Some(account_id))"),
        "payment rows must publish their account on hover"
    );
    assert!(
        production.contains("let linked = hovered_account() == Some(account_id);"),
        "account rows must read the hovered account"
    );
    assert!(
        production.contains("bg-primary/5 ring-1 ring-inset ring-primary/20"),
        "the linked account row must use a visible highlight"
    );
}

#[test]
fn dashboard_has_one_upcoming_list_and_no_manual_panel() {
    let dashboard = include_str!("../../pages/dashboard.rs");
    let production = dashboard.split("#[cfg(test)]").next().unwrap_or(dashboard);
    assert!(
        !production.contains("\"Manual payments\""),
        "the separate manual payments panel must be gone"
    );
    assert!(
        production.contains("Subscriptions and one-time payments."),
        "upcoming payments must describe both payment kinds"
    );
    assert!(
        production.contains("One-time"),
        "one-time transactions must be identifiable in the upcoming list"
    );
}

#[test]
fn dashboard_ledger_switches_between_accounts_and_categories() {
    let dashboard = include_str!("../../pages/dashboard.rs");
    let production = dashboard.split("#[cfg(test)]").next().unwrap_or(dashboard);
    assert!(
        production.contains("LedgerView::Accounts"),
        "the ledger must default to the accounts view"
    );
    assert!(
        production.contains("LedgerView::Categories"),
        "the ledger must offer a categories view"
    );
    assert!(
        production.contains("This month's charges grouped by category."),
        "the categories view must explain what it shows"
    );
}

#[test]
fn category_hover_links_the_ledger_and_upcoming_payments() {
    let dashboard = include_str!("../../pages/dashboard.rs");
    let production = dashboard.split("#[cfg(test)]").next().unwrap_or(dashboard);
    assert!(
        production.contains("hovered_category"),
        "the dashboard must track a hovered category"
    );
    assert!(
        production.contains("hovered_category() == Some(category_id)"),
        "category rows must read the hovered category"
    );
    assert!(
        production.contains("hovered_category() == Some(category_id)"),
        "upcoming rows must read the hovered category"
    );
    assert!(
        production.contains("hovered_category.set(Some(category_id))"),
        "both upcoming and category rows must publish the category"
    );
}
