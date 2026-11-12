use super::*;
use chrono::Datelike;
use crate::components::AccountCombobox;

const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const MAX_INTERVAL: u16 = 100;
const MAX_MONTH_DAY: u8 = 31;
const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September",
    "October", "November", "December",
];

// ── Dialog ───────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_dialog(
    mut form: Signal<DialogForm>,
    mut recurrence: Signal<Recurrence>,
    active: Signal<bool>,
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
    let rec = recurrence();
    let rec_interval = rec.interval;
    let rec_unit = rec.unit.clone();
    let rec_days = rec.days.clone();
    let rec_weekdays = rec.weekdays.clone();
    let rec_dates = rec.dates.clone();
    let form_start_date = form().start_date;
    let form_end_date = form().end_date;
    let today = chrono::Utc::now().date_naive();
    let form_account = form_account_id();
    let selected_account = form_account.clone();
    let form_category_id = form().category_id;
    let form_notes = form().notes;
    let form_logo_url = form().logo_url;

    let name_err = name_error();
    let amount_err = amount_error();
    let currency_err = currency_error();
    let frequency_err = frequency_error();
    let schedule_hint = recurrence_hint(&rec);
    let preview_start = form_start_date
        .parse::<chrono::NaiveDate>()
        .unwrap_or(today);
    let first_charge = crate::utils::recurrence::first_occurrence_on_or_after(preview_start, &rec)
        .map(|date| {
            crate::utils::dates::format_date(&date, &PREFERENCES().date_format())
        });
    let unit_options: [(&str, &str); 4] = if rec_interval == 1 {
        [("day", "day"), ("week", "week"), ("month", "month"), ("year", "year")]
    } else {
        [("day", "days"), ("week", "weeks"), ("month", "months"), ("year", "years")]
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
                            let rec = recurrence();
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

                            // Validate recurrence
                            if !(1..=MAX_INTERVAL).contains(&rec.interval) {
                                frequency_error.set(format!("Repeat every must be between 1 and {MAX_INTERVAL}"));
                                valid = false;
                            } else {
                                match rec.unit.as_str() {
                                    "week" if rec.weekdays.is_empty() => {
                                        frequency_error.set("Choose at least one weekday".into());
                                        valid = false;
                                    }
                                    "month" if rec.days.is_empty() => {
                                        frequency_error.set("Choose at least one day of the month".into());
                                        valid = false;
                                    }
                                    "year" if rec.dates.is_empty() => {
                                        frequency_error.set("Add at least one annual date".into());
                                        valid = false;
                                    }
                                    "year" if rec.dates.iter().any(|d| !(1..=12).contains(&d.month) || d.day > 31) => {
                                        frequency_error.set("Check the annual dates".into());
                                        valid = false;
                                    }
                                    "year" if {
                                        let mut dates = rec.dates.clone();
                                        dates.sort_by_key(|date| (date.month, date.day));
                                        let before = dates.len();
                                        dates.dedup_by_key(|date| (date.month, date.day));
                                        dates.len() != before
                                    } => {
                                        frequency_error.set("Annual dates must be unique".into());
                                        valid = false;
                                    }
                                    _ => {}
                                }
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
                                    frequency: Some(rec.legacy_frequency()),
                                    schedule_day: rec.legacy_day(),
                                    schedule_month: rec.legacy_month(),
                                    recurrence: Some(rec.clone()),
                                    is_active: Some(active()),
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
                                    frequency: rec.legacy_frequency(),
                                    schedule_day: rec.legacy_day(),
                                    schedule_month: rec.legacy_month(),
                                    recurrence: Some(rec.clone()),
                                    is_active: Some(active()),
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
                    label { class: "text-sm font-medium", "Name " span { class: "text-destructive", "*" } }
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
                        label { class: "text-sm font-medium", "Amount " span { class: "text-destructive", "*" } }
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
                        label { class: "text-sm font-medium", "Currency " span { class: "text-destructive", "*" } }
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

                // Billing schedule — a single, explicit recurrence control.
                div { class: "rounded-lg border border-border/60 bg-background/30 p-4 space-y-4",
                    div { class: "flex flex-wrap items-start justify-between gap-2",
                        div {
                            p { class: "text-sm font-semibold", "Billing schedule " span { class: "text-destructive", "*" } }
                            p { class: "text-xs text-muted-foreground", {schedule_hint} }
                            if let Some(first) = first_charge {
                                p { class: "text-[11px] font-medium text-primary", "First charge: {first}" }
                            }
                        }
                    }

                    div { class: "flex flex-wrap items-center gap-2",
                        span { class: "text-sm font-medium", "Every" }
                        input {
                            r#type: "number",
                            min: "1",
                            max: MAX_INTERVAL.to_string(),
                            value: rec_interval.to_string(),
                            oninput: move |e: Event<FormData>| {
                                if let Ok(value) = e.value().parse::<u16>() {
                                    recurrence.write().interval = value.clamp(1, 100);
                                    frequency_error.set(String::new());
                                }
                            },
                            class: "h-9 w-20 rounded-md border border-border/60 bg-background px-2 text-sm",
                        }
                        select {
                            class: "h-9 rounded-md border border-border/60 bg-background px-2 text-sm",
                            value: rec_unit.clone(),
                            onchange: move |e: Event<FormData>| {
                                let unit = e.value();
                                let start = form().start_date.parse::<chrono::NaiveDate>().ok().unwrap_or(today);
                                let mut next = recurrence();
                                next.unit = unit.clone();
                                match unit.as_str() {
                                    "week" if next.weekdays.is_empty() => {
                                        next.weekdays = vec![start.weekday().number_from_monday() as u8];
                                    }
                                    "month" if next.days.is_empty() => {
                                        next.days = vec![start.day() as u8];
                                    }
                                    "year" if next.dates.is_empty() => {
                                        next.dates = vec![crate::models::subscription::AnnualDate {
                                            month: start.month() as u8,
                                            day: start.day() as u8,
                                            amount: None,
                                        }];
                                    }
                                    _ => {}
                                }
                                recurrence.set(next);
                                frequency_error.set(String::new());
                            },
                            for (value, label) in unit_options {
                                option { value: value.to_string(), selected: rec_unit == value, {label} }
                            }
                        }
                    }

                    if rec_unit == "week" {
                        div { class: "space-y-1.5",
                            label { class: "text-sm font-medium", "Charge on" }
                            div { class: "flex flex-wrap gap-1.5",
                                for (idx, day) in WEEKDAYS.iter().enumerate() {
                                    {
                                        let value = (idx + 1) as u8;
                                        let active = rec_weekdays.contains(&value);
                                        rsx! {
                                            button {
                                                r#type: "button",
                                                class: if active {
                                                    "rounded-md bg-primary px-2.5 py-1 text-xs font-semibold text-primary-foreground"
                                                } else {
                                                    "rounded-md border border-border/60 px-2.5 py-1 text-xs font-medium text-muted-foreground hover:bg-accent"
                                                },
                                                onclick: move |_| {
                                                    let mut next = recurrence();
                                                    if next.weekdays.contains(&value) {
                                                        next.weekdays.retain(|d| *d != value);
                                                    } else {
                                                        next.weekdays.push(value);
                                                    }
                                                    recurrence.set(next);
                                                    frequency_error.set(String::new());
                                                },
                                                {day.to_string()}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if rec_unit == "month" {
                        div { class: "space-y-1.5",
                            div { class: "flex items-center justify-between gap-2",
                                label { class: "text-sm font-medium", "Charge on these days" }
                                div { class: "flex gap-2 text-xs",
                                    button { r#type: "button", class: "text-primary hover:underline",
                                        onclick: move |_| { let mut next = recurrence(); next.days = (1..=MAX_MONTH_DAY).collect(); recurrence.set(next); frequency_error.set(String::new()); },
                                        "Select all"
                                    }
                                    button { r#type: "button", class: "text-muted-foreground hover:underline",
                                        onclick: move |_| { let mut next = recurrence(); next.days.clear(); recurrence.set(next); },
                                        "Clear"
                                    }
                                }
                            }
                            div { class: "flex flex-wrap gap-1",
                                for day in 1..=MAX_MONTH_DAY {
                                    {
                                        let active = rec_days.contains(&day);
                                        rsx! {
                                            button {
                                                r#type: "button",
                                                class: if active {
                                                    "h-8 w-8 rounded-md bg-primary text-xs font-semibold text-primary-foreground"
                                                } else {
                                                    "h-8 w-8 rounded-md border border-border/60 text-xs font-medium text-muted-foreground hover:bg-accent"
                                                },
                                                onclick: move |_| {
                                                    let mut next = recurrence();
                                                    if next.days.contains(&day) {
                                                        next.days.retain(|d| *d != day);
                                                    } else {
                                                        next.days.push(day);
                                                    }
                                                    recurrence.set(next);
                                                    frequency_error.set(String::new());
                                                },
                                                {day.to_string()}
                                            }
                                        }
                                    }
                                }
                                if rec_days.iter().any(|day| *day >= 29) || rec_days.contains(&0) {
                                    button {
                                        r#type: "button",
                                        class: if rec_days.contains(&0) {
                                            "h-8 rounded-md bg-primary px-2 text-xs font-semibold text-primary-foreground"
                                        } else {
                                            "h-8 rounded-md border border-border/60 px-2 text-xs font-medium text-muted-foreground hover:bg-accent"
                                        },
                                        onclick: move |_| {
                                            let mut next = recurrence();
                                            if next.days.contains(&0) {
                                                next.days.retain(|d| *d != 0);
                                            } else {
                                                next.days.retain(|d| *d < 29);
                                                next.days.push(0);
                                            }
                                            recurrence.set(next);
                                            frequency_error.set(String::new());
                                        },
                                        "Last day"
                                    }
                                }
                            }
                            p { class: "text-[11px] text-muted-foreground",
                                "Days 29-31 use “Last day” for short months. Each selected day is a separate charge."
                            }
                        }
                    } else if rec_unit == "year" {
                        div { class: "space-y-2",
                            label { class: "text-sm font-medium", "Annual dates" }
                            for (index, entry) in rec_dates.iter().enumerate() {
                                {
                                    let idx = index;
                                    let entry = entry.clone();
                                    let day_value = entry.day.to_string();
                                    let amount_value = entry.amount.map(|value| value.to_string()).unwrap_or_default();
                                    rsx! {
                                        div { class: "flex flex-wrap items-center gap-2 rounded-md border border-border/50 p-2",
                                            select {
                                                class: "h-8 rounded-md border border-border/60 bg-background px-2 text-xs",
                                                value: entry.month.to_string(),
                                                onchange: move |e: Event<FormData>| {
                                                    if let Ok(month) = e.value().parse::<u8>() {
                                                        let mut next = recurrence();
                                                        if let Some(date) = next.dates.get_mut(idx) { date.month = month; }
                                                        recurrence.set(next);
                                                    }
                                                },
                                                for (month_idx, month) in MONTHS.iter().enumerate() {
                                                    option { value: (month_idx + 1).to_string(), selected: entry.month as usize == month_idx + 1, {month.to_string()} }
                                                }
                                            }
                                            select {
                                                class: "h-8 rounded-md border border-border/60 bg-background px-2 text-xs",
                                                value: day_value,
                                                onchange: move |e: Event<FormData>| {
                                                    if let Ok(day) = e.value().parse::<u8>() {
                                                        let mut next = recurrence();
                                                        if let Some(date) = next.dates.get_mut(idx) { date.day = day; }
                                                        recurrence.set(next);
                                                    }
                                                },
                                                for day in 1..=MAX_MONTH_DAY {
                                                    option { value: day.to_string(), selected: entry.day == day, "Day {day}" }
                                                }
                                                option { value: "0", selected: entry.day == 0, "Last day" }
                                            }
                                            input {
                                                r#type: "number",
                                                step: "0.01",
                                                min: "0",
                                                placeholder: "Same amount",
                                                value: amount_value,
                                                oninput: move |e: Event<FormData>| {
                                                    let amount = e.value().parse::<Decimal>().ok();
                                                    let mut next = recurrence();
                                                    if let Some(date) = next.dates.get_mut(idx) { date.amount = amount; }
                                                    recurrence.set(next);
                                                },
                                                class: "h-8 w-32 rounded-md border border-border/60 bg-background px-2 text-xs",
                                            }
                                            button {
                                                r#type: "button",
                                                class: "ml-auto h-8 rounded-md px-2 text-xs text-destructive hover:bg-destructive/10",
                                                onclick: move |_| {
                                                    let mut next = recurrence();
                                                    if idx < next.dates.len() { next.dates.remove(idx); }
                                                    recurrence.set(next);
                                                },
                                                "Remove"
                                            }
                                        }
                                    }
                                }
                            }
                            button {
                                r#type: "button",
                                class: "h-8 rounded-md border border-border/60 px-3 text-xs font-medium hover:bg-accent",
                                onclick: move |_| {
                                    let start = form().start_date.parse::<chrono::NaiveDate>().ok().unwrap_or(today);
                                    let mut next = recurrence();
                                    next.dates.push(crate::models::subscription::AnnualDate {
                                        month: start.month() as u8,
                                        day: start.day() as u8,
                                        amount: None,
                                    });
                                    next.dates.sort_by_key(|date| (date.month, date.day));
                                    recurrence.set(next);
                                    frequency_error.set(String::new());
                                },
                                "+ Add annual date"
                            }
                            p { class: "text-[11px] text-muted-foreground",
                                "Leave the amount empty to use the subscription amount. Each date is a separate charge."
                            }
                        }
                    } else {
                        p { class: "text-xs text-muted-foreground", "Charges every selected number of days from the activation date." }
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
                        p { class: "text-[11px] text-muted-foreground", "Last billing date (inclusive)." }
                        if !end_date_err.is_empty() {
                            p { class: "text-xs text-red-500", {end_date_err.to_string()} }
                        }
                    }
                }

                // Account (search or create, same combobox as categories)
                div { class: "space-y-1.5",
                    label { class: "text-sm font-medium", "Account " span { class: "text-destructive", "*" } }
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
                    label { class: "text-sm font-medium", "Category" }
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
                    label { class: "text-sm font-medium", "Logo" }
                    LogoSearch {
                        selected_url: form_logo_url.clone(),
                        default_query: form_name.clone(),
                        on_select: move |url: String| form.write().logo_url = url,
                    }
                }

                // Notes
                div { class: "space-y-1.5",
                    label { class: "text-sm font-medium", "Notes" }
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



fn recurrence_hint(recurrence: &Recurrence) -> String {
    let every = recurrence.interval;
    match recurrence.unit.as_str() {
        "day" => {
            if every == 1 {
                "Charges every day.".to_string()
            } else {
                format!("Charges every {every} days.")
            }
        }
        "week" => {
            let mut days = recurrence.weekdays.clone();
            days.sort_unstable();
            days.dedup();
            let names = days
                .iter()
                .filter_map(|day| WEEKDAYS.get(*day as usize - 1))
                .copied()
                .collect::<Vec<_>>()
                .join(", ");
            if every == 1 {
                format!("Charges every week on {names}.")
            } else {
                format!("Charges every {every} weeks on {names}.")
            }
        }
        "month" => {
            let mut days = recurrence.days.clone();
            days.sort_unstable();
            days.dedup();
            let names = days
                .iter()
                .map(|day| {
                    if *day == 0 {
                        "the last day".to_string()
                    } else {
                        format!("day {day}")
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            if every == 1 {
                format!("Charges monthly on {names}.")
            } else {
                format!("Charges every {every} months on {names}.")
            }
        }
        _ => {
            let mut dates = recurrence.dates.clone();
            dates.sort_by_key(|date| (date.month, date.day));
            let names = dates
                .iter()
                .map(|date| {
                    let day = if date.day == 0 {
                        "last day".to_string()
                    } else {
                        date.day.to_string()
                    };
                    let month = MONTHS
                        .get(date.month as usize - 1)
                        .copied()
                        .unwrap_or("month");
                    format!("{day} {month}")
                })
                .collect::<Vec<_>>()
                .join(", ");
            if every == 1 {
                format!("Charges every year on {names}.")
            } else {
                format!("Charges every {every} years on {names}.")
            }
        }
    }
}
