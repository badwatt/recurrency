use crate::models::account::{Account, CreateAccountRequest, UpdateAccountRequest};
use crate::models::category::{Category, CreateCategoryRequest, UpdateCategoryRequest};
use crate::models::subscription::{CalendarItem, CreateSubscriptionRequest, Recurrence, Subscription, UpdateSubscriptionRequest};
use crate::models::transaction::{CreateTransactionRequest, Transaction, UpdateTransactionRequest};
use crate::models::debt::{CreateDebtRequest, Debt, DebtPayment, RecordDebtPaymentRequest, UpdateDebtRequest};
use crate::api::client::ApiError;
use chrono::{Datelike, NaiveDate, Utc};

#[allow(unused_imports)]
use rust_decimal::Decimal;
use uuid::Uuid;

// ── Local JSON storage ────────────────────────────────────────
// Stores data in:
//   Native: {app_data_dir}/recurrency/local_{collection}.json
//   WASM:   localStorage keys "recurrency_local_{collection}"
//
// Thread-safe via std::sync::Mutex for the in-memory cache.

static SUBSCRIPTIONS: std::sync::Mutex<Option<Vec<Subscription>>> = std::sync::Mutex::new(None);
static TRANSACTIONS: std::sync::Mutex<Option<Vec<Transaction>>> = std::sync::Mutex::new(None);
static CATEGORIES: std::sync::Mutex<Option<Vec<Category>>> = std::sync::Mutex::new(None);
static ACCOUNTS: std::sync::Mutex<Option<Vec<Account>>> = std::sync::Mutex::new(None);
static DEBTS: std::sync::Mutex<Option<Vec<Debt>>> = std::sync::Mutex::new(None);
static DEBT_PAYMENTS: std::sync::Mutex<Option<Vec<DebtPayment>>> = std::sync::Mutex::new(None);

// ── Default categories ──

/// The only category created automatically. It is the permanent fallback for
/// subscriptions and transactions that do not have a user category.
const DEFAULT_CATEGORY_NAME: &str = "Uncategorized";

fn default_categories() -> Vec<Category> {
    vec![Category {
        id: Uuid::new_v4(),
        user_id: uuid::Uuid::nil(),
        name: DEFAULT_CATEGORY_NAME.to_string(),
        icon: None,
        color: None,
        is_system: true,
        is_default: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }]
}

// ── Persistence helpers ──

#[cfg(not(target_arch = "wasm32"))]
fn data_dir() -> Option<std::path::PathBuf> {
    crate::state::auth::get_app_data_dir().map(|d| d.join("recurrency"))
}

#[cfg(not(target_arch = "wasm32"))]
fn load_collection<T: serde::de::DeserializeOwned>(name: &str) -> Option<Vec<T>> {
    let dir = data_dir()?;
    let file = dir.join(format!("local_{}.json", name));
    std::fs::read_to_string(&file).ok().and_then(|s| serde_json::from_str(&s).ok())
}

#[cfg(not(target_arch = "wasm32"))]
fn save_collection<T: serde::Serialize>(name: &str, data: &Vec<T>) -> Result<(), ApiError> {
    let dir = data_dir().ok_or_else(|| ApiError::Network("No data directory".into()))?;
    std::fs::create_dir_all(&dir).map_err(|e| ApiError::Network(e.to_string()))?;
    let json = serde_json::to_string_pretty(data).map_err(|e| ApiError::Parse(e.to_string()))?;
    let file = dir.join(format!("local_{}.json", name));
    std::fs::write(&file, json).map_err(|e| ApiError::Network(e.to_string()))
}

#[cfg(target_arch = "wasm32")]
fn load_collection<T: serde::de::DeserializeOwned>(name: &str) -> Option<Vec<T>> {
    let key = format!("recurrency_local_{}", name);
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(json)) = storage.get_item(&key) {
                return serde_json::from_str(&json).ok();
            }
        }
    }
    None
}

#[cfg(target_arch = "wasm32")]
fn save_collection<T: serde::Serialize>(name: &str, data: &Vec<T>) -> Result<(), ApiError> {
    let key = format!("recurrency_local_{}", name);
    let json = serde_json::to_string(data).map_err(|e| ApiError::Parse(e.to_string()))?;
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            storage.set_item(&key, &json).map_err(|e| ApiError::Network(format!("localStorage error: {:?}", e)))?;
        }
    }
    Ok(())
}

// ── Subscription operations ──

pub async fn list_subscriptions() -> Result<Vec<Subscription>, ApiError> {
    ensure_accounts().await?;
    let mut cache = SUBSCRIPTIONS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("subscriptions");
    }
    let mut list = cache.clone().unwrap_or_default();
    let today = chrono::Utc::now().date_naive();
    let mut changed = false;
    for subscription in &mut list {
        // Upgrade records written before recurring schedules were introduced.
        let legacy_recurrence = subscription.recurrence == Recurrence::default()
            && (subscription.frequency != "monthly"
                || subscription.start_date.day() != 1
                || subscription.schedule_day.is_some_and(|day| day != 1)
                || subscription.schedule_month.is_some());
        if legacy_recurrence {
            subscription.recurrence = Recurrence::from_legacy(
                &subscription.frequency,
                subscription.start_date,
                subscription.schedule_day,
                subscription.schedule_month,
            );
            changed = true;
        }
        if subscription.is_active
            && subscription
                .next_billing_date
                .is_none_or(|date| date < today)
        {
            let next = crate::utils::recurrence::next_occurrence_on_or_after(
                subscription.start_date,
                &subscription.recurrence,
                today,
                subscription.end_date,
            );
            if next != subscription.next_billing_date {
                subscription.next_billing_date = next;
                changed = true;
            }
        }
    }
    if changed {
        save_collection("subscriptions", &list)?;
        *cache = Some(list.clone());
    }
    Ok(list)
}

fn days_in_month_local(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or(28)
}

fn add_months_local(date: chrono::NaiveDate, months: i32, day: i16) -> chrono::NaiveDate {
    let total = date.year() * 12 + date.month() as i32 - 1 + months;
    let year = total.div_euclid(12);
    let month = (total.rem_euclid(12) + 1) as u32;
    let day = (day.max(1) as u32).min(days_in_month_local(year, month));
    chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap_or(date)
}

fn advance_scheduled_local(
    date: chrono::NaiveDate,
    frequency: &str,
    schedule_day: Option<i16>,
    schedule_month: Option<i16>,
) -> chrono::NaiveDate {
    match frequency {
        "daily" => date + chrono::Duration::days(1),
        "weekly" => date + chrono::Duration::weeks(1),
        "monthly" => add_months_local(date, 1, schedule_day.unwrap_or(date.day() as i16)),
        "quarterly" => add_months_local(date, 3, schedule_day.unwrap_or(date.day() as i16)),
        "yearly" => {
            let year = date.year() + 1;
            let month = schedule_month.unwrap_or(date.month() as i16) as u32;
            let day = (schedule_day.unwrap_or(date.day() as i16).max(1) as u32)
                .min(days_in_month_local(year, month));
            chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap_or(date)
        }
        _ => date + chrono::Duration::days(30),
    }
}

