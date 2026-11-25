//! Render coverage for shared/domain components that are only visible after a
//! user interaction. They use the same isolated offline environment as the
//! route smoke tests.

use dioxus::prelude::*;

use crate::pages::settings_page::about_page::AboutPage;
use crate::pages::settings_page::apk_card::ApkCard;
use crate::pages::settings_page::delivery_card::DeliveryCard;
use crate::pages::settings_page::exchange_rates_card::ExchangeRatesCard;
use crate::pages::settings_page::preferences_card::PreferencesCard;
use crate::pages::settings_page::summary_card::SummaryCard;
use crate::pages::settings_page::sync_card::SyncFolderCard;
use crate::pages::transactions_page::cards::TransactionCard;
use crate::pages::transactions_page::dialog::TransactionDialog;
use crate::pages::transactions_page::notes_dialog::NotesDialog;

fn render(component: fn() -> Element) {
    let _guard = crate::storage::STORAGE_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _auth_guard = crate::storage::local_auth::AUTH_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    crate::api::client::force_offline_for_tests();
    let dir =
        std::env::temp_dir().join(format!("recurrency-shared-render-{}", uuid::Uuid::new_v4()));
    crate::storage::local_auth::set_render_test_dir(dir);
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    crate::api::client::reset_offline_for_tests();
    crate::storage::local_auth::clear_render_test_dir();
}

#[test]
fn settings_content_cards_render_with_ssr() {
    for (html, expected) in [
        (
            dioxus_ssr::render_element(rsx! { PreferencesCard {} }),
            "Defaults",
        ),
        (
            dioxus_ssr::render_element(rsx! { SummaryCard {} }),
            "Dashboard summary",
        ),
        (
            dioxus_ssr::render_element(rsx! { ExchangeRatesCard {} }),
            "Exchange rates",
        ),
        (
            dioxus_ssr::render_element(rsx! { SyncFolderCard {} }),
            "Cloud folder sync",
        ),
        (
            dioxus_ssr::render_element(rsx! { DeliveryCard {} }),
            "Delivery channels",
        ),
        (
            dioxus_ssr::render_element(rsx! { AboutPage {} }),
            "Every expense is a subscription",
        ),
        (
            dioxus_ssr::render_element(rsx! { ApkCard {} }),
            "Android app",
        ),
    ] {
        assert!(html.contains(expected), "missing {expected}");
    }
}

#[test]
fn settings_content_cards_render() {
    render(|| rsx! { PreferencesCard {} });
    render(|| rsx! { ExchangeRatesCard {} });
    render(|| rsx! { SummaryCard {} });
    render(|| rsx! { SyncFolderCard {} });
    render(|| rsx! { DeliveryCard {} });
}

#[test]
fn about_and_android_cards_render() {
    render(|| rsx! { AboutPage {} });
    render(|| rsx! { ApkCard {} });
}

#[component]
fn TransactionNotesHarness() -> Element {
    let open = use_signal(|| true);
    let id = use_signal(|| Some(uuid::Uuid::new_v4()));
    rsx! { NotesDialog { open, txn_id: id } }
}

#[test]
fn list_pages_render_with_ssr() {
    let _guard = crate::storage::STORAGE_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    crate::api::client::force_offline_for_tests();
    for (html, expected) in [
        (
            dioxus_ssr::render_element(
                rsx! { crate::pages::subscriptions_page::SubscriptionsPage {} },
            ),
            "Subscriptions",
        ),
        (
            dioxus_ssr::render_element(rsx! { crate::pages::TransactionsPage {} }),
            "Transactions",
        ),
        (
            dioxus_ssr::render_element(rsx! { crate::pages::DebtsPage {} }),
            "Debts",
        ),
        (
            dioxus_ssr::render_element(rsx! { crate::pages::ReportsPage {} }),
            "Reports",
        ),
        (
            dioxus_ssr::render_element(rsx! { crate::pages::CalendarPage {} }),
            "Calendar",
        ),
        (
            dioxus_ssr::render_element(rsx! { crate::pages::AccountsPage {} }),
            "Accounts",
        ),
        (
            dioxus_ssr::render_element(rsx! { crate::pages::CategoriesPage {} }),
            "Categories",
        ),
    ] {
        assert!(html.contains(expected), "missing {expected}");
    }
    crate::api::client::reset_offline_for_tests();
}

#[test]
fn transaction_notes_dialog_renders_open() {
    render(|| rsx! { TransactionNotesHarness {} });
    let html = dioxus_ssr::render_element(rsx! { TransactionNotesHarness {} });
    assert!(html.contains("Notes"));
}

#[component]
fn TransactionDialogHarness() -> Element {
    let open = use_signal(|| true);
    let editing = use_signal(|| None);
    rsx! { TransactionDialog { open, editing_id: editing } }
}

#[test]
fn transaction_dialog_renders_open() {
    render(|| rsx! { TransactionDialogHarness {} });
    let html = dioxus_ssr::render_element(rsx! { TransactionDialogHarness {} });
    assert!(html.contains("Add Transaction"));
}

