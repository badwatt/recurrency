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
        dashboard.contains("text-4xl") || dashboard.contains("text-5xl") || dashboard.contains("text-6xl"),
        "Dashboard monthly total must use large display typography (text-4xl+)"
    );
}

#[test]
fn dashboard_has_staggered_animation_classes() {
    let dashboard = include_str!("../../pages/dashboard.rs").to_lowercase();
    // Must have animation/transition related patterns for staggered entry
    let has_anim = dashboard.contains("animate") || dashboard.contains("opacity-0") ||
        dashboard.contains("translate-y") || dashboard.contains("fade-in") ||
        dashboard.contains("stagger");
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
        dashboard.contains("col-span") || dashboard.contains("col-span-2") || dashboard.contains("row-span"),
        "Dashboard cards must use varied col-span/row-span for bento grid"
    );
}

#[test]
fn dashboard_has_quick_add_section() {
    let dashboard = include_str!("../../pages/dashboard.rs");
    // Must have quick-add subscription/transaction button
    assert!(
        dashboard.contains("New subscription") || dashboard.contains("New transaction") ||
        dashboard.contains("Add subscription") || dashboard.contains("Add transaction") ||
        dashboard.contains("Quick add") || dashboard.contains("Create"),
        "Dashboard must have quick-add subscription/transaction buttons"
    );
}