fn first_scheduled_local(
    start: chrono::NaiveDate,
    frequency: &str,
    schedule_day: Option<i16>,
    schedule_month: Option<i16>,
) -> chrono::NaiveDate {
    match frequency {
        "weekly" => {
            let target = schedule_day.unwrap_or(start.weekday().number_from_monday() as i16);
            let current = start.weekday().number_from_monday() as i16;
            start + chrono::Duration::days((target - current).rem_euclid(7) as i64)
        }
        "monthly" | "quarterly" => {
            let day = schedule_day.unwrap_or(start.day() as i16);
            let day = (day.max(1) as u32).min(days_in_month_local(start.year(), start.month()));
            let candidate = chrono::NaiveDate::from_ymd_opt(start.year(), start.month(), day)
                .unwrap_or(start);
            if candidate < start {
                add_months_local(candidate, 1, day as i16)
            } else {
                candidate
            }
        }
        "yearly" => {
            let day = schedule_day.unwrap_or(start.day() as i16);
            let month = schedule_month.unwrap_or(start.month() as i16) as u32;
            let day = (day.max(1) as u32).min(days_in_month_local(start.year(), month));
            let candidate =
                chrono::NaiveDate::from_ymd_opt(start.year(), month, day).unwrap_or(start);
            if candidate < start {
                chrono::NaiveDate::from_ymd_opt(
                    start.year() + 1,
                    month,
                    (day).min(days_in_month_local(start.year() + 1, month)),
                )
                .unwrap_or(start)
            } else {
                candidate
            }
        }
        _ => start,
    }
}

fn next_billing_local(
    start: chrono::NaiveDate,
    frequency: &str,
    schedule_day: Option<i16>,
    schedule_month: Option<i16>,
    end_date: Option<chrono::NaiveDate>,
) -> Option<chrono::NaiveDate> {
    let today = chrono::Utc::now().date_naive();
    let mut date = first_scheduled_local(start, frequency, schedule_day, schedule_month);
    let mut guard = 0;
    while date < today && guard < 10_000 {
        date = advance_scheduled_local(date, frequency, schedule_day, schedule_month);
        guard += 1;
    }
    if end_date.is_some_and(|end| date >= end) {
        None
    } else {
        Some(date)
    }
}

pub async fn create_subscription(req: &CreateSubscriptionRequest) -> Result<Subscription, ApiError> {
    ensure_accounts().await?;
    let account_id = if req.account_id.is_nil() {
        default_account_id()?
    } else {
        req.account_id
    };
    let recurrence = req
        .recurrence
        .clone()
        .unwrap_or_else(|| Recurrence::from_legacy(
            &req.frequency,
            req.start_date,
            req.schedule_day,
            req.schedule_month,
        ));
    let next_billing_date = crate::utils::recurrence::next_occurrence_on_or_after(
        req.start_date,
        &recurrence,
        chrono::Utc::now().date_naive(),
        req.end_date,
    );
    let sub = Subscription {
        account_id,
        id: Uuid::new_v4(),
        user_id: Uuid::nil(),
        category_id: req.category_id.unwrap_or(Uuid::nil()),
        name: req.name.clone(),
        amount: req.amount,
        currency: req.currency.clone(),
        frequency: req.frequency.clone(),
        schedule_day: req.schedule_day,
        schedule_month: req.schedule_month,
        recurrence,
        start_date: req.start_date,
        end_date: req.end_date,
        next_billing_date,
        is_active: req.is_active.unwrap_or(true),
        notes: req.notes.clone(),
        logo_url: req.logo_url.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let mut cache = SUBSCRIPTIONS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("subscriptions");
    }
    let list = cache.get_or_insert_with(Vec::new);
    list.push(sub.clone());
    save_collection("subscriptions", list)?;
    Ok(sub)
}

pub async fn update_subscription(id: &Uuid, req: &UpdateSubscriptionRequest) -> Result<Subscription, ApiError> {
    let mut cache = SUBSCRIPTIONS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("subscriptions");
    }
    let list = cache.as_mut().ok_or_else(|| ApiError::Network("No subscriptions".into()))?;
    let sub = list.iter_mut().find(|s| &s.id == id).ok_or_else(|| ApiError::Http(404, "Not found".into()))?;
    if let Some(name) = &req.name { sub.name = name.clone(); }
    if let Some(amount) = req.amount { sub.amount = amount; }
    if let Some(currency) = &req.currency { sub.currency = currency.clone(); }
    if let Some(recurrence) = &req.recurrence {
        sub.recurrence = recurrence.clone();
    } else if req.frequency.is_some() || req.schedule_day.is_some() || req.schedule_month.is_some() {
        let frequency = req.frequency.clone().unwrap_or_else(|| sub.frequency.clone());
        sub.recurrence = Recurrence::from_legacy(
            &frequency,
            req.start_date.unwrap_or(sub.start_date),
            req.schedule_day.or(sub.schedule_day),
            req.schedule_month.or(sub.schedule_month),
        );
    }
    if let Some(frequency) = &req.frequency { sub.frequency = frequency.clone(); }
    if let Some(day) = req.schedule_day { sub.schedule_day = Some(day); }
    if let Some(month) = req.schedule_month { sub.schedule_month = Some(month); }
    if let Some(start_date) = req.start_date { sub.start_date = start_date; }
    if req.clear_end_date { sub.end_date = None; } else if let Some(end_date) = req.end_date { sub.end_date = Some(end_date); }
    if let Some(next_billing_date) = req.next_billing_date {
        sub.next_billing_date = Some(next_billing_date);
    } else {
        sub.next_billing_date = crate::utils::recurrence::next_occurrence_on_or_after(
            sub.start_date,
            &sub.recurrence,
            chrono::Utc::now().date_naive(),
            sub.end_date,
        );
    }
    if let Some(account_id) = req.account_id { sub.account_id = account_id; }
    if let Some(category_id) = req.category_id { sub.category_id = category_id; }
    if let Some(is_active) = req.is_active { sub.is_active = is_active; }
    if let Some(notes) = &req.notes {
        sub.notes = if notes.trim().is_empty() { None } else { Some(notes.clone()) };
    }
    if let Some(logo_url) = &req.logo_url { sub.logo_url = Some(logo_url.clone()); }
    sub.updated_at = Utc::now();
    let updated = sub.clone();
    save_collection("subscriptions", list)?;
    Ok(updated)
}

pub async fn delete_subscription(id: &Uuid) -> Result<(), ApiError> {
    let mut cache = SUBSCRIPTIONS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("subscriptions");
    }
    let list = cache.as_mut().ok_or_else(|| ApiError::Network("No subscriptions".into()))?;
    list.retain(|s| &s.id != id);
    record_tombstone("subscription", *id);
    save_collection("subscriptions", list)
}

pub async fn calendar_month(year: i32, month: u32) -> Result<Vec<CalendarItem>, ApiError> {
    let subs = list_subscriptions().await?;
    let mut items = Vec::new();
    for sub in subs.into_iter().filter(|s| s.is_active) {
        for (billing_date, amount) in crate::utils::recurrence::occurrences_in_month(
            sub.start_date,
            &sub.recurrence,
            sub.end_date,
            year,
            month,
            sub.amount,
        ) {
            items.push(CalendarItem {
                subscription_id: sub.id,
                name: sub.name.clone(),
                amount,
                currency: sub.currency.clone(),
                frequency: sub.frequency.clone(),
                billing_date,
                is_active: sub.is_active,
                logo_url: sub.logo_url.clone(),
            });
        }
    }
    items.sort_by_key(|item| item.billing_date);
    Ok(items)
}

// ── Transaction operations ──

pub async fn list_transactions() -> Result<Vec<Transaction>, ApiError> {
    ensure_accounts().await?;
    let mut cache = TRANSACTIONS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("transactions");
    }
    Ok(cache.clone().unwrap_or_default())
}

pub async fn create_transaction(req: &CreateTransactionRequest) -> Result<Transaction, ApiError> {
    ensure_accounts().await?;
    let account_id = if req.account_id.is_nil() {
        default_account_id()?
    } else {
        req.account_id
    };
    let txn = Transaction {
        account_id,
        id: Uuid::new_v4(),
        user_id: Uuid::nil(),
        category_id: req.category_id.unwrap_or(Uuid::nil()),
        name: req.name.clone(),
        amount: req.amount,
        currency: req.currency.clone(),
        date: req.date,
        notes: req.notes.clone(),
        logo_url: req.logo_url.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let mut cache = TRANSACTIONS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("transactions");
    }
    let list = cache.get_or_insert_with(Vec::new);
    list.push(txn.clone());
    save_collection("transactions", list)?;
    Ok(txn)
}