#[component]
fn TransactionCardHarness() -> Element {
    let txn = sample_transaction(None);
    rsx! {
        TransactionCard {
            txn,
            selection_mode: false,
            selected: false,
            on_select: move |_| {},
            on_edit: move |_| {},
            on_delete: move |_| {},
            on_notes: move |_| {},
        }
    }
}

#[component]
fn SelectedTransactionCardHarness() -> Element {
    let txn = sample_transaction(Some("Notes".to_string()));
    rsx! {
        TransactionCard {
            txn,
            selection_mode: true,
            selected: true,
            on_select: move |_| {},
            on_edit: move |_| {},
            on_delete: move |_| {},
            on_notes: move |_| {},
        }
    }
}

fn sample_transaction(notes: Option<String>) -> crate::models::transaction::Transaction {
    crate::models::transaction::Transaction {
        id: uuid::Uuid::new_v4(),
        user_id: uuid::Uuid::new_v4(),
        account_id: uuid::Uuid::nil(),
        category_id: uuid::Uuid::nil(),
        name: "Coffee".to_string(),
        amount: rust_decimal::Decimal::new(350, 2),
        currency: "EUR".to_string(),
        date: chrono::NaiveDate::from_ymd_opt(2026, 1, 2).unwrap_or_default(),
        notes,
        logo_url: None,
        created_at: chrono::DateTime::UNIX_EPOCH,
        updated_at: chrono::DateTime::UNIX_EPOCH,
    }
}

#[test]
fn transaction_cards_render_in_both_selection_states() {
    render(|| rsx! { TransactionCardHarness {} });
    render(|| rsx! { SelectedTransactionCardHarness {} });
    let html = dioxus_ssr::render_element(rsx! { TransactionCardHarness {} });
    assert!(html.contains("Coffee"));
}

#[component]
fn SubscriptionDialogHarness() -> Element {
    let form = use_signal(crate::components::entry_form::EntryForm::default);
    let recurrence = use_signal(crate::models::subscription::Recurrence::default);
    let active = use_signal(|| true);
    let editing = use_signal(|| None);
    let dialog_open = use_signal(|| true);
    let name_error = use_signal(String::new);
    let amount_error = use_signal(String::new);
    let currency_error = use_signal(String::new);
    let frequency_error = use_signal(String::new);
    let start_date_error = use_signal(String::new);
    let end_date_error = use_signal(String::new);
    let account_error = use_signal(String::new);
    let category_error = use_signal(String::new);
    let account_id = use_signal(String::new);
    rsx! {
        {crate::pages::subscriptions_page::dialog::render_dialog(
            form, recurrence, active, editing, dialog_open,
            name_error, amount_error, currency_error, frequency_error,
            start_date_error, end_date_error, account_error, category_error,
            account_id, None,
        )}
    }
}

#[test]
fn subscription_dialog_renders_open() {
    render(|| rsx! { SubscriptionDialogHarness {} });
    let html = dioxus_ssr::render_element(rsx! { SubscriptionDialogHarness {} });
    assert!(html.contains("Add Subscription"));
}

#[component]
fn SubscriptionNotesHarness() -> Element {
    let open = use_signal(|| true);
    let id = use_signal(|| Some(uuid::Uuid::new_v4()));
    let value = use_signal(String::new);
    let saving = use_signal(|| false);
    rsx! {
        crate::pages::subscriptions_page::notes_dialog::NotesSubscriptionDialog {
            open, sub_id: id, value, saving
        }
    }
}

#[component]
fn SubscriptionDeleteHarness() -> Element {
    let id = use_signal(|| Some(uuid::Uuid::new_v4()));
    rsx! { crate::pages::subscriptions_page::delete_dialog::DeleteSubscriptionDialog { confirm_id: id } }
}

#[test]
fn subscription_notes_and_delete_dialogs_render() {
    render(|| rsx! { SubscriptionNotesHarness {} });
    render(|| rsx! { SubscriptionDeleteHarness {} });
    let html = dioxus_ssr::render_element(rsx! { SubscriptionNotesHarness {} });
    assert!(html.contains("Notes"));
    let html = dioxus_ssr::render_element(rsx! { SubscriptionDeleteHarness {} });
    assert!(html.contains("Delete subscription"));
}

