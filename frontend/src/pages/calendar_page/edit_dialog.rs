use crate::components::entry_form::{EntryForm as EditForm, EntityType};
use crate::models::subscription::{CalendarItem, Recurrence};
use crate::pages::subscriptions_page::dialog::render_dialog;
use crate::state::accounts::ACCOUNTS;
use crate::state::auth::AUTH;
use crate::state::categories::CATEGORIES;
use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::storage::Storage;
use dioxus::prelude::*;
use uuid::Uuid;

/// Calendar uses the exact same subscription modal as the Subscriptions page.
/// The only calendar-specific behavior is refreshing the visible month after save.
#[component]
pub fn EditSubscriptionDialog(
    open: Signal<bool>,
    editing_id: Signal<Option<Uuid>>,
    prefill_date: Signal<Option<String>>,
    calendar_items: Signal<Vec<CalendarItem>>,
    current_year: Signal<i32>,
    current_month: Signal<u32>,
) -> Element {
    let initial = if let Some(id) = editing_id() {
        SUBSCRIPTIONS()
            .subscriptions
            .iter()
            .find(|subscription| subscription.id == id)
            .map(|subscription| {
                EditForm::from_entry(
                    &subscription.name,
                    &subscription.amount.to_string(),
                    &subscription.currency,
                    &subscription.frequency,
                    subscription.schedule_day,
                    subscription.schedule_month,
                    &subscription.start_date.to_string(),
                    &subscription
                        .end_date
                        .map(|date| date.to_string())
                        .unwrap_or_default(),
                    &subscription.category_id.to_string(),
                    &subscription.notes.clone().unwrap_or_default(),
                    &subscription.logo_url.clone().unwrap_or_default(),
                )
            })
            .unwrap_or_default()
    } else {
        let default_category = CATEGORIES()
            .default_category()
            .map(|category| category.id.to_string())
            .unwrap_or_default();
        EditForm::init_for_create(
            &prefill_date().unwrap_or_default(),
            AUTH().currency(),
            &default_category,
            EntityType::Subscription,
        )
    };

    let form = use_signal(move || initial);
    let active = use_signal(|| true);
    let recurrence = use_signal(|| {
        editing_id()
            .and_then(|id| {
                SUBSCRIPTIONS()
                    .subscriptions
                    .iter()
                    .find(|subscription| subscription.id == id)
                    .map(|subscription| subscription.recurrence.clone())
            })
            .unwrap_or_else(|| {
                let days = prefill_date()
                    .and_then(|value| value.parse::<chrono::NaiveDate>().ok())
                    .map(|date| vec![chrono::Datelike::day(&date) as u8])
                    .unwrap_or_else(|| vec![1]);
                Recurrence {
                    days,
                    ..Default::default()
                }
            })
    });
    let form_account_id = use_signal(|| {
        editing_id()
            .and_then(|id| {
                SUBSCRIPTIONS()
                    .subscriptions
                    .iter()
                    .find(|subscription| subscription.id == id)
                    .map(|subscription| subscription.account_id.to_string())
            })
            .or_else(|| ACCOUNTS().default_account().map(|account| account.id.to_string()))
            .unwrap_or_default()
    });

    let name_error = use_signal(String::new);
    let amount_error = use_signal(String::new);
    let currency_error = use_signal(String::new);
    let frequency_error = use_signal(String::new);
    let start_date_error = use_signal(String::new);
    let end_date_error = use_signal(String::new);
    let account_error = use_signal(String::new);
    let category_error = use_signal(String::new);

    let on_saved = EventHandler::new(move |_| {
        spawn(async move {
            if let Ok(items) = Storage::calendar_month(current_year(), current_month()).await {
                calendar_items.set(items);
            }
        });
    });

    render_dialog(
        form,
        recurrence,
        active,
        editing_id,
        open,
        name_error,
        amount_error,
        currency_error,
        frequency_error,
        start_date_error,
        end_date_error,
        account_error,
        category_error,
        form_account_id,
        Some(on_saved),
    )
}