pub async fn update_transaction(id: &Uuid, req: &UpdateTransactionRequest) -> Result<Transaction, ApiError> {
    let mut cache = TRANSACTIONS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("transactions");
    }
    let list = cache.as_mut().ok_or_else(|| ApiError::Network("No transactions".into()))?;
    let txn = list.iter_mut().find(|t| &t.id == id).ok_or_else(|| ApiError::Http(404, "Not found".into()))?;
    if let Some(name) = &req.name { txn.name = name.clone(); }
    if let Some(amount) = req.amount { txn.amount = amount; }
    if let Some(currency) = &req.currency { txn.currency = currency.clone(); }
    if let Some(date) = req.date { txn.date = date; }
    if let Some(account_id) = req.account_id { txn.account_id = account_id; }
    if let Some(category_id) = req.category_id { txn.category_id = category_id; }
    if let Some(notes) = &req.notes {
        txn.notes = if notes.trim().is_empty() { None } else { Some(notes.clone()) };
    }
    if let Some(logo_url) = &req.logo_url { txn.logo_url = Some(logo_url.clone()); }
    txn.updated_at = Utc::now();
    let updated = txn.clone();
    save_collection("transactions", list)?;
    Ok(updated)
}

pub async fn delete_transaction(id: &Uuid) -> Result<(), ApiError> {
    let mut cache = TRANSACTIONS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("transactions");
    }
    let list = cache.as_mut().ok_or_else(|| ApiError::Network("No transactions".into()))?;
    list.retain(|t| &t.id != id);
    record_tombstone("transaction", *id);
    save_collection("transactions", list)
}


// ── Account operations ──

fn default_account_id() -> Result<Uuid, ApiError> {
    let cache = ACCOUNTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    cache
        .as_ref()
        .and_then(|accounts| accounts.iter().find(|a| a.is_default).or_else(|| accounts.first()))
        .map(|a| a.id)
        .ok_or_else(|| ApiError::Network("No accounts".into()))
}

