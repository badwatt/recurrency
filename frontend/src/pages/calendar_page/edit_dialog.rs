use crate::components::entry_form::{EntryForm as EditForm, EntityType, validate_entry_form};
use crate::components::CategoryCombobox;
use crate::models::subscription::{
    CalendarItem, CreateSubscriptionRequest, UpdateSubscriptionRequest,
};
use crate::state::auth::AUTH;
use crate::state::categories::CATEGORIES;
use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::storage::Storage;
use crate::utils::constants::{CURRENCIES, FREQUENCIES};
use dioxus::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

/// Create/edit subscription dialog. Mounted fresh each time it opens, so the
/// form is initialised from `editing_id` (None = create, using `prefill_date`).
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
            .find(|s| s.id == id)
            .map(|s| {
                EditForm::from_entry(
                    &s.name,
                    &s.amount.to_string(),
                    &s.currency,
                    &s.frequency,
                    &s.start_date.to_string(),
                    &s.category_id.to_string(),
                    &s.notes.clone().unwrap_or_default(),
                    &s.logo_url.clone().unwrap_or_default(),
                )
            })
            .unwrap_or_default()
    } else {
        let default_cat = CATEGORIES()
            .default_category()
            .map(|c| c.id.to_string())
            .unwrap_or_default();
        let date = prefill_date().unwrap_or_default();
        EditForm::init_for_create(
            &date,
            AUTH().currency(),
            &default_cat,
            EntityType::Subscription,
        )
    };
    let mut edit_form = use_signal(move || initial);

    let mut name_error: Signal<String> = use_signal(String::new);
    let mut amount_error: Signal<String> = use_signal(String::new);
    let mut currency_error: Signal<String> = use_signal(String::new);
    let mut frequency_error: Signal<String> = use_signal(String::new);
    let mut start_date_error: Signal<String> = use_signal(String::new);
    let mut category_error: Signal<String> = use_signal(String::new);
    let mut saving = use_signal(|| false);

    let handle_edit_submit = move |_| {
        if saving() {
            return;
        }
        let f = edit_form();
        let result = validate_entry_form(
            &f.name,
            &f.amount,
            &f.currency,
            &f.frequency,
            &f.start_date,
            &f.category_id,
            EntityType::Subscription,
        );
        if !result.is_valid() {
            name_error.set(result.name_error.unwrap_or_default());
            amount_error.set(result.amount_error.unwrap_or_default());
            currency_error.set(result.currency_error.unwrap_or_default());
            frequency_error.set(result.frequency_error.unwrap_or_default());
            start_date_error.set(result.start_date_error.unwrap_or_default());
            category_error.set(result.category_error.unwrap_or_default());
            return;
        }

        saving.set(true);
        let amount = f.amount.parse::<Decimal>().unwrap_or(dec!(0));
        let category = if f.category_id.is_empty() {
            None
        } else {
            Uuid::parse_str(&f.category_id).ok()
        };

        if let Some(id) = editing_id() {
            let req = UpdateSubscriptionRequest {
                name: Some(f.name),
                amount: Some(amount),
                currency: Some(f.currency),
                frequency: Some(f.frequency),
                start_date: Some(f.start_date.parse().unwrap_or(
                    chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
                        .expect("2024-01-01 is a valid date"),
                )),
                category_id: category,
                notes: if f.notes.is_empty() {
                    None
                } else {
                    Some(f.notes.clone())
                },
                logo_url: if f.logo_url.is_empty() {
                    None
                } else {
                    Some(f.logo_url.clone())
                },
                ..Default::default()
            };
            // Refresh everything before closing: the dialog is a child
            // component, and closing it cancels any task still in flight.
            spawn(async move {
                let _ = Storage::update_subscription(&id, &req).await;
                match Storage::list_subscriptions().await {
                    Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                    Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                }
                if let Ok(items) =
                    Storage::calendar_month(current_year(), current_month()).await
                {
                    calendar_items.set(items);
                }
                saving.set(false);
                open.set(false);
            });
        } else {
            let req = CreateSubscriptionRequest {
                name: f.name,
                amount,
                currency: f.currency,
                frequency: f.frequency,
                start_date: f.start_date.parse().unwrap_or(
                    chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
                        .expect("2024-01-01 is a valid date"),
                ),
                category_id: category,
                notes: if f.notes.is_empty() {
                    None
                } else {
                    Some(f.notes.clone())
                },
                logo_url: if f.logo_url.is_empty() {
                    None
                } else {
                    Some(f.logo_url.clone())
                },
            };
            spawn(async move {
                let _ = Storage::create_subscription(&req).await;
                match Storage::list_subscriptions().await {
                    Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                    Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                }
                if let Ok(items) =
                    Storage::calendar_month(current_year(), current_month()).await
                {
                    calendar_items.set(items);
                }
                saving.set(false);
                open.set(false);
            });
        }
    };

    let form_name = edit_form().name;
    let form_amount = edit_form().amount;
    let form_currency = edit_form().currency;
    let form_frequency = edit_form().frequency;
    let form_start_date = edit_form().start_date;
    let form_category_id = edit_form().category_id;
    let form_notes = edit_form().notes;

    let name_err = name_error();
    let amount_err = amount_error();
    let currency_err = currency_error();
    let frequency_err = frequency_error();
    let start_date_err = start_date_error();
    let category_err = category_error();

    let is_editing = editing_id().is_some();
    let edit_title = if is_editing {
        "Edit Subscription".to_string()
    } else {
        "Add Subscription".to_string()
    };

    rsx! {
        div {
            class: "fixed inset-0 z-[60] flex items-center justify-center bg-black/50",
            onclick: move |_| {
                if !saving() {
                    open.set(false);
                }
            },
            div {
                class: "bg-background rounded-none sm:rounded-lg p-4 sm:p-6 w-full sm:max-w-[425px] shadow-lg sm:mx-4 h-dvh sm:h-auto sm:max-h-[85vh] flex flex-col",
                onclick: move |e| e.stop_propagation(),
                h2 { class: "text-lg font-semibold", {edit_title.to_string()} }
                p { class: "text-sm text-muted-foreground mb-4", "Make changes to your subscription here." }

                div { class: "space-y-4",
                    // Name
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Name" }
                        input {
                            class: if name_err.is_empty() { "w-full h-9 rounded-md border bg-background px-3 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-3 text-sm" },
                            placeholder: "Netflix, Spotify, etc.",
                            value: form_name.to_string(),
                            oninput: move |e: Event<FormData>| {
                                edit_form.write().name = e.value();
                                name_error.set(String::new());
                            },
                        }
                        if !name_err.is_empty() {
                            p { class: "text-xs text-red-500", {name_err.to_string()} }
                        }
                    }

                    // Amount + Currency
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "space-y-1.5",
                            label { class: "text-sm font-medium", "Amount" }
                            input {
                                r#type: "number",
                                step: "0.01",
                                min: "0",
                                class: if amount_err.is_empty() { "w-full h-9 rounded-md border bg-background px-3 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-3 text-sm" },
                                placeholder: "9.99",
                                value: form_amount.to_string(),
                                oninput: move |e: Event<FormData>| {
                                    edit_form.write().amount = e.value();
                                    amount_error.set(String::new());
                                },
                            }
                            if !amount_err.is_empty() {
                                p { class: "text-xs text-red-500", {amount_err.to_string()} }
                            }
                        }
                        div { class: "space-y-1.5",
                            label { class: "text-sm font-medium", "Currency" }
                            select {
                                class: if currency_err.is_empty() { "w-full h-9 rounded-md border bg-background px-2 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-2 text-sm" },
                                value: form_currency.to_string(),
                                onchange: move |e: Event<FormData>| {
                                    edit_form.write().currency = e.value();
                                    currency_error.set(String::new());
                                },
                                for (val, label) in CURRENCIES {
                                    option { value: val.to_string(), {label.to_string()} }
                                }
                            }
                            if !currency_err.is_empty() {
                                p { class: "text-xs text-red-500", {currency_err.to_string()} }
                            }
                        }
                    }

                    // Frequency + Start date
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "space-y-1.5",
                            label { class: "text-sm font-medium", "Frequency" }
                            select {
                                class: if frequency_err.is_empty() { "w-full h-9 rounded-md border bg-background px-2 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-2 text-sm" },
                                value: form_frequency.to_string(),
                                onchange: move |e: Event<FormData>| {
                                    edit_form.write().frequency = e.value();
                                    frequency_error.set(String::new());
                                },
                                for (val, label) in FREQUENCIES {
                                    option { value: val.to_string(), selected: form_frequency == *val, {label.to_string()} }
                                }
                            }
                            if !frequency_err.is_empty() {
                                p { class: "text-xs text-red-500", {frequency_err.to_string()} }
                            }
                        }
                        div { class: "space-y-1.5",
                            label { class: "text-sm font-medium", "Start Date" }
                            input {
                                r#type: "date",
                                class: if start_date_err.is_empty() { "w-full h-9 rounded-md border bg-background px-3 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-3 text-sm" },
                                value: form_start_date.to_string(),
                                oninput: move |e: Event<FormData>| {
                                    edit_form.write().start_date = e.value();
                                    start_date_error.set(String::new());
                                },
                            }
                            if !start_date_err.is_empty() {
                                p { class: "text-xs text-red-500", {start_date_err.to_string()} }
                            }
                        }
                    }

                    // Category
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Category" }
                        {rsx! {
                            CategoryCombobox {
                                selected_id: form_category_id,
                                on_change: move |id: String| {
                                    edit_form.write().category_id = id;
                                    category_error.set(String::new());
                                },
                                error: if category_err.is_empty() { None } else { Some(category_err) },
                            }
                        }}
                    }

                    // Notes
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Notes (optional)" }
                        textarea {
                            class: "w-full min-h-[80px] rounded-md border bg-background px-3 py-2 text-sm resize-none",
                            placeholder: "Additional notes...",
                            value: form_notes.to_string(),
                            oninput: move |e: Event<FormData>| edit_form.write().notes = e.value(),
                        }
                    }
                }

                // Footer
                div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2 mt-6",
                    button {
                        class: "border px-4 py-2 rounded-md text-sm hover:bg-muted disabled:opacity-50",
                        disabled: saving(),
                        onclick: move |_| {
                            if !saving() {
                                open.set(false);
                            }
                        },
                        "Cancel"
                    }
                    button {
                        class: "bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm hover:bg-primary/90 disabled:opacity-50",
                        disabled: saving(),
                        onclick: handle_edit_submit,
                        if saving() { "Saving..." } else { "Save Changes" }
                    }
                }
            }
        }
    }
}
