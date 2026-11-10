use super::*;
use crate::components::AccountCombobox;

// ── Dialog ───────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub fn render_dialog(
    mut form: Signal<DialogForm>,
    editing_id: Signal<Option<Uuid>>,
    mut dialog_open: Signal<bool>,
    mut name_error: Signal<String>,
    mut amount_error: Signal<String>,
    mut currency_error: Signal<String>,
    mut frequency_error: Signal<String>,
    mut start_date_error: Signal<String>,
    mut account_error: Signal<String>,
    mut category_error: Signal<String>,
    mut form_account_id: Signal<String>,
) -> Element {
    let is_editing = editing_id().is_some();
    let title = if is_editing {
        "Edit Subscription"
    } else {
        "Add Subscription"
    };
    let desc = if is_editing {
        "Make changes to your subscription here."
    } else {
        "Add a new recurring expense or subscription."
    };
    let submit_label = if is_editing {
        "Save Changes"
    } else {
        "Add Subscription"
    };

    let form_name = form().name;
    let form_amount = form().amount;
    let form_currency = form().currency;
    let form_frequency = form().frequency;
    let form_start_date = form().start_date;
    let form_account = form_account_id();
    let selected_account = form_account.clone();
    let form_category_id = form().category_id;
    let form_notes = form().notes;
    let form_logo_url = form().logo_url;

    let name_err = name_error();
    let amount_err = amount_error();
    let currency_err = currency_error();
    let frequency_err = frequency_error();
    let start_date_err = start_date_error();
    let account_err = account_error();

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| dialog_open.set(false),
            title: title,
            footer: rsx! {
                div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2",
                    button {
                        class: "border px-4 py-2 rounded-md text-sm hover:bg-muted",
                        onclick: move |_| dialog_open.set(false),
                        "Cancel"
                    }
                    button {
                        class: "bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm hover:bg-primary/90",
                        onclick: move |_| {
                            let f = form();
                            let mut valid = true;

                            // Validate name
                            if f.name.trim().is_empty() {
                                name_error.set("Name is required".into());
                                valid = false;
                            } else if f.name.trim().len() < 2 {
                                name_error.set("Name must be at least 2 characters".into());
                                valid = false;
                            }

                            // Validate amount
                            let amount_result = f.amount.parse::<Decimal>();
                            if f.amount.is_empty() {
                                amount_error.set("Amount is required".into());
                                valid = false;
                            } else if amount_result.is_err() || amount_result.as_ref().unwrap_or(&Decimal::ZERO) <= &Decimal::ZERO {
                                amount_error.set("Amount must be greater than 0".into());
                                valid = false;
                            }

                            // Validate currency
                            if f.currency.is_empty() {
                                currency_error.set("Currency is required".into());
                                valid = false;
                            }

                            // Validate frequency
                            if f.frequency.is_empty() {
                                frequency_error.set("Frequency is required".into());
                                valid = false;
                            }

                            // Validate start_date
                            if f.start_date.is_empty() {
                                start_date_error.set("Start date is required".into());
                                valid = false;
                            } else if f.start_date.parse::<chrono::NaiveDate>().is_err() {
                                start_date_error.set("Invalid date format".into());
                                valid = false;
                            }

                            // Validate account
                            if form_account.is_empty() {
                                account_error.set("Account is required".into());
                                valid = false;
                            }

                            if !valid { return; }

                            let amount = f.amount.parse::<Decimal>().unwrap_or(Decimal::ZERO);
                            let account = Uuid::parse_str(&form_account).ok();
                            let category = if f.category_id.is_empty() { None } else { Uuid::parse_str(&f.category_id).ok() };

                            if let Some(id) = editing_id() {
                                let req = UpdateSubscriptionRequest {
                                    account_id: account,
                                    name: Some(f.name),
                                    amount: Some(amount),
                                    currency: Some(f.currency),
                                    frequency: Some(f.frequency),
                                    start_date: Some(f.start_date.parse().unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())),
                                    category_id: category,
                                    notes: if f.notes.is_empty() { None } else { Some(f.notes.clone()) },
                                    logo_url: if f.logo_url.is_empty() { None } else { Some(f.logo_url.clone()) },
                                    ..Default::default()
                                };
                                spawn(async move {
                                    let _ = Storage::update_subscription(&id, &req).await;
                                    match Storage::list_subscriptions().await {
                                        Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                                        Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                                    }
                                });
                            } else {
                                let req = CreateSubscriptionRequest {
                                    account_id: account.unwrap_or(Uuid::nil()),
                                    name: f.name,
                                    amount,
                                    currency: f.currency,
                                    frequency: f.frequency,
                                    start_date: f.start_date.parse().unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
                                    category_id: category,
                                    notes: if f.notes.is_empty() { None } else { Some(f.notes.clone()) },
                                    logo_url: if f.logo_url.is_empty() { None } else { Some(f.logo_url.clone()) },
                                };
                                spawn(async move {
                                    let _ = Storage::create_subscription(&req).await;
                                    match Storage::list_subscriptions().await {
                                        Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                                        Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                                    }
                                });
                            }

                            dialog_open.set(false);
                        },
                        {submit_label.to_string()}                    }
                }
            },
            p { class: "text-sm text-muted-foreground", {desc.to_string()} }
            div { class: "space-y-4",
                // Name
                div { class: "space-y-1.5",
                    label { class: "text-sm font-medium", "Name" }
                    input {
                        class: if name_err.is_empty() { "w-full h-9 rounded-md border bg-background px-3 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-3 text-sm" },
                        placeholder: "Netflix, Spotify, etc.",
                        value: form_name.to_string(),
                        oninput: move |e: Event<FormData>| {
                            form.write().name = e.value();
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
                                form.write().amount = e.value();
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
                                form.write().currency = e.value();
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
                                form.write().frequency = e.value();
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
                                form.write().start_date = e.value();
                                start_date_error.set(String::new());
                            },
                        }
                        if !start_date_err.is_empty() {
                            p { class: "text-xs text-red-500", {start_date_err.to_string()} }
                        }
                    }
                }

                // Account (search or create, same combobox as categories)
                div { class: "space-y-1.5",
                    label { class: "text-sm font-medium", "Account" }
                    AccountCombobox {
                        selected_id: selected_account.clone(),
                        on_change: move |id: String| {
                            form_account_id.set(id);
                            account_error.set(String::new());
                        },
                        error: if account_err.is_empty() { None } else { Some(account_err.to_string()) },
                    }
                }

                // Category (optional)
                div { class: "space-y-1.5",
                    label { class: "text-sm font-medium", "Category (optional)" }
                    {rsx! {
                        crate::components::CategoryCombobox {
                            selected_id: form_category_id,
                            on_change: move |id: String| {
                                form.write().category_id = id;
                                category_error.set(String::new());
                            },
                            error: None,
                        }
                    }}
                }

                // Logo
                div { class: "space-y-1.5",
                    label { class: "text-sm font-medium", "Logo (optional)" }
                    LogoSearch {
                        selected_url: form_logo_url.clone(),
                        default_query: form_name.clone(),
                        on_select: move |url: String| form.write().logo_url = url,
                    }
                }

                // Notes
                div { class: "space-y-1.5",
                    label { class: "text-sm font-medium", "Notes (optional)" }
                    textarea {
                        class: "w-full min-h-[80px] rounded-md border bg-background px-3 py-2 text-sm resize-none",
                        placeholder: "Additional notes...",
                        value: form_notes.to_string(),
                        oninput: move |e: Event<FormData>| form.write().notes = e.value(),
                    }
                }

            }
        }
    }
}