/// Seed accounts from legacy categories when local data predates accounts,
/// or create a single General account for a brand-new user.
async fn ensure_accounts() -> Result<(), ApiError> {
    if ACCOUNTS.lock().map_err(|e| ApiError::Network(e.to_string()))?.is_some() {
        return Ok(());
    }

    let mut accounts = load_collection::<Account>("accounts").unwrap_or_default();
    if accounts.is_empty() {
        let has_data = load_collection::<Subscription>("subscriptions")
            .is_some_and(|v| !v.is_empty())
            || load_collection::<Transaction>("transactions").is_some_and(|v| !v.is_empty());

        if has_data {
            accounts = list_categories()
                .await?
                .into_iter()
                .map(|c| Account {
                    id: c.id,
                    user_id: c.user_id,
                    name: c.name,
                    icon: c.icon,
                    color: c.color,
                    is_default: c.is_default,
                    created_at: c.created_at,
                    updated_at: c.updated_at,
                })
                .collect();
        }

        if accounts.is_empty() {
            accounts.push(Account {
                id: Uuid::new_v4(),
                user_id: Uuid::nil(),
                name: "General".into(),
                icon: Some("Wallet".into()),
                color: Some("#6b7280".into()),
                is_default: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        save_collection("accounts", &accounts)?;
    }

    if accounts.iter().filter(|a| a.is_default).count() != 1 {
        for (index, account) in accounts.iter_mut().enumerate() {
            account.is_default = index == 0;
        }
        save_collection("accounts", &accounts)?;
    }

    let default_id = accounts
        .iter()
        .find(|a| a.is_default)
        .or_else(|| accounts.first())
        .map(|a| a.id)
        .unwrap_or(Uuid::nil());
    let known_ids: std::collections::HashSet<Uuid> = accounts.iter().map(|a| a.id).collect();

    if let Some(mut subs) = load_collection::<Subscription>("subscriptions") {
        let mut changed = false;
        for sub in &mut subs {
            if sub.account_id.is_nil() {
                sub.account_id = if known_ids.contains(&sub.category_id) {
                    sub.category_id
                } else {
                    default_id
                };
                changed = true;
            }
        }
        if changed {
            save_collection("subscriptions", &subs)?;
        }
    }

    if let Some(mut txns) = load_collection::<Transaction>("transactions") {
        let mut changed = false;
        for txn in &mut txns {
            if txn.account_id.is_nil() {
                txn.account_id = if known_ids.contains(&txn.category_id) {
                    txn.category_id
                } else {
                    default_id
                };
                changed = true;
            }
        }
        if changed {
            save_collection("transactions", &txns)?;
        }
    }

    *ACCOUNTS.lock().map_err(|e| ApiError::Network(e.to_string()))? = Some(accounts);
    Ok(())
}

pub async fn list_accounts() -> Result<Vec<Account>, ApiError> {
    ensure_accounts().await?;
    let cache = ACCOUNTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    Ok(cache.clone().unwrap_or_default())
}

pub async fn create_account(req: &CreateAccountRequest) -> Result<Account, ApiError> {
    ensure_accounts().await?;
    let mut cache = ACCOUNTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    let list = cache.get_or_insert_with(Vec::new);
    let is_default = list.is_empty();
    let account = Account {
        id: Uuid::new_v4(),
        user_id: Uuid::nil(),
        name: req.name.clone(),
        icon: req.icon.clone(),
        color: req.color.clone(),
        is_default,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    list.push(account.clone());
    save_collection("accounts", list)?;
    Ok(account)
}

pub async fn update_account(id: &Uuid, req: &UpdateAccountRequest) -> Result<Account, ApiError> {
    ensure_accounts().await?;
    let mut cache = ACCOUNTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    let list = cache.as_mut().ok_or_else(|| ApiError::Network("No accounts".into()))?;
    let account = list
        .iter_mut()
        .find(|a| &a.id == id)
        .ok_or_else(|| ApiError::Http(404, "Not found".into()))?;
    if let Some(name) = &req.name {
        account.name = name.clone();
    }
    if let Some(icon) = &req.icon {
        account.icon = Some(icon.clone());
    }
    if let Some(color) = &req.color {
        account.color = Some(color.clone());
    }
    account.updated_at = Utc::now();
    let updated = account.clone();
    save_collection("accounts", list)?;
    Ok(updated)
}

pub async fn delete_account(id: &Uuid) -> Result<(), ApiError> {
    ensure_accounts().await?;
    let mut cache = ACCOUNTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    let list = cache.as_mut().ok_or_else(|| ApiError::Network("No accounts".into()))?;
    let target = list
        .iter()
        .find(|a| &a.id == id)
        .ok_or_else(|| ApiError::Http(404, "Not found".into()))?;
    if target.is_default {
        return Err(ApiError::Http(400, "Cannot delete the default account".into()));
    }
    let default_id = list
        .iter()
        .find(|a| a.is_default)
        .map(|a| a.id)
        .ok_or_else(|| ApiError::Network("Default account missing".into()))?;

    if let Ok(mut subs) = SUBSCRIPTIONS.lock() {
        if subs.is_none() {
            *subs = load_collection("subscriptions");
        }
        if let Some(items) = subs.as_mut() {
            for sub in items.iter_mut().filter(|s| s.account_id == *id) {
                sub.account_id = default_id;
            }
            save_collection("subscriptions", items)?;
        }
    }
    if let Ok(mut txns) = TRANSACTIONS.lock() {
        if txns.is_none() {
            *txns = load_collection("transactions");
        }
        if let Some(items) = txns.as_mut() {
            for txn in items.iter_mut().filter(|t| t.account_id == *id) {
                txn.account_id = default_id;
            }
            save_collection("transactions", items)?;
        }
    }

    list.retain(|a| &a.id != id);
    record_tombstone("account", *id);
    save_collection("accounts", list)
}

pub async fn set_default_account(id: &Uuid) -> Result<(), ApiError> {
    ensure_accounts().await?;
    let mut cache = ACCOUNTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    let list = cache.as_mut().ok_or_else(|| ApiError::Network("No accounts".into()))?;
    if !list.iter().any(|a| &a.id == id) {
        return Err(ApiError::Http(404, "Not found".into()));
    }
    for account in list.iter_mut() {
        account.is_default = &account.id == id;
        account.updated_at = Utc::now();
    }
    save_collection("accounts", list)
}

// ── Debt operations ──

async fn default_category_id() -> Result<Uuid, ApiError> {
    list_categories()
        .await?
        .into_iter()
        .find(|category| category.is_default)
        .or_else(list_categories_blocking_fallback)
        .map(|category| category.id)
        .ok_or_else(|| ApiError::Network("Default category missing".into()))
}

fn list_categories_blocking_fallback() -> Option<Category> {
    CATEGORIES
        .lock()
        .ok()
        .and_then(|cache| cache.as_ref().and_then(|list| list.first().cloned()))
}

fn normalize_payment_days(days: &mut Vec<u8>, start: NaiveDate) {
    if days.is_empty() {
        days.push(start.day() as u8);
    }
    days.sort_unstable();
    days.dedup();
}

fn monthly_recurrence(days: &[u8]) -> Recurrence {
    Recurrence {
        interval: 1,
        unit: "month".into(),
        days: days.to_vec(),
        ..Default::default()
    }
}

pub async fn list_debts() -> Result<Vec<Debt>, ApiError> {
    let mut cache = DEBTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("debts");
    }
    let mut debts = cache.clone().unwrap_or_default();
    debts.sort_by_key(|debt| (!debt.is_active, debt.created_at));
    Ok(debts)
}

pub async fn create_debt(req: &CreateDebtRequest) -> Result<Debt, ApiError> {
    ensure_accounts().await?;
    let account_id = if req.account_id.is_nil() {
        default_account_id()?
    } else {
        req.account_id
    };
    let category_id = match req.category_id {
        Some(id) => id,
        None => default_category_id().await?,
    };
    let debt_type = req.debt_type.trim().to_lowercase();
    if debt_type != crate::utils::constants::DEBT_TYPE_MANUAL
        && debt_type != crate::utils::constants::DEBT_TYPE_AUTOMATIC
    {
        return Err(ApiError::Http(400, "Debt type must be manual or automatic".into()));
    }
    if req.total_amount <= Decimal::ZERO {
        return Err(ApiError::Http(400, "Debt amount must be greater than zero".into()));
    }
    if req.name.trim().is_empty() {
        return Err(ApiError::Http(400, "Debt name is required".into()));
    }
    let start_date = req.start_date.unwrap_or_else(|| Utc::now().date_naive());
    let mut payment_days = req.payment_days.clone();
    let (monthly_payment, next_payment_date, subscription_id) =
        if debt_type == crate::utils::constants::DEBT_TYPE_AUTOMATIC {
            normalize_payment_days(&mut payment_days, start_date);
            if payment_days.iter().any(|day| *day > 31) {
                return Err(ApiError::Http(400, "Payment days must be 0-31".into()));
            }
            let monthly = req.monthly_payment.ok_or_else(|| {
                ApiError::Http(400, "Automatic debts need a monthly payment".into())
            })?;
            if monthly <= Decimal::ZERO {
                return Err(ApiError::Http(400, "Monthly payment must be greater than zero".into()));
            }
            let recurrence = monthly_recurrence(&payment_days);
            let subscription = create_subscription(&crate::models::subscription::CreateSubscriptionRequest {
                name: req.name.clone(),
                amount: monthly,
                currency: req.currency.clone(),
                frequency: "monthly".into(),
                schedule_day: payment_days.first().map(|day| i16::from(*day)),
                schedule_month: None,
                recurrence: Some(recurrence.clone()),
                is_active: Some(true),
                start_date,
                end_date: None,
                account_id,
                category_id: Some(category_id),
                notes: Some("Automatic debt payment".to_string()),
                logo_url: req.logo_url.clone(),
            })
            .await?;
            let next = crate::utils::recurrence::next_occurrence_on_or_after(
                start_date,
                &recurrence,
                Utc::now().date_naive(),
                None,
            );
            (Some(monthly), next, Some(subscription.id))
        } else {
            (None, None, None)
        };

    let debt = Debt {
        id: Uuid::new_v4(),
        user_id: Uuid::nil(),
        account_id,
        category_id,
        name: req.name.trim().to_string(),
        currency: req.currency.to_uppercase(),
        total_amount: req.total_amount,
        remaining_amount: req.total_amount,
        paid_amount: Decimal::ZERO,
        debt_type,
        monthly_payment,
        payment_days,
        start_date,
        next_payment_date,
        subscription_id,
        is_active: true,
        notes: req.notes.clone(),
        logo_url: req.logo_url.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let mut cache = DEBTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    let list = cache.get_or_insert_with(Vec::new);
    list.push(debt.clone());
    save_collection("debts", list)?;
    Ok(debt)
}

pub async fn update_debt(id: &Uuid, req: &UpdateDebtRequest) -> Result<Debt, ApiError> {
    ensure_accounts().await?;
    let existing = {
        let mut cache = DEBTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
        if cache.is_none() {
            *cache = load_collection("debts");
        }
        cache
            .as_ref()
            .and_then(|list| list.iter().find(|debt| &debt.id == id).cloned())
            .ok_or_else(|| ApiError::Http(404, "Debt not found".into()))?
    };
    let debt_type = req
        .debt_type
        .clone()
        .unwrap_or_else(|| existing.debt_type.clone())
        .trim()
        .to_lowercase();
    if debt_type != crate::utils::constants::DEBT_TYPE_MANUAL
        && debt_type != crate::utils::constants::DEBT_TYPE_AUTOMATIC
    {
        return Err(ApiError::Http(400, "Debt type must be manual or automatic".into()));
    }
    let total_amount = req.total_amount.unwrap_or(existing.total_amount);
    if total_amount <= Decimal::ZERO {
        return Err(ApiError::Http(400, "Debt amount must be greater than zero".into()));
    }
    let paid = existing.total_amount - existing.remaining_amount;
    let remaining_amount = if total_amount == existing.total_amount {
        existing.remaining_amount
    } else {
        (total_amount - paid).max(Decimal::ZERO)
    };
    let start_date = req.start_date.unwrap_or(existing.start_date);
    let mut payment_days = req.payment_days.clone().unwrap_or_else(|| existing.payment_days.clone());
    let mut subscription_id = existing.subscription_id;
    let mut monthly_payment = None;
    let mut next_payment_date = None;
    if debt_type == crate::utils::constants::DEBT_TYPE_AUTOMATIC {
        normalize_payment_days(&mut payment_days, start_date);
        if payment_days.iter().any(|day| *day > 31) {
            return Err(ApiError::Http(400, "Payment days must be 0-31".into()));
        }
        let monthly = req
            .monthly_payment
            .or(existing.monthly_payment)
            .ok_or_else(|| ApiError::Http(400, "Automatic debts need a monthly payment".into()))?;
        monthly_payment = Some(monthly);
        let recurrence = monthly_recurrence(&payment_days);
        let account_id = req.account_id.unwrap_or(existing.account_id);
        let category_id = req.category_id.unwrap_or(existing.category_id);
        if let Some(sub_id) = subscription_id {
            update_subscription(
                &sub_id,
                &crate::models::subscription::UpdateSubscriptionRequest {
                    name: req.name.clone(),
                    amount: Some(monthly),
                    category_id: Some(category_id),
                    account_id: Some(account_id),
                    recurrence: Some(recurrence.clone()),
                    start_date: Some(start_date),
                    ..Default::default()
                },
            )
            .await?;
        } else {
            let subscription = create_subscription(&crate::models::subscription::CreateSubscriptionRequest {
                name: req.name.clone().unwrap_or_else(|| existing.name.clone()),
                amount: monthly,
                currency: req.currency.clone().unwrap_or_else(|| existing.currency.clone()),
                frequency: "monthly".into(),
                schedule_day: payment_days.first().map(|day| i16::from(*day)),
                schedule_month: None,
                recurrence: Some(recurrence.clone()),
                is_active: Some(true),
                start_date,
                end_date: None,
                account_id,
                category_id: Some(category_id),
                notes: Some("Automatic debt payment".to_string()),
                logo_url: req.logo_url.clone().or_else(|| existing.logo_url.clone()),
            })
            .await?;
            subscription_id = Some(subscription.id);
        }
        next_payment_date = crate::utils::recurrence::next_occurrence_on_or_after(
            start_date,
            &recurrence,
            Utc::now().date_naive(),
            None,
        );
    } else {
        payment_days.clear();
        if let Some(sub_id) = subscription_id {
            let _ = delete_subscription(&sub_id).await;
        }
        subscription_id = None;
    }
    let notes = if req.clear_notes {
        None
    } else {
        match &req.notes {
            Some(value) if value.trim().is_empty() => None,
            Some(value) => Some(value.clone()),
            None => existing.notes.clone(),
        }
    };
    let updated = Debt {
        account_id: req.account_id.unwrap_or(existing.account_id),
        category_id: req.category_id.unwrap_or(existing.category_id),
        name: req.name.clone().unwrap_or(existing.name),
        currency: req.currency.clone().unwrap_or(existing.currency).to_uppercase(),
        total_amount,
        remaining_amount,
        paid_amount: total_amount - remaining_amount,
        debt_type,
        monthly_payment,
        payment_days,
        start_date,
        next_payment_date,
        subscription_id,
        is_active: remaining_amount > Decimal::ZERO,
        notes,
        logo_url: req.logo_url.clone().or(existing.logo_url),
        updated_at: Utc::now(),
        ..existing
    };
    let mut cache = DEBTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    let list = cache.as_mut().ok_or_else(|| ApiError::Network("No debts".into()))?;
    let item = list.iter_mut().find(|debt| &debt.id == id).ok_or_else(|| ApiError::Http(404, "Debt not found".into()))?;
    *item = updated.clone();
    save_collection("debts", list)?;
    Ok(updated)
}

pub async fn delete_debt(id: &Uuid) -> Result<(), ApiError> {
    let subscription_id = {
        let mut cache = DEBTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
        if cache.is_none() {
            *cache = load_collection("debts");
        }
        let list = cache.as_mut().ok_or_else(|| ApiError::Network("No debts".into()))?;
        let subscription_id = list
            .iter()
            .find(|debt| &debt.id == id)
            .and_then(|debt| debt.subscription_id);
        list.retain(|debt| &debt.id != id);
        save_collection("debts", list)?;
        subscription_id
    };
    if let Some(sub_id) = subscription_id {
        let _ = delete_subscription(&sub_id).await;
    }
    record_tombstone("debt", *id);
    let mut payments = DEBT_PAYMENTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if payments.is_none() {
        *payments = load_collection("debt_payments");
    }
    if let Some(list) = payments.as_mut() {
        list.retain(|payment| &payment.debt_id != id);
        save_collection("debt_payments", list)?;
    }
    Ok(())
}

pub async fn record_debt_payment(id: &Uuid, req: &RecordDebtPaymentRequest) -> Result<Debt, ApiError> {
    if req.amount <= Decimal::ZERO {
        return Err(ApiError::Http(400, "Payment must be greater than zero".into()));
    }
    let existing = {
        let mut cache = DEBTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
        if cache.is_none() {
            *cache = load_collection("debts");
        }
        let list = cache.as_mut().ok_or_else(|| ApiError::Network("No debts".into()))?;
        let debt = list.iter_mut().find(|debt| &debt.id == id).ok_or_else(|| ApiError::Http(404, "Debt not found".into()))?;
        if debt.remaining_amount <= Decimal::ZERO {
            return Err(ApiError::Http(400, "Debt is already paid".into()));
        }
        if req.amount > debt.remaining_amount {
            return Err(ApiError::Http(400, "Payment exceeds the remaining debt".into()));
        }
        debt.remaining_amount -= req.amount;
        debt.paid_amount += req.amount;
        debt.is_active = debt.remaining_amount > Decimal::ZERO;
        debt.updated_at = Utc::now();
        let clone = debt.clone();
        save_collection("debts", list)?;
        clone
    };
    {
        let mut payments = DEBT_PAYMENTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
        let list = payments.get_or_insert_with(Vec::new);
        list.push(DebtPayment {
            id: Uuid::new_v4(),
            debt_id: *id,
            user_id: Uuid::nil(),
            amount: req.amount,
            paid_on: req.paid_on,
            note: req.note.clone(),
            created_at: Utc::now(),
        });
        save_collection("debt_payments", list)?;
    }
    if !existing.is_active
        && let Some(sub_id) = existing.subscription_id
    {
        let _ = update_subscription(
            &sub_id,
            &crate::models::subscription::UpdateSubscriptionRequest {
                is_active: Some(false),
                end_date: Some(existing.start_date),
                ..Default::default()
            },
        )
        .await;
    }
    Ok(existing)
}

pub async fn list_debt_payments(id: &Uuid) -> Result<Vec<DebtPayment>, ApiError> {
    let mut cache = DEBT_PAYMENTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("debt_payments");
    }
    Ok(cache
        .clone()
        .unwrap_or_default()
        .into_iter()
        .filter(|payment| &payment.debt_id == id)
        .collect())
}

// ── Category operations ──

fn migrate_legacy_categories(categories: &mut Vec<Category>) -> Result<bool, ApiError> {
    let keep = categories
        .iter()
        .position(|category| category.is_system && category.name == "Uncategorized");
    let created = keep.is_none();
    let keep_id = if let Some(index) = keep {
        categories[index].is_system = true;
        categories[index].name = "Uncategorized".into();
        categories[index].icon = None;
        categories[index].color = None;
        categories[index].is_default = true;
        categories[index].id
    } else {
        let category = Category {
            id: Uuid::new_v4(),
            user_id: Uuid::nil(),
            name: "Uncategorized".into(),
            icon: None,
            color: None,
            is_system: true,
            is_default: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let id = category.id;
        categories.push(category);
        id
    };

    let removed: Vec<Uuid> = categories
        .iter()
        .filter(|category| category.is_system && category.id != keep_id)
        .map(|category| category.id)
        .collect();
    if removed.is_empty() {
        for category in categories.iter_mut() {
            category.is_default = category.id == keep_id;
            if category.id == keep_id {
                category.is_system = true;
                category.name = "Uncategorized".into();
                category.icon = None;
                category.color = None;
            }
        }
        return Ok(created);
    }

    if let Some(mut subscriptions) = load_collection::<Subscription>("subscriptions") {
        let mut changed = false;
        for subscription in &mut subscriptions {
            if removed.contains(&subscription.category_id) {
                subscription.category_id = keep_id;
                subscription.updated_at = Utc::now();
                changed = true;
            }
        }
        if changed { save_collection("subscriptions", &subscriptions)?; }
    }
    if let Some(mut transactions) = load_collection::<Transaction>("transactions") {
        let mut changed = false;
        for transaction in &mut transactions {
            if removed.contains(&transaction.category_id) {
                transaction.category_id = keep_id;
                transaction.updated_at = Utc::now();
                changed = true;
            }
        }
        if changed { save_collection("transactions", &transactions)?; }
    }
    if let Some(mut debts) = load_collection::<Debt>("debts") {
        let mut changed = false;
        for debt in &mut debts {
            if removed.contains(&debt.category_id) {
                debt.category_id = keep_id;
                debt.updated_at = Utc::now();
                changed = true;
            }
        }
        if changed { save_collection("debts", &debts)?; }
    }
    categories.retain(|category| !removed.contains(&category.id));
    for category in categories.iter_mut() {
        category.is_default = category.id == keep_id;
    }
    for id in removed {
        record_tombstone("category", id);
    }
    Ok(true)
}

pub async fn list_categories() -> Result<Vec<Category>, ApiError> {
    let mut cache = CATEGORIES.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("categories");
        // Seed defaults if empty
        if cache.as_ref().is_none_or(|c| c.is_empty()) {
            *cache = Some(default_categories());
            if let Some(ref list) = *cache {
                save_collection("categories", list)?;
            }
        }
    }
    if let Some(list) = cache.as_mut()
        && migrate_legacy_categories(list)?
    {
        save_collection("categories", list)?;
    }
    Ok(cache.clone().unwrap_or_default())
}

pub async fn create_category(req: &CreateCategoryRequest) -> Result<Category, ApiError> {
    let cat = Category {
        id: Uuid::new_v4(),
        user_id: Uuid::nil(),
        name: req.name.clone(),
        icon: req.icon.clone(),
        color: req.color.clone(),
        is_system: false,
        is_default: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let mut cache = CATEGORIES.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("categories");
    }
    let list = cache.get_or_insert_with(Vec::new);
    list.push(cat.clone());
    save_collection("categories", list)?;
    Ok(cat)
}

pub async fn update_category(id: &Uuid, req: &UpdateCategoryRequest) -> Result<Category, ApiError> {
    let mut cache = CATEGORIES.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("categories");
    }
    let list = cache.as_mut().ok_or_else(|| ApiError::Network("No categories".into()))?;
    let cat = list.iter_mut().find(|c| &c.id == id).ok_or_else(|| ApiError::Http(404, "Not found".into()))?;
    if cat.is_system {
        return Err(ApiError::Http(
            400,
            "Uncategorized is a system category and cannot be edited".into(),
        ));
    }
    if let Some(name) = &req.name { cat.name = name.clone(); }
    if let Some(icon) = &req.icon { cat.icon = Some(icon.clone()); }
    if let Some(color) = &req.color { cat.color = Some(color.clone()); }
    cat.updated_at = Utc::now();
    let updated = cat.clone();
    save_collection("categories", list)?;
    Ok(updated)
}

pub async fn delete_category(id: &Uuid) -> Result<(), ApiError> {
    let mut cache = CATEGORIES.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("categories");
    }
    let list = cache.as_mut().ok_or_else(|| ApiError::Network("No categories".into()))?;
    let category = list
        .iter()
        .find(|c| &c.id == id)
        .ok_or_else(|| ApiError::Http(404, "Not found".into()))?;
    if category.is_system || category.is_default {
        return Err(ApiError::Http(
            400,
            "Uncategorized is a system category and cannot be deleted".into(),
        ));
    }
    list.retain(|c| &c.id != id);
    record_tombstone("category", *id);
    save_collection("categories", list)
}

pub async fn set_default_category(id: &Uuid) -> Result<(), ApiError> {
    let mut cache = CATEGORIES.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("categories");
    }
    let list = cache.as_mut().ok_or_else(|| ApiError::Network("No categories".into()))?;
    let Some(category) = list.iter().find(|c| &c.id == id) else {
        return Err(ApiError::Http(404, "Not found".into()));
    };
    if !category.is_system {
        return Err(ApiError::Http(
            400,
            "Uncategorized is the fixed default category".into(),
        ));
    }
    for cat in list.iter_mut() {
        cat.is_default = cat.is_system;
    }
    save_collection("categories", list)
}

