//! Smoke-render every route page without a browser or network.

use dioxus::prelude::*;

use crate::pages::{
    AccountsPage, AdminPage, CalendarPage, CategoriesPage, Dashboard, DebtsPage, ReportsPage,
    SettingsAboutPage, SettingsAdminPage, SettingsAiPage, SettingsDataPage, SettingsMoneyPage,
    SettingsNotificationsPage, SettingsPreferencesPage, SettingsProfilePage, SettingsServerPage,
    TransactionsPage,
};
use crate::pages::subscriptions_page::SubscriptionsPage;
use crate::components::ai_chat::AiChat;

fn render(component: fn() -> Element) {
    crate::api::client::force_offline_for_tests();
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    crate::api::client::reset_offline_for_tests();
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
