//! Smoke-render every route page without a browser or network.

use dioxus::prelude::*;

use crate::pages::{
    AccountsPage, AdminPage, CalendarPage, CategoriesPage, Dashboard, DebtsPage, ReportsPage,
    SettingsAboutPage, SettingsAdminPage, SettingsAiPage, SettingsDataPage, SettingsMoneyPage,
    SettingsAppearancePage, SettingsNotificationsPage, SettingsPreferencesPage,
    SettingsProfilePage, SettingsServerPage,
    TransactionsPage,
};
use crate::pages::subscriptions_page::SubscriptionsPage;
use crate::components::ai_chat::AiChat;

fn render(component: fn() -> Element) {
    let _guard = crate::storage::STORAGE_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _auth_guard = crate::storage::local_auth::AUTH_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    crate::api::client::force_offline_for_tests();
    let dir = std::env::temp_dir().join(format!(
        "recurrency-render-auth-{}",
        uuid::Uuid::new_v4()
    ));
    crate::storage::local_auth::set_render_test_dir(dir);
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    crate::api::client::reset_offline_for_tests();
    crate::storage::local_auth::clear_render_test_dir();
}

#[test]
fn dashboard_renders() {
    render(|| rsx! { Dashboard {} });
}

#[test]
fn subscriptions_page_renders() {
    render(|| rsx! { SubscriptionsPage {} });
}

#[test]
fn transactions_page_renders() {
    render(|| rsx! { TransactionsPage {} });
}

#[test]
fn debts_page_renders() {
    render(|| rsx! { DebtsPage {} });
}

#[test]
fn calendar_page_renders() {
    render(|| rsx! { CalendarPage {} });
}

#[test]
fn reports_page_renders() {
    render(|| rsx! { ReportsPage {} });
}

#[test]
fn accounts_and_categories_render() {
    render(|| rsx! { AccountsPage {} });
    render(|| rsx! { CategoriesPage {} });
}

#[test]
fn admin_page_renders() {
    render(|| rsx! { AdminPage {} });
}

#[test]
fn settings_pages_render() {
    render(|| rsx! { SettingsProfilePage {} });
    render(|| rsx! { SettingsAppearancePage {} });
    render(|| rsx! { SettingsPreferencesPage {} });
    render(|| rsx! { SettingsMoneyPage {} });
    render(|| rsx! { SettingsDataPage {} });
    render(|| rsx! { SettingsServerPage {} });
    render(|| rsx! { SettingsNotificationsPage {} });
    render(|| rsx! { SettingsAboutPage {} });
    render(|| rsx! { SettingsAdminPage {} });
    render(|| rsx! { SettingsAiPage {} });
}

#[test]
fn ai_chat_renders() {
    render(|| rsx! { AiChat {} });
}

#[test]
fn app_loader_renders() {
    render(|| rsx! { crate::layout::app_layout::AppLoader {} });
}

#[test]
fn settings_layout_renders() {
    render(|| rsx! { crate::layout::SettingsLayout {} });
}

#[test]
fn app_layout_renders() {
    render(|| rsx! { crate::layout::AppLayout {} });
}

#[test]
fn admin_page_renders_for_an_administrator() {
    use crate::models::auth::UserResponse;
    use crate::state::auth::AUTH;
    let user = UserResponse {
        id: uuid::Uuid::new_v4(),
        email: "admin@test.com".into(),
        avatar: None,
        name: Some("Admin".into()),
        currency: "EUR".into(),
        verified: true,
        is_admin: true,
    };
    let _guard = crate::storage::STORAGE_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _auth_guard = crate::storage::local_auth::AUTH_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let dir = std::env::temp_dir().join(format!(
        "recurrency-render-auth-{}",
        uuid::Uuid::new_v4()
    ));
    crate::storage::local_auth::set_render_test_dir(dir);
    let mut dom = VirtualDom::new(AdminPage);
    dom.in_runtime(|| {
        AUTH.write().set_authenticated("token".into(), user, false);
    });
    dom.rebuild_in_place();
}

#[test]
fn route_wrappers_render() {
    render(|| rsx! { crate::Login {} });
    render(|| rsx! { crate::Register {} });
    render(|| rsx! { crate::ServerSettings {} });
    render(|| rsx! { crate::Dashboard {} });
    render(|| rsx! { crate::Subscriptions {} });
    render(|| rsx! { crate::Transactions {} });
    render(|| rsx! { crate::Debts {} });
    render(|| rsx! { crate::Calendar {} });
    render(|| rsx! { crate::Reports {} });
    render(|| rsx! { crate::Accounts {} });
    render(|| rsx! { crate::Categories {} });
    render(|| rsx! { crate::Settings {} });
    render(|| rsx! { crate::SettingsProfile {} });
    render(|| rsx! { crate::SettingsAppearance {} });
    render(|| rsx! { crate::SettingsPreferences {} });
    render(|| rsx! { crate::SettingsAi {} });
    render(|| rsx! { crate::SettingsMoney {} });
    render(|| rsx! { crate::SettingsNotifications {} });
    render(|| rsx! { crate::SettingsData {} });
    render(|| rsx! { crate::SettingsServer {} });
    render(|| rsx! { crate::SettingsAboutPage {} });
    render(|| rsx! { crate::SettingsAdmin {} });
    render(|| rsx! { crate::SettingsAccounts {} });
    render(|| rsx! { crate::SettingsCategories {} });
}

#[test]
fn font_face_css_uses_bundled_assets() {
    let css = crate::font_face_css();
    assert!(css.contains("Cabinet Grotesk"));
    assert!(css.contains("JetBrains Mono"));
    // Block avoids a late swap that reflows text after the loader is gone.
    assert_eq!(css.matches("font-display: block").count(), 6);
    assert_eq!(css.matches("@font-face").count(), 6);
}