// ── Reports ──

pub async fn get_report_summary(year: i32, month: u32) -> Result<crate::models::report::ReportResponse, ApiError> {
    use crate::models::report::{MonthlySummary, CategoryBreakdown, ReportResponse};
    use chrono::Datelike;
    use rust_decimal::Decimal;

    let subs = list_subscriptions().await.unwrap_or_default();
    let txns = list_transactions().await.unwrap_or_default();
    let cats = list_categories().await.unwrap_or_default();

    // Filter subscriptions active in the given month
    let month_subs: Vec<_> = subs.iter()
        .filter(|s| {
            let start = s.start_date;
            // Subscription started before or during this month
            (start.year() < year || (start.year() == year && start.month() <= month))
            && s.is_active
        })
        .collect();

    let subs_total: Decimal = month_subs.iter().map(|s| s.amount).sum();

    // Filter transactions in the given month
    let month_txns: Vec<_> = txns.iter()
        .filter(|t| t.date.year() == year && t.date.month() == month)
        .collect();

    let txn_total: Decimal = month_txns.iter().map(|t| t.amount).sum();

    // Category breakdown (from subscriptions)
    let mut cat_map: std::collections::HashMap<Uuid, (String, Decimal, i64)> = std::collections::HashMap::new();
    for s in &month_subs {
        let entry = cat_map.entry(s.category_id).or_insert((String::new(), Decimal::ZERO, 0));
        if let Some(cat) = cats.iter().find(|c| c.id == s.category_id) {
            entry.0 = cat.name.clone();
        }
        entry.1 += s.amount;
        entry.2 += 1;
    }
    let overall = subs_total + txn_total;
    let category_breakdown: Vec<CategoryBreakdown> = cat_map.into_iter().map(|(_, (name, total, count))| {
        let pct = if overall > Decimal::ZERO { (total / overall * Decimal::from(100)).to_string().parse::<f64>().unwrap_or(0.0) } else { 0.0 };
        CategoryBreakdown {
            category_id: None,
            category_name: name,
            total,
            count,
            percentage: pct,
        }
    }).collect();

    Ok(ReportResponse {
        monthly_summary: MonthlySummary {
            year,
            month,
            subscriptions_total: subs_total,
            transactions_total: txn_total,
            overall_total: overall,
            subscriptions_count: month_subs.len() as i64,
            transactions_count: month_txns.len() as i64,
        },
        category_breakdown,
        recent_trends: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[derive(Clone)]
    struct MergeItem {
        id: Uuid,
        updated_at: chrono::DateTime<Utc>,
        value: &'static str,
    }

    #[test]
    fn merge_by_id_keeps_the_newer_record() {
        let id = Uuid::new_v4();
        let old = MergeItem {
            id,
            updated_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            value: "old",
        };
        let new = MergeItem {
            id,
            updated_at: Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap(),
            value: "new",
        };
        let merged = merge_by_id(vec![old], vec![new], |item| item.id, |item| item.updated_at);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].value, "new");
    }

    #[test]
    fn sync_document_round_trips() {
        let doc = SyncDocument {
            format_version: 1,
            device_id: "device".to_string(),
            tombstones: vec![Tombstone {
                kind: "subscription".to_string(),
                id: Uuid::new_v4(),
                deleted_at: Utc::now(),
            }],
            ..Default::default()
        };
        let json = serde_json::to_string(&doc).unwrap();
        let back: SyncDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(back.device_id, "device");
        assert_eq!(back.tombstones.len(), 1);
    }



    #[test]
    fn default_categories_count() {
        let cats = default_categories();
        assert_eq!(cats.len(), 1);
        assert!(cats.iter().all(|c| c.is_system));
        assert!(cats.iter().all(|c| c.is_default));
    }

    #[test]
    fn default_categories_have_unique_ids() {
        let cats = default_categories();
        let ids: std::collections::HashSet<uuid::Uuid> = cats.iter().map(|c| c.id).collect();
        assert_eq!(ids.len(), cats.len(), "All category IDs should be unique");
    }

    #[test]
    fn default_categories_names() {
        let cats = default_categories();
        assert_eq!(cats.len(), 1);
        assert_eq!(cats[0].name, DEFAULT_CATEGORY_NAME);
        assert_eq!(cats[0].name, "Uncategorized");
    }

    #[test]
    fn system_category_is_the_only_default() {
        let mut cats = default_categories();
        cats.push(Category {
            id: Uuid::new_v4(),
            user_id: Uuid::nil(),
            name: "Custom".into(),
            icon: None,
            color: None,
            is_system: false,
            is_default: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
        normalize_category_defaults(&mut cats);
        assert!(cats[0].is_default);
        assert!(!cats[1].is_default);
    }

    #[test]
    fn local_calendar_uses_recurrence_days() {
        let sub = Subscription {
            account_id: Uuid::nil(),
            id: Uuid::new_v4(),
            user_id: Uuid::nil(),
            category_id: Uuid::nil(),
            name: "Test".into(),
            amount: Decimal::new(10, 2),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            schedule_day: Some(15),
            schedule_month: None,
            recurrence: Recurrence::from_legacy(
                "monthly",
                chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                Some(15),
                None,
            ),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            end_date: None,
            next_billing_date: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let dates = crate::utils::recurrence::occurrences_in_month(
            sub.start_date,
            &sub.recurrence,
            sub.end_date,
            2024,
            6,
            sub.amount,
        );
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0].0.day(), 15);
    }

    #[test]
    fn local_calendar_clamps_short_months() {
        let start = chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let recurrence = Recurrence::from_legacy("monthly", start, Some(31), None);
        let dates = crate::utils::recurrence::occurrences_in_month(
            start,
            &recurrence,
            None,
            2024,
            2,
            Decimal::new(10, 2),
        );
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0].0.day(), 29);
    }

    #[test]
    fn local_storage_module_compiles() {
        assert!(true);
    }
}

