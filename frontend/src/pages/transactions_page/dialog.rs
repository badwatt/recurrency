use crate::components::AccountCombobox;
use crate::components::button::{Button, ButtonVariant};
use crate::components::dialog::Dialog;
use crate::components::date_picker::DatePicker;
use crate::components::field::Field;
use crate::components::input::Input;
use crate::components::textarea::Textarea;
use crate::components::logo_search::LogoSearch;
use crate::components::CategoryCombobox;
use crate::components::select_menu::{SelectMenu, SelectOption};
use crate::models::transaction::{CreateTransactionRequest, UpdateTransactionRequest};
use crate::state::accounts::ACCOUNTS;
use crate::state::auth::AUTH;
use crate::state::categories::CATEGORIES;
use crate::state::transactions::TRANSACTIONS;
use crate::storage::Storage;
use crate::utils::constants::CURRENCIES;
use dioxus::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

/// Create/edit transaction dialog. Mounted fresh each time it opens, so its
/// form state is initialised from `editing_id` (None = create).
#[component]
pub fn TransactionDialog(
    open: Signal<bool>,
    editing_id: Signal<Option<Uuid>>,
) -> Element {
    let existing = editing_id().and_then(|id| {
        TRANSACTIONS()
            .transactions
            .iter()
            .find(|t| t.id == id)
            .cloned()
    });

    let init_name = existing.as_ref().map(|t| t.name.clone()).unwrap_or_default();
    let init_amount = existing
        .as_ref()
        .map(|t| t.amount.to_string())
        .unwrap_or_default();
    let init_currency = existing
        .as_ref()
        .map(|t| t.currency.clone())
        .unwrap_or_else(|| AUTH().currency().to_string());
    let init_date = existing
        .as_ref()
        .map(|t| t.date.to_string())
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let init_category = existing
        .as_ref()
        .map(|t| Some(t.category_id))
        .unwrap_or_else(|| CATEGORIES().default_category().map(|c| c.id));
    let init_account = existing
        .as_ref()
        .map(|t| t.account_id)
        .filter(|id| !id.is_nil())
        .or_else(|| ACCOUNTS().default_account().map(|a| a.id))
        .unwrap_or_else(Uuid::nil);
    let init_notes = existing
        .as_ref()
        .and_then(|t| t.notes.clone())
        .unwrap_or_default();
    let init_logo = existing
        .as_ref()
        .and_then(|t| t.logo_url.clone())
        .unwrap_or_default();

    let mut form_name = use_signal(move || init_name);
    let mut form_amount = use_signal(move || init_amount);
    let mut form_currency = use_signal(move || init_currency);
    let mut form_date = use_signal(move || init_date);
    let mut form_category_id = use_signal(move || init_category);
    let mut form_account_id = use_signal(move || init_account);
    let mut form_notes = use_signal(move || init_notes);
    let mut form_logo_url = use_signal(move || init_logo);
    let mut form_saving = use_signal(|| false);

    let mut name_error = use_signal(String::new);
    let mut amount_error = use_signal(String::new);
    let mut currency_error = use_signal(String::new);
    let mut date_error = use_signal(String::new);
    let mut account_error = use_signal(String::new);
    let mut category_error = use_signal(String::new);

    use_future(move || async move {
        if ACCOUNTS().accounts.is_empty() && !ACCOUNTS().is_loading {
            ACCOUNTS.write().set_loading();
            match Storage::list_accounts().await {
                Ok(data) => ACCOUNTS.write().set_data(data),
                Err(e) => ACCOUNTS.write().set_error(e.to_string()),
            }
        }
        if form_account_id().is_nil()
            && let Some(default) = ACCOUNTS().default_account()
        {
            form_account_id.set(default.id);
        }
    });

    let handle_save = move |_| {
        let mut valid = true;
        let name_val = form_name();
        let amount_str = form_amount();
        let currency_val = form_currency();
        let date_str = form_date();
        let account_id_val = form_account_id();
        let cat_id_val = form_category_id();

        if name_val.trim().is_empty() {
            name_error.set("Name is required".into());
            valid = false;
        } else if name_val.trim().len() < 2 {
            name_error.set("Name must be at least 2 characters".into());
            valid = false;
        }

        if amount_str.is_empty() {
            amount_error.set("Amount is required".into());
            valid = false;
        } else {
            match Decimal::from_str(&amount_str) {
                Ok(d) if d > Decimal::ZERO => {}
                Ok(_) => {
                    amount_error.set("Amount must be greater than 0".into());
                    valid = false;
                }
                Err(_) => {
                    amount_error.set("Invalid amount".into());
                    valid = false;
                }
            }
        }

        if currency_val.is_empty() {
            currency_error.set("Currency is required".into());
            valid = false;
        }

        if date_str.is_empty() {
            date_error.set("Date is required".into());
            valid = false;
        } else if chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").is_err() {
            date_error.set("Invalid date format".into());
            valid = false;
        }

        if account_id_val.is_nil() {
            account_error.set("Account is required".into());
            valid = false;
        }

        if !valid {
            return;
        }

        let amount = match Decimal::from_str(&amount_str) {
            Ok(d) => d,
            Err(_) => return,
        };
        let date = match chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => return,
        };
        let editing = editing_id();
        let account_id = account_id_val;
        let cat_id = cat_id_val;

        form_saving.set(true);
        spawn(async move {
            let result = if let Some(id) = editing {
                Storage::update_transaction(
                    &id,
                    &UpdateTransactionRequest {
                        account_id: if account_id.is_nil() { None } else { Some(account_id) },
                        name: Some(name_val),
                        amount: Some(amount),
                        currency: Some(form_currency()),
                        date: Some(date),
                        category_id: cat_id,
                        notes: Some(form_notes()),
                        logo_url: if form_logo_url().is_empty() {
                            None
                        } else {
                            Some(form_logo_url())
                        },
                    },
                )
                .await
            } else {
                Storage::create_transaction(&CreateTransactionRequest {
                    account_id,
                    name: name_val,
                    amount,
                    currency: form_currency(),
                    date,
                    category_id: cat_id,
                    notes: if form_notes().is_empty() {
                        None
                    } else {
                        Some(form_notes())
                    },
                    logo_url: if form_logo_url().is_empty() {
                        None
                    } else {
                        Some(form_logo_url())
                    },
                })
                .await
            };

            match result {
                Ok(_) => {
                    // Refresh before closing: closing unmounts this component
                    // and Dioxus cancels its spawned tasks, so a refresh that
                    // started after `open.set(false)` would never finish.
                    match Storage::list_transactions().await {
                        Ok(data) => TRANSACTIONS.write().set_data(data),
                        Err(e) => TRANSACTIONS.write().set_error(e.to_string()),
                    }
                    form_saving.set(false);
                    open.set(false);
                }
                Err(e) => {
                    log::error!("Save failed: {}", e);
                    form_saving.set(false);
                }
            }
        });
    };

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| {
                if !form_saving() {
                    open.set(false);
                }
            },
            title: if editing_id().is_some() { "Edit Transaction" } else { "Add Transaction" },
            footer: rsx! {
                div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        disabled: form_saving(),
                        onclick: move |_| {
                            if !form_saving() {
                                open.set(false);
                            }
                        },
                        "Cancel"
                    }
                    Button {
                        disabled: form_saving(),
                        onclick: handle_save,
                        if form_saving() { "Saving..." } else { "Save" }
                    }
                }
            },
            div { class: "space-y-4",
                // Name
                Field { label: "Name", required: true, error: name_error(),
                    Input {
                        value: form_name.to_string(),
                        on_input: move |value| {
                            form_name.set(value);
                            name_error.set(String::new());
                        },
                        class: if name_error().is_empty() { "" } else { "border-red-500" },
                    }
                }
                // Amount
                div { class: "grid grid-cols-2 gap-4",
                    Field { label: "Amount", required: true, error: amount_error(),
                        Input {
                            r#type: "number",
                            step: "0.01",
                            value: form_amount.to_string(),
                            on_input: move |value| {
                                form_amount.set(value);
                                amount_error.set(String::new());
                            },
                            class: if amount_error().is_empty() { "" } else { "border-red-500" },
                        }
                    }
                    Field { label: "Currency", required: true, error: currency_error(),
                        SelectMenu {
                            value: form_currency.to_string(),
                            options: CURRENCIES.iter().map(|(value, label)| SelectOption::new(*value, *label)).collect::<Vec<_>>(),
                            on_change: move |value: String| {
                                form_currency.set(value);
                                currency_error.set(String::new());
                            },
                            searchable: true,
                            placeholder: if currency_error().is_empty() { "Select currency..." } else { "Currency is required" },
                        }
                    }
                }
                // Date
                Field  { label: "Date", required: true, error: date_error(),
                    DatePicker {
                        value: form_date.to_string(),
                        on_input: move |value| {
                            form_date.set(value);
                            date_error.set(String::new());
                        },
                        class: if date_error().is_empty() { "" } else { "border-red-500" },
                    }
                }
                // Account (search or create, same combobox as categories)
                Field { label: "Account", required: true,
                    AccountCombobox {
                        selected_id: form_account_id().to_string(),
                        on_change: move |id: String| {
                            form_account_id.set(Uuid::parse_str(&id).unwrap_or(Uuid::nil()));
                            account_error.set(String::new());
                        },
                        error: if account_error().is_empty() { None } else { Some(account_error()) },
                    }
                }
                // Category (optional)
                Field { label: "Category",
                    {rsx! {
                        CategoryCombobox {
                            selected_id: form_category_id().map(|id| id.to_string()).unwrap_or_default(),
                            on_change: move |id: String| {
                                form_category_id.set(if id.is_empty() { None } else { Uuid::parse_str(&id).ok() });
                                category_error.set(String::new());
                            },
                            error: None,
                        }
                    }}
                }
                // Notes
                Field { label: "Notes",
                    Textarea {
                        value: form_notes.to_string(),
                        on_input: move |value| form_notes.set(value),
                        class: "min-h-[80px]",
                    }
                }
                // Logo
                Field { label: "Logo",
                    LogoSearch {
                        selected_url: form_logo_url(),
                        default_query: form_name(),
                        on_select: move |url: String| form_logo_url.set(url),
                    }
                }
            }
        }
    }
}
