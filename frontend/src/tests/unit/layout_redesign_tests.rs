/// Tests for radical layout redesign — top nav replaces fixed sidebar.

#[test]
fn no_fixed_sidebar_in_layout() {
    let layout = include_str!("../../layout/app_layout.rs");
    let prod = layout.split("#[cfg(test)]").next().unwrap_or(layout);
    let fixed_count = prod.matches("lg:fixed lg:inset-y-0").count();
    assert_eq!(fixed_count, 0, "app_layout must NOT have fixed left sidebar");
    assert!(
        !prod.contains("lg:w-64"),
        "app_layout must NOT have 64-unit wide fixed sidebar"
    );
}

#[test]
fn top_navigation_bar_exists() {
    let layout = include_str!("../../layout/app_layout.rs");
    // Must have a horizontal nav at the top, not just the old header
    assert!(
        layout.contains("top-0") && layout.contains("sticky"),
        "app_layout must have sticky top navigation bar"
    );
    assert!(
        layout.contains("w-full"),
        "top nav bar must be full width"
    );
}

#[test]
fn no_left_sidebar_padding_on_main() {
    let layout = include_str!("../../layout/app_layout.rs");
    // Main content must not have lg:pl-64 (the old sidebar padding)
    let count = layout.matches("lg:pl-64").count();
    assert_eq!(count, 0, "main content must not have lg:pl-64 (no sidebar to offset)");
}

#[test]
fn layout_uses_top_nav_not_sidebar_terms() {
    let layout = include_str!("../../layout/app_layout.rs").to_lowercase();
    // Should reference nav/header navigation, not sidebar
    // The word "sidebar" may appear in comments/tests, but the structure must not be sidebar-based
    // Check that the main layout uses nav or top-bar, not aside as primary navigation
    assert!(
        layout.contains("nav") || layout.contains("top-bar") || layout.contains("topbar"),
        "layout must reference nav or top-bar navigation"
    );
}

#[test]
fn mobile_bottom_bar_exists() {
    let layout = include_str!("../../layout/app_layout.rs").to_lowercase();
    // Mobile must have bottom bar instead of hamburger sidebar
    // Look for bottom-0 or fixed bottom patterns
    assert!(
        layout.contains("bottom-0") || layout.contains("bottom-bar") || layout.contains("bottombar"),
        "mobile layout must have bottom navigation bar"
    );
}