// ── Portable backup ─────────────────────────────────────────

pub async fn export_backup() -> Result<serde_json::Value, ApiError> {
    let debt_payments = {
        let mut payments = DEBT_PAYMENTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
        if payments.is_none() { *payments = load_collection("debt_payments"); }
        payments.clone().unwrap_or_default()
    };
    Ok(serde_json::json!({
        "format_version": 1,
        "app_version": env!("CARGO_PKG_VERSION"),
        "exported_at": Utc::now(),
        "accounts": list_accounts().await?,
        "categories": list_categories().await?,
        "subscriptions": list_subscriptions().await?,
        "transactions": list_transactions().await?,
        "debts": list_debts().await?,
        "debt_payments": debt_payments,
    }))
}

pub async fn import_backup(backup: &serde_json::Value) -> Result<serde_json::Value, ApiError> {
    if backup.get("format_version").and_then(|value| value.as_u64()) != Some(1) {
        return Err(ApiError::Http(400, "Unsupported backup format version".into()));
    }
    let mut accounts: Vec<Account> = serde_json::from_value(
        backup.get("accounts").cloned().unwrap_or_else(|| serde_json::json!([])),
    )
    .map_err(|error| ApiError::Parse(error.to_string()))?;
    let mut categories: Vec<Category> = serde_json::from_value(
        backup.get("categories").cloned().unwrap_or_else(|| serde_json::json!([])),
    )
    .map_err(|error| ApiError::Parse(error.to_string()))?;
    let mut subscriptions: Vec<Subscription> = serde_json::from_value(
        backup.get("subscriptions").cloned().unwrap_or_else(|| serde_json::json!([])),
    )
    .map_err(|error| ApiError::Parse(error.to_string()))?;
    let mut transactions: Vec<Transaction> = serde_json::from_value(
        backup.get("transactions").cloned().unwrap_or_else(|| serde_json::json!([])),
    )
    .map_err(|error| ApiError::Parse(error.to_string()))?;
    let mut debts: Vec<Debt> = serde_json::from_value(
        backup.get("debts").cloned().unwrap_or_else(|| serde_json::json!([])),
    )
    .map_err(|error| ApiError::Parse(error.to_string()))?;
    let mut debt_payments: Vec<DebtPayment> = serde_json::from_value(
        backup.get("debt_payments").cloned().unwrap_or_else(|| serde_json::json!([])),
    )
    .map_err(|error| ApiError::Parse(error.to_string()))?;
    if accounts.is_empty() || categories.is_empty() {
        return Err(ApiError::Http(400, "Backup must contain at least one account and category".into()));
    }

    for account in &mut accounts { account.user_id = Uuid::nil(); }
    for category in &mut categories { category.user_id = Uuid::nil(); }
    for subscription in &mut subscriptions { subscription.user_id = Uuid::nil(); }
    for transaction in &mut transactions { transaction.user_id = Uuid::nil(); }
    for debt in &mut debts { debt.user_id = Uuid::nil(); }
    for payment in &mut debt_payments { payment.user_id = Uuid::nil(); }

    normalize_defaults(&mut accounts);
    migrate_legacy_categories(&mut categories)?;
    normalize_category_defaults(&mut categories);
    save_collection("accounts", &accounts)?;
    save_collection("categories", &categories)?;
    save_collection("subscriptions", &subscriptions)?;
    save_collection("transactions", &transactions)?;
    save_collection("debts", &debts)?;
    save_collection("debt_payments", &debt_payments)?;
    save_collection("tombstones", &Vec::<Tombstone>::new())?;
    let counts = serde_json::json!({
        "accounts": accounts.len(),
        "categories": categories.len(),
        "subscriptions": subscriptions.len(),
        "transactions": transactions.len(),
        "debts": debts.len(),
        "debt_payments": debt_payments.len(),
    });
    *ACCOUNTS.lock().map_err(|e| ApiError::Network(e.to_string()))? = Some(accounts);
    *CATEGORIES.lock().map_err(|e| ApiError::Network(e.to_string()))? = Some(categories);
    *SUBSCRIPTIONS.lock().map_err(|e| ApiError::Network(e.to_string()))? = Some(subscriptions);
    *TRANSACTIONS.lock().map_err(|e| ApiError::Network(e.to_string()))? = Some(transactions);
    *DEBTS.lock().map_err(|e| ApiError::Network(e.to_string()))? = Some(debts);
    *DEBT_PAYMENTS.lock().map_err(|e| ApiError::Network(e.to_string()))? = Some(debt_payments);

    Ok(counts)
}