fn sample_account(id: uuid::Uuid) -> crate::models::account::Account {
    crate::models::account::Account {
        id,
        user_id: uuid::Uuid::new_v4(),
        name: "Main".to_string(),
        icon: Some("Wallet".to_string()),
        color: Some("#6366f1".to_string()),
        is_default: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

fn sample_category(id: uuid::Uuid) -> crate::models::category::Category {
    crate::models::category::Category {
        id,
        user_id: uuid::Uuid::new_v4(),
        name: "General".to_string(),
        icon: Some("Tag".to_string()),
        color: Some("#6366f1".to_string()),
        is_system: false,
        is_default: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

fn sample_subscription(
    account_id: uuid::Uuid,
    category_id: uuid::Uuid,
) -> crate::models::subscription::Subscription {
    let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap_or_default();
    crate::models::subscription::Subscription {
        id: uuid::Uuid::new_v4(),
        user_id: uuid::Uuid::new_v4(),
        account_id,
        category_id,
        name: "Spotify".to_string(),
        amount: rust_decimal::Decimal::new(999, 2),
        currency: "EUR".to_string(),
        frequency: "monthly".to_string(),
        schedule_day: Some(1),
        schedule_month: None,
        recurrence: crate::models::subscription::Recurrence::from_legacy(
            "monthly",
            start,
            Some(1),
            None,
        ),
        start_date: start,
        end_date: None,
        next_billing_date: Some(start),
        is_active: true,
        notes: Some("Family plan".to_string()),
        logo_url: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

fn sample_transaction_with_account(
    account_id: uuid::Uuid,
    category_id: uuid::Uuid,
) -> crate::models::transaction::Transaction {
    crate::models::transaction::Transaction {
        id: uuid::Uuid::new_v4(),
        user_id: uuid::Uuid::new_v4(),
        account_id,
        category_id,
        name: "Coffee".to_string(),
        amount: rust_decimal::Decimal::new(350, 2),
        currency: "EUR".to_string(),
        date: chrono::NaiveDate::from_ymd_opt(2026, 1, 2).unwrap_or_default(),
        notes: None,
        logo_url: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[component]
fn SeededPages(children: Element) -> Element {
    use_hook(|| {
        let account_id = uuid::Uuid::nil();
        let category_id = uuid::Uuid::nil();
        crate::state::accounts::ACCOUNTS
            .write()
            .set_data(vec![sample_account(account_id)]);
        crate::state::categories::CATEGORIES
            .write()
            .set_data(vec![sample_category(category_id)]);
        crate::state::subscriptions::SUBSCRIPTIONS
            .write()
            .set_data(vec![sample_subscription(account_id, category_id)]);
        crate::state::transactions::TRANSACTIONS
            .write()
            .set_data(vec![sample_transaction_with_account(
                account_id,
                category_id,
            )]);
    });
    use_drop(|| {
        crate::state::accounts::ACCOUNTS
            .write()
            .set_data(Vec::new());
        crate::state::categories::CATEGORIES
            .write()
            .set_data(Vec::new());
        crate::state::subscriptions::SUBSCRIPTIONS
            .write()
            .set_data(Vec::new());
        crate::state::transactions::TRANSACTIONS
            .write()
            .set_data(Vec::new());
    });
    rsx! { {children} }
}

#[test]
fn pages_render_with_sample_data() {
    let _guard = crate::storage::STORAGE_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    crate::api::client::force_offline_for_tests();

    let html = dioxus_ssr::render_element(rsx! {
        SeededPages { crate::pages::subscriptions_page::SubscriptionsPage {} }
    });
    assert!(html.contains("Spotify"));
    let html = dioxus_ssr::render_element(rsx! {
        SeededPages { crate::pages::TransactionsPage {} }
    });
    assert!(html.contains("Coffee"));
    let html = dioxus_ssr::render_element(rsx! {
        SeededPages { crate::pages::AccountsPage {} }
    });
    assert!(html.contains("Main"));
    let html = dioxus_ssr::render_element(rsx! {
        SeededPages { crate::pages::CategoriesPage {} }
    });
    assert!(html.contains("General"));

    crate::api::client::reset_offline_for_tests();
}

#[test]
fn privacy_and_account_form_primitives_render_with_ssr() {
    let html =
        dioxus_ssr::render_element(rsx! { crate::components::privacy_toggle::PrivacyToggle {} });
    assert!(html.contains("Hide amounts") || html.contains("Show amounts"));

    #[component]
    fn AccountFieldsHarness() -> Element {
        let name = use_signal(|| "Main".to_string());
        let icon = use_signal(|| "Wallet".to_string());
        let color = use_signal(|| "#6366f1".to_string());
        let error = use_signal(String::new);
        rsx! { crate::components::account_fields::AccountFields { name, icon, color, name_error: error } }
    }
    let html = dioxus_ssr::render_element(rsx! { AccountFieldsHarness {} });
    assert!(html.contains("Main"));
}

#[test]
fn seeded_lists_and_calendar_render_with_ssr() {
    let _guard = crate::storage::STORAGE_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    crate::api::client::force_offline_for_tests();
    let html = dioxus_ssr::render_element(rsx! {
        SeededPages { crate::pages::CalendarPage {} }
    });
    assert!(html.contains("Calendar"));
    let html = dioxus_ssr::render_element(rsx! {
        SeededPages { crate::pages::SettingsMoneyPage {} }
    });
    assert!(html.contains("Accounts"));
    crate::api::client::reset_offline_for_tests();
}
