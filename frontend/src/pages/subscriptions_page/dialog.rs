use super::*;
use chrono::Datelike;
use crate::components::AccountCombobox;

const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September",
    "October", "November", "December",
];

// ── Dialog ───────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_dialog(
    mut form: Signal<DialogForm>,
    editing_id: Signal<Option<Uuid>>,
    mut dialog_open: Signal<bool>,
    mut name_error: Signal<String>,
    mut amount_error: Signal<String>,
    mut currency_error: Signal<String>,
    mut frequency_error: Signal<String>,
    mut start_date_error: Signal<String>,
    mut end_date_error: Signal<String>,
    mut account_error: Signal<String>,
    mut category_error: Signal<String>,
    mut form_account_id: Signal<String>,
    on_saved: Option<EventHandler<()>>,
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
    let form_schedule_day = form().schedule_day;
    let form_schedule_month = form().schedule_month;
    let form_start_date = form().start_date;
    let form_end_date = form().end_date;
    let today = chrono::Utc::now().date_naive();
    let reference_date = form_start_date
        .parse::<chrono::NaiveDate>()
        .unwrap_or(today);
    let default_day = reference_date.day() as i16;
    let default_weekday = reference_date.weekday().number_from_monday() as i16;
    let default_month = reference_date.month() as i16;
    let selected_day = form_schedule_day.unwrap_or(default_day);
    let selected_weekday = form_schedule_day.unwrap_or(default_weekday);
    let selected_month = form_schedule_month.unwrap_or(default_month);
    let form_account = form_account_id();
    let selected_account = form_account.clone();
    let form_category_id = form().category_id;
    let form_notes = form().notes;
    let form_logo_url = form().logo_url;

    let name_err = name_error();
    let amount_err = amount_error();
    let currency_err = currency_error();
    let frequency_err = frequency_error();
    let frequency_label = FREQUENCIES
        .iter()
        .find(|(value, _)| *value == form_frequency)
        .map(|(_, label)| *label)
        .unwrap_or("Monthly");
    let schedule_hint = match form_frequency.as_str() {
        "daily" => "Charges every day.".to_string(),
        "weekly" => format!("Charges every {}", WEEKDAYS[(selected_weekday - 1).clamp(0, 6) as usize]),
        "monthly" => format!("Charges on day {selected_day} of each month."),
        "quarterly" => format!("Charges every 3 months on day {selected_day}."),
        "yearly" => format!("Charges every year on {} {selected_day}.", MONTHS[(selected_month - 1).clamp(0, 11) as usize]),
        _ => "Choose how often this subscription renews.".to_string(),
    };
    let start_date_err = start_date_error();
    let end_date_err = end_date_error();
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

                            // Start date is optional; today is the anchor when omitted.
                            let start_date = f
                                .start_date
                                .parse::<chrono::NaiveDate>()
                                .ok()
                                .or_else(|| {
                                    if f.start_date.is_empty() {
                                        Some(chrono::Utc::now().date_naive())
                                    } else {
                                        start_date_error.set("Invalid date format".into());
                                        valid = false;
                                        None
                                    }
                                });

                            // Validate recurrence details
                            match f.frequency.as_str() {
                                "weekly" if !(1..=7).contains(&f.schedule_day.unwrap_or(1)) => {
                                    frequency_error.set("Choose a weekday".into());
                                    valid = false;
                                }
                                "monthly" | "quarterly"
                                    if !(1..=31).contains(&f.schedule_day.unwrap_or(1)) =>
                                {
                                    frequency_error.set("Choose a day of the month".into());
                                    valid = false;
                                }
                                "yearly"
                                    if !(1..=31).contains(&f.schedule_day.unwrap_or(1))
                                        || !(1..=12).contains(&f.schedule_month.unwrap_or(1)) =>
                                {
                                    frequency_error.set("Choose a month and day".into());
                                    valid = false;
                                }
                                _ => {}
                            }

                            // Validate optional end date
                            let end_date = if f.end_date.is_empty() {
                                None
                            } else {
                                match f.end_date.parse::<chrono::NaiveDate>() {
                                    Ok(end) => Some(end),
                                    Err(_) => {
                                        end_date_error.set("Invalid end date".into());
                                        valid = false;
                                        None
                                    }
                                }
                            };
                            if let (Some(start), Some(end)) = (start_date, end_date)
                                && end < start
                            {
                                end_date_error.set("End date must be after the start date".into());
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
                                    schedule_day: f.schedule_day,
                                    schedule_month: f.schedule_month,
                                    start_date: Some(f.start_date.parse().unwrap_or_else(|_| chrono::Utc::now().date_naive())),
                                    end_date,
                                    clear_end_date: f.end_date.is_empty(),
                                    category_id: category,
                                    notes: Some(f.notes.clone()),
                                    logo_url: if f.logo_url.is_empty() { None } else { Some(f.logo_url.clone()) },
                                    ..Default::default()
                                };
                                spawn(async move {
                                    let _ = Storage::update_subscription(&id, &req).await;
                                    match Storage::list_subscriptions().await {
                                        Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                                        Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                                    }
                                    if let Some(callback) = on_saved {
                                        callback.call(());
                                    }
                                });
                            } else {
                                let req = CreateSubscriptionRequest {
                                    account_id: account.unwrap_or(Uuid::nil()),
                                    name: f.name,
                                    amount,
                                    currency: f.currency,
                                    frequency: f.frequency,
                                    schedule_day: f.schedule_day,
                                    schedule_month: f.schedule_month,
                                    start_date: f.start_date.parse().unwrap_or_else(|_| chrono::Utc::now().date_naive()),
                                    end_date,
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
                                    if let Some(callback) = on_saved {
                                        callback.call(());
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

                // Billing schedule
                div { class: "rounded-lg border border-border/60 bg-background/30 p-4 space-y-4",
                    div { class: "flex flex-wrap items-start justify-between gap-2",
                        div {
                            p { class: "text-sm font-semibold", "Billing schedule" }
                            p { class: "text-xs text-muted-foreground", {schedule_hint} }
                        }
                        span { class: "rounded-full border border-border/60 px-2.5 py-1 text-[11px] font-medium text-muted-foreground", {frequency_label} }
                    }
                    div { class: "grid grid-cols-2 gap-2 sm:grid-cols-5",
                        for (val, label) in FREQUENCIES {
                            {
                                let value = val.to_string();
                                let active = form_frequency == *val;
                                rsx! {
                                    button {
                                        r#type: "button",
                                        class: if active {
                                            "h-9 rounded-md border border-primary/50 bg-primary/15 px-2 text-xs font-semibold text-primary"
                                        } else {
                                            "h-9 rounded-md border border-border/60 px-2 text-xs font-medium text-muted-foreground hover:border-border hover:text-foreground"
                                        },
                                        onclick: move |_| {
                                            let start = form().start_date.parse::<chrono::NaiveDate>().ok();
                                            let (day, month) = match value.as_str() {
                                                "weekly" => (start.map(|d| d.weekday().number_from_monday() as i16), None),
                                                "monthly" | "quarterly" => (start.map(|d| d.day() as i16), None),
                                                "yearly" => (start.map(|d| d.day() as i16), start.map(|d| d.month() as i16)),
                                                _ => (None, None),
                                            };
                                            form.write().frequency = value.clone();
                                            form.write().schedule_day = day;
                                            form.write().schedule_month = month;
                                            frequency_error.set(String::new());
                                        },
                                        {label.to_string()}
                                    }
                                }
                            }
                        }
                    }

                    if form_frequency == "weekly" {
                        div { class: "space-y-1.5",
                            label { class: "text-sm font-medium", "Day of the week" }
                            div { class: "flex flex-wrap gap-1.5",
                                for (idx, day) in WEEKDAYS.iter().enumerate() {
                                    {
                                        let value = (idx + 1) as i16;
                                        let active = selected_weekday == value;
                                        rsx! {
                                            button {
                                                r#type: "button",
                                                class: if active {
                                                    "rounded-md bg-primary px-2.5 py-1 text-xs font-semibold text-primary-foreground"
                                                } else {
                                                    "rounded-md border border-border/60 px-2.5 py-1 text-xs font-medium text-muted-foreground hover:bg-accent"
                                                },
                                                onclick: move |_| {
                                                    form.write().schedule_day = Some(value);
                                                    frequency_error.set(String::new());
                                                },
                                                {day.to_string()}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if form_frequency == "monthly" || form_frequency == "quarterly" {
                        label { class: "block max-w-48 space-y-1.5 text-sm", "Day of the month"
                            select {
                                class: "h-9 w-full rounded-md border border-border/60 bg-background px-2 text-sm",
                                value: selected_day.to_string(),
                                onchange: move |e: Event<FormData>| {
                                    form.write().schedule_day = e.value().parse().ok();
                                    frequency_error.set(String::new());
                                },
                                for day in 1..=31 {
                                    option { value: day.to_string(), selected: selected_day == day, "Day {day}" }
                                }
                            }
                        }
                    } else if form_frequency == "yearly" {
                        div { class: "grid grid-cols-2 gap-4",
                            label { class: "space-y-1.5 text-sm", "Month"
                                select {
                                    class: "h-9 w-full rounded-md border border-border/60 bg-background px-2 text-sm",
                                    value: selected_month.to_string(),
                                    onchange: move |e: Event<FormData>| {
                                        form.write().schedule_month = e.value().parse().ok();
                                        frequency_error.set(String::new());
                                    },
                                    for (idx, month) in MONTHS.iter().enumerate() {
                                        option { value: (idx + 1).to_string(), selected: selected_month == (idx + 1) as i16, {month.to_string()} }
                                    }
                                }
                            }
                            label { class: "space-y-1.5 text-sm", "Day"
                                select {
                                    class: "h-9 w-full rounded-md border border-border/60 bg-background px-2 text-sm",
                                    value: selected_day.to_string(),
                                    onchange: move |e: Event<FormData>| {
                                        form.write().schedule_day = e.value().parse().ok();
                                        frequency_error.set(String::new());
                                    },
                                    for day in 1..=31 {
                                        option { value: day.to_string(), selected: selected_day == day, "Day {day}" }
                                    }
                                }
                            }
                        }
                    } else {
                        p { class: "text-xs text-muted-foreground", "No extra schedule needed. Every day is a renewal day." }
                    }
                    if !frequency_err.is_empty() {
                        p { class: "text-xs text-red-500", {frequency_err.to_string()} }
                    }
                }

                // Start and optional end date
                div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2",
                    div { class: "space-y-1.5",
                        div { class: "flex items-center justify-between gap-2",
                            label { class: "text-sm font-medium", "Start date" }
                            span { class: "text-[11px] text-muted-foreground", "Optional" }
                        }
                        input {
                            r#type: "date",
                            class: if start_date_err.is_empty() { "w-full h-9 rounded-md border bg-background px-3 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-3 text-sm" },
                            value: form_start_date.to_string(),
                            oninput: move |e: Event<FormData>| {
                                form.write().start_date = e.value();
                                start_date_error.set(String::new());
                            },
                        }
                        p { class: "text-[11px] text-muted-foreground", "Leave empty to start from today." }
                        if !start_date_err.is_empty() {
                            p { class: "text-xs text-red-500", {start_date_err.to_string()} }
                        }
                    }
                    div { class: "space-y-1.5",
                        div { class: "flex items-center justify-between gap-2",
                            label { class: "text-sm font-medium", "End date" }
                            span { class: "text-[11px] text-muted-foreground", "Optional" }
                        }
                        div { class: "flex gap-2",
                            input {
                                r#type: "date",
                                class: if end_date_err.is_empty() { "w-full h-9 rounded-md border bg-background px-3 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-3 text-sm" },
                                value: form_end_date.to_string(),
                                oninput: move |e: Event<FormData>| {
                                    form.write().end_date = e.value();
                                    end_date_error.set(String::new());
                                },
                            }
                            if !form_end_date.is_empty() {
                                button {
                                    r#type: "button",
                                    class: "h-9 rounded-md border border-border px-2 text-xs text-muted-foreground hover:text-foreground",
                                    onclick: move |_| {
                                        form.write().end_date = String::new();
                                        end_date_error.set(String::new());
                                    },
                                    "Clear"
                                }
                            }
                        }
                        p { class: "text-[11px] text-muted-foreground", "Billing stops before this date." }
                        if !end_date_err.is_empty() {
                            p { class: "text-xs text-red-500", {end_date_err.to_string()} }
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