// ── Cloud-folder sync document ───────────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Tombstone {
    pub kind: String,
    pub id: Uuid,
    pub deleted_at: chrono::DateTime<Utc>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SyncDocument {
    #[serde(default = "sync_format_version")]
    pub format_version: u32,
    #[serde(default)]
    pub device_id: String,
    #[serde(default)]
    pub subscriptions: Vec<Subscription>,
    #[serde(default)]
    pub transactions: Vec<Transaction>,
    #[serde(default)]
    pub accounts: Vec<Account>,
    #[serde(default)]
    pub categories: Vec<Category>,
    #[serde(default)]
    pub debts: Vec<Debt>,
    #[serde(default)]
    pub debt_payments: Vec<DebtPayment>,
    #[serde(default)]
    pub tombstones: Vec<Tombstone>,
}

fn sync_format_version() -> u32 { 1 }

fn load_text(name: &str) -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let dir = data_dir()?;
        std::fs::read_to_string(dir.join(name)).ok()
    }
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let storage = window.local_storage().ok()??;
        storage.get_item(&format!("recurrency_local_{name}")).ok()?
    }
}

fn save_text(name: &str, value: &str) -> Result<(), ApiError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let dir = crate::state::auth::get_app_data_dir()
            .map(|d| d.join("recurrency"))
            .ok_or_else(|| ApiError::Network("No data directory".into()))?;
        std::fs::create_dir_all(&dir).map_err(|e| ApiError::Network(e.to_string()))?;
        std::fs::write(dir.join(name), value).map_err(|e| ApiError::Network(e.to_string()))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window().ok_or_else(|| ApiError::Network("No window".into()))?;
        let storage = window
            .local_storage()
            .map_err(|_| ApiError::Network("No local storage".into()))?
            .ok_or_else(|| ApiError::Network("No local storage".into()))?;
        storage
            .set_item(&format!("recurrency_local_{name}"), value)
            .map_err(|_| ApiError::Network("Could not write local storage".into()))
    }
}

fn sync_device_id() -> Result<String, ApiError> {
    if let Some(value) = load_text("sync_device_id")
        && !value.trim().is_empty()
    {
        return Ok(value.trim().to_string());
    }
    let value = Uuid::new_v4().to_string();
    save_text("sync_device_id", &value)?;
    Ok(value)
}

fn load_tombstones() -> Vec<Tombstone> {
    load_collection("tombstones").unwrap_or_default()
}

fn record_tombstone(kind: &str, id: Uuid) {
    let mut tombstones = load_tombstones();
    tombstones.retain(|item| !(item.kind == kind && item.id == id));
    tombstones.push(Tombstone {
        kind: kind.to_string(),
        id,
        deleted_at: Utc::now(),
    });
    let _ = save_collection("tombstones", &tombstones);
}

fn normalize_category_defaults(categories: &mut [Category]) {
    let system = categories.iter().position(|category| category.is_system);
    for (index, category) in categories.iter_mut().enumerate() {
        category.is_default = system == Some(index);
    }
}

fn normalize_defaults<T>(items: &mut [T])
where
    T: IsDefaultRecord + Clone,
{
    let first_default = items.iter().position(|item| item.is_default());
    let fallback = first_default.or(if items.is_empty() { None } else { Some(0) });
    for (index, item) in items.iter_mut().enumerate() {
        item.set_default(Some(index) == fallback);
    }
}

trait IsDefaultRecord {
    fn is_default(&self) -> bool;
    fn set_default(&mut self, value: bool);
}

impl IsDefaultRecord for Account {
    fn is_default(&self) -> bool { self.is_default }
    fn set_default(&mut self, value: bool) { self.is_default = value; }
}

impl IsDefaultRecord for Category {
    fn is_default(&self) -> bool { self.is_default }
    fn set_default(&mut self, value: bool) { self.is_default = value; }
}

pub async fn sync_snapshot() -> Result<String, ApiError> {
    let document = SyncDocument {
        format_version: 1,
        device_id: sync_device_id()?,
        subscriptions: list_subscriptions().await?,
        transactions: list_transactions().await?,
        accounts: list_accounts().await?,
        categories: list_categories().await?,
        debts: list_debts().await?,
        debt_payments: {
            let mut payments = DEBT_PAYMENTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
            if payments.is_none() { *payments = load_collection("debt_payments"); }
            payments.clone().unwrap_or_default()
        },
        tombstones: load_tombstones(),
    };
    serde_json::to_string(&document).map_err(|e| ApiError::Parse(e.to_string()))
}

fn merge_by_id<T>(
    local: Vec<T>,
    remote: Vec<T>,
    id: impl Fn(&T) -> Uuid,
    updated: impl Fn(&T) -> chrono::DateTime<Utc>,
) -> Vec<T>
where
    T: Clone,
{
    let mut merged: std::collections::HashMap<Uuid, T> =
        local.into_iter().map(|item| (id(&item), item)).collect();
    for item in remote {
        let key = id(&item);
        match merged.get(&key) {
            Some(current) if updated(current) > updated(&item) => {}
            _ => {
                merged.insert(key, item);
            }
        }
    }
    merged.into_values().collect()
}

pub async fn sync_merge(remote_files: &[String]) -> Result<String, ApiError> {
    let mut remote_docs = Vec::new();
    for raw in remote_files {
        if let Ok(doc) = serde_json::from_str::<SyncDocument>(raw) {
            remote_docs.push(doc);
        }
    }

    let mut subscriptions = list_subscriptions().await?;
    let mut transactions = list_transactions().await?;
    let mut accounts = list_accounts().await?;
    let mut categories = list_categories().await?;
    let mut debts = list_debts().await?;
    let mut debt_payments = {
        let mut payments = DEBT_PAYMENTS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
        if payments.is_none() { *payments = load_collection("debt_payments"); }
        payments.clone().unwrap_or_default()
    };
    let mut tombstones = load_tombstones();

    for doc in &remote_docs {
        subscriptions = merge_by_id(subscriptions, doc.subscriptions.clone(), |i| i.id, |i| i.updated_at);
        transactions = merge_by_id(transactions, doc.transactions.clone(), |i| i.id, |i| i.updated_at);
        accounts = merge_by_id(accounts, doc.accounts.clone(), |i| i.id, |i| i.updated_at);
        categories = merge_by_id(categories, doc.categories.clone(), |i| i.id, |i| i.updated_at);
        debts = merge_by_id(debts, doc.debts.clone(), |i| i.id, |i| i.updated_at);
        debt_payments = merge_by_id(debt_payments, doc.debt_payments.clone(), |i| i.id, |i| i.created_at);
        for tombstone in &doc.tombstones {
            if let Some(existing) = tombstones.iter_mut().find(|item| item.kind == tombstone.kind && item.id == tombstone.id) {
                if tombstone.deleted_at > existing.deleted_at {
                    existing.deleted_at = tombstone.deleted_at;
                }
            } else {
                tombstones.push(tombstone.clone());
            }
        }
    }

    let is_deleted = |kind: &str, id: Uuid, updated: chrono::DateTime<Utc>| {
        tombstones.iter().any(|t| t.kind == kind && t.id == id && t.deleted_at >= updated)
    };
    subscriptions.retain(|i| !is_deleted("subscription", i.id, i.updated_at));
    transactions.retain(|i| !is_deleted("transaction", i.id, i.updated_at));
    accounts.retain(|i| !is_deleted("account", i.id, i.updated_at));
    categories.retain(|i| !is_deleted("category", i.id, i.updated_at));
    debts.retain(|i| !is_deleted("debt", i.id, i.updated_at));

    normalize_defaults(&mut accounts);
    normalize_category_defaults(&mut categories);
    save_collection("subscriptions", &subscriptions)?;
    save_collection("transactions", &transactions)?;
    save_collection("accounts", &accounts)?;
    save_collection("categories", &categories)?;
    save_collection("debts", &debts)?;
    save_collection("debt_payments", &debt_payments)?;
    save_collection("tombstones", &tombstones)?;
    *SUBSCRIPTIONS.lock().map_err(|e| ApiError::Network(e.to_string()))? = Some(subscriptions.clone());
    *TRANSACTIONS.lock().map_err(|e| ApiError::Network(e.to_string()))? = Some(transactions.clone());
    *ACCOUNTS.lock().map_err(|e| ApiError::Network(e.to_string()))? = Some(accounts.clone());
    *CATEGORIES.lock().map_err(|e| ApiError::Network(e.to_string()))? = Some(categories.clone());
    *DEBTS.lock().map_err(|e| ApiError::Network(e.to_string()))? = Some(debts.clone());
    *DEBT_PAYMENTS.lock().map_err(|e| ApiError::Network(e.to_string()))? = Some(debt_payments.clone());

    let document = SyncDocument {
        format_version: 1,
        device_id: sync_device_id()?,
        subscriptions,
        transactions,
        accounts,
        categories,
        debts,
        debt_payments,
        tombstones,
    };
    serde_json::to_string(&document).map_err(|e| ApiError::Parse(e.to_string()))
}
