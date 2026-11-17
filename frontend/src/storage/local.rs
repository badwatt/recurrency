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
    #[cfg(test)]
    if let Some(dir) = TEST_DATA_DIR.lock().ok().and_then(|dir| dir.clone()) {
        // Only the thread running the storage test may use its temporary
        // directory. Page effects run on other threads and must not observe
        // or remove test files.
        let owner = TEST_DATA_OWNER
            .lock()
            .ok()
            .and_then(|owner| *owner);
        if owner == Some(std::thread::current().id()) {
            return Some(dir);
        }
        return None;
    }
    crate::state::auth::get_app_data_dir().map(|d| d.join("recurrency"))
}

#[cfg(test)]
static TEST_DATA_DIR: std::sync::Mutex<Option<std::path::PathBuf>> = std::sync::Mutex::new(None);

#[cfg(test)]
static TEST_DATA_OWNER: std::sync::Mutex<Option<std::thread::ThreadId>> =
    std::sync::Mutex::new(None);

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
    use rust_decimal_macros::dec;
    use std::future::Future;
    use std::path::PathBuf;

    struct LocalTest {
        _lock: std::sync::MutexGuard<'static, ()>,
        dir: PathBuf,
    }

    impl LocalTest {
        fn new() -> Self {
            let lock = crate::storage::STORAGE_TEST_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let dir = std::env::temp_dir().join(format!(
                "recurrency-local-test-{}",
                Uuid::new_v4()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            *TEST_DATA_DIR
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(dir.clone());
            *TEST_DATA_OWNER
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                Some(std::thread::current().id());
            reset_caches();
            Self { _lock: lock, dir }
        }
    }

    impl Drop for LocalTest {
        fn drop(&mut self) {
            reset_caches();
            *TEST_DATA_DIR
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
            *TEST_DATA_OWNER
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    fn reset_caches() {
        *SUBSCRIPTIONS.lock().unwrap_or_else(|p| p.into_inner()) = None;
        *TRANSACTIONS.lock().unwrap_or_else(|p| p.into_inner()) = None;
        *CATEGORIES.lock().unwrap_or_else(|p| p.into_inner()) = None;
        *ACCOUNTS.lock().unwrap_or_else(|p| p.into_inner()) = None;
        *DEBTS.lock().unwrap_or_else(|p| p.into_inner()) = None;
        *DEBT_PAYMENTS.lock().unwrap_or_else(|p| p.into_inner()) = None;
    }

    fn run<F: Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime")
            .block_on(future)
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn today() -> NaiveDate {
        Utc::now().date_naive()
    }

    fn subscription_request(start: NaiveDate) -> CreateSubscriptionRequest {
        CreateSubscriptionRequest {
            name: "Streaming".into(),
            amount: dec!(9.99),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            schedule_day: Some(start.day() as i16),
            schedule_month: None,
            recurrence: None,
            is_active: Some(true),
            start_date: start,
            end_date: None,
            account_id: Uuid::nil(),
            category_id: None,
            notes: Some("first".into()),
            logo_url: None,
        }
    }

    fn transaction_request(day: NaiveDate) -> CreateTransactionRequest {
        CreateTransactionRequest {
            name: "Coffee".into(),
            amount: dec!(3.50),
            currency: "EUR".into(),
            date: day,
            account_id: Uuid::nil(),
            category_id: None,
            notes: None,
            logo_url: None,
        }
    }

    fn debt_request(debt_type: &str) -> CreateDebtRequest {
        CreateDebtRequest {
            name: "Loan".into(),
            total_amount: dec!(100),
            currency: "eur".into(),
            account_id: Uuid::nil(),
            category_id: None,
            debt_type: debt_type.into(),
            monthly_payment: Some(dec!(10)),
            payment_days: vec![today().day() as u8],
            start_date: Some(date(2024, 1, 1)),
            notes: None,
            logo_url: None,
        }
    }

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

    fn stored_subscription(category_id: Uuid, start: NaiveDate) -> Subscription {
        Subscription {
            account_id: Uuid::nil(),
            id: Uuid::new_v4(),
            user_id: Uuid::nil(),
            category_id,
            name: "Legacy plan".into(),
            amount: dec!(5),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            schedule_day: Some(start.day() as i16),
            schedule_month: None,
            recurrence: Recurrence::default(),
            start_date: start,
            end_date: None,
            next_billing_date: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn stored_transaction(category_id: Uuid) -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            user_id: Uuid::nil(),
            account_id: Uuid::nil(),
            category_id,
            name: "Legacy purchase".into(),
            amount: dec!(4),
            currency: "EUR".into(),
            date: today(),
            notes: None,
            logo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn stored_debt(category_id: Uuid) -> Debt {
        Debt {
            id: Uuid::new_v4(),
            user_id: Uuid::nil(),
            account_id: Uuid::nil(),
            category_id,
            name: "Legacy loan".into(),
            currency: "EUR".into(),
            total_amount: dec!(100),
            remaining_amount: dec!(80),
            paid_amount: dec!(20),
            debt_type: "manual".into(),
            monthly_payment: None,
            payment_days: Vec::new(),
            start_date: date(2024, 1, 1),
            next_payment_date: None,
            subscription_id: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn local_subscription_crud_calendar_and_tombstone() {
        let _test = LocalTest::new();
        run(async {
            assert!(list_subscriptions().await.unwrap().is_empty());

            let start = today();
            let subscription = create_subscription(&subscription_request(start)).await.unwrap();
            assert!(!subscription.account_id.is_nil());
            assert_eq!(subscription.next_billing_date, Some(start));
            assert_eq!(list_subscriptions().await.unwrap().len(), 1);

            let calendar = calendar_month(start.year(), start.month()).await.unwrap();
            assert_eq!(calendar.len(), 1);
            assert_eq!(calendar[0].subscription_id, subscription.id);

            let updated = update_subscription(
                &subscription.id,
                &UpdateSubscriptionRequest {
                    name: Some("Annual plan".into()),
                    amount: Some(dec!(120)),
                    currency: Some("USD".into()),
                    frequency: Some("yearly".into()),
                    schedule_day: Some(10),
                    schedule_month: Some(6),
                    start_date: Some(date(2024, 1, 1)),
                    clear_end_date: true,
                    notes: Some("   ".into()),
                    is_active: Some(true),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            assert_eq!(updated.name, "Annual plan");
            assert_eq!(updated.amount, dec!(120));
            assert_eq!(updated.currency, "USD");
            assert_eq!(updated.recurrence.unit, "year");
            assert_eq!(updated.recurrence.dates[0].day, 10);
            assert!(updated.notes.is_none());
            assert!(updated.is_active);

            let june = calendar_month(2024, 6).await.unwrap();
            assert_eq!(june.len(), 1);
            assert_eq!(june[0].billing_date, date(2024, 6, 10));

            delete_subscription(&subscription.id).await.unwrap();
            assert!(list_subscriptions().await.unwrap().is_empty());
            assert!(load_tombstones().iter().any(|item| item.kind == "subscription" && item.id == subscription.id));

            assert!(update_subscription(&Uuid::new_v4(), &UpdateSubscriptionRequest::default()).await.is_err());
        });
    }

    #[test]
    fn local_legacy_records_are_upgraded_and_receive_accounts() {
        let _test = LocalTest::new();
        run(async {
            let start = date(2024, 1, 15);
            let subscription = stored_subscription(Uuid::nil(), start);
            let transaction = stored_transaction(Uuid::nil());
            save_collection("subscriptions", &vec![subscription.clone()]).unwrap();
            save_collection("transactions", &vec![transaction.clone()]).unwrap();

            let subscriptions = list_subscriptions().await.unwrap();
            assert_eq!(subscriptions.len(), 1);
            assert_eq!(subscriptions[0].recurrence.days, vec![15]);
            assert!(subscriptions[0].next_billing_date.is_some());
            assert!(!subscriptions[0].account_id.is_nil());

            let transactions = list_transactions().await.unwrap();
            assert_eq!(transactions.len(), 1);
            assert!(!transactions[0].account_id.is_nil());
        });
    }

    #[test]
    fn local_missing_collections_return_errors() {
        let _test = LocalTest::new();
        run(async {
            assert!(update_subscription(&Uuid::new_v4(), &UpdateSubscriptionRequest::default()).await.is_err());
            assert!(delete_subscription(&Uuid::new_v4()).await.is_err());
            assert!(update_transaction(&Uuid::new_v4(), &UpdateTransactionRequest::default()).await.is_err());
            assert!(delete_transaction(&Uuid::new_v4()).await.is_err());
            assert!(update_category(&Uuid::new_v4(), &UpdateCategoryRequest { name: None, icon: None, color: None }).await.is_err());
            assert!(delete_category(&Uuid::new_v4()).await.is_err());
            assert!(set_default_category(&Uuid::new_v4()).await.is_err());
            assert!(update_debt(&Uuid::new_v4(), &UpdateDebtRequest::default()).await.is_err());
            assert!(delete_debt(&Uuid::new_v4()).await.is_err());
            assert!(record_debt_payment(&Uuid::new_v4(), &RecordDebtPaymentRequest {
                amount: dec!(1),
                paid_on: today(),
                note: None,
            }).await.is_err());
            assert!(list_debt_payments(&Uuid::new_v4()).await.unwrap().is_empty());
        });
    }

    #[test]
    fn local_transaction_crud_lifecycle() {
        let _test = LocalTest::new();
        run(async {
            assert!(list_transactions().await.unwrap().is_empty());
            let transaction = create_transaction(&transaction_request(today())).await.unwrap();
            assert!(!transaction.account_id.is_nil());

            let updated = update_transaction(
                &transaction.id,
                &UpdateTransactionRequest {
                    name: Some("Lunch".into()),
                    amount: Some(dec!(12.25)),
                    currency: Some("USD".into()),
                    date: Some(date(2024, 3, 4)),
                    account_id: Some(Uuid::new_v4()),
                    category_id: Some(Uuid::new_v4()),
                    notes: Some("  ".into()),
                    logo_url: Some("https://example.test/logo.png".into()),
                },
            )
            .await
            .unwrap();
            assert_eq!(updated.name, "Lunch");
            assert_eq!(updated.amount, dec!(12.25));
            assert_eq!(updated.date, date(2024, 3, 4));
            assert!(updated.notes.is_none());
            assert_eq!(updated.logo_url.as_deref(), Some("https://example.test/logo.png"));

            assert_eq!(list_transactions().await.unwrap().len(), 1);
            delete_transaction(&transaction.id).await.unwrap();
            assert!(list_transactions().await.unwrap().is_empty());
            assert!(update_transaction(&Uuid::new_v4(), &UpdateTransactionRequest::default()).await.is_err());
        });
    }

    #[test]
    fn local_accounts_are_seeded_and_manage_defaults() {
        let _test = LocalTest::new();
        run(async {
            let accounts = list_accounts().await.unwrap();
            assert_eq!(accounts.len(), 1);
            assert!(accounts[0].is_default);
            assert_eq!(accounts[0].name, "General");

            let second = create_account(&CreateAccountRequest {
                name: "Savings".into(),
                icon: Some("PiggyBank".into()),
                color: Some("#123456".into()),
            })
            .await
            .unwrap();
            assert!(!second.is_default);

            let updated = update_account(
                &second.id,
                &UpdateAccountRequest {
                    name: Some("Rainy day".into()),
                    icon: Some("Umbrella".into()),
                    color: Some("#abcdef".into()),
                },
            )
            .await
            .unwrap();
            assert_eq!(updated.name, "Rainy day");
            assert_eq!(updated.icon.as_deref(), Some("Umbrella"));
            assert_eq!(updated.color.as_deref(), Some("#abcdef"));

            set_default_account(&second.id).await.unwrap();
            let accounts = list_accounts().await.unwrap();
            assert!(accounts.iter().find(|account| account.id == second.id).unwrap().is_default);

            let subscription = create_subscription(&CreateSubscriptionRequest {
                account_id: second.id,
                ..subscription_request(today())
            })
            .await
            .unwrap();
            let transaction = create_transaction(&CreateTransactionRequest {
                account_id: second.id,
                ..transaction_request(today())
            })
            .await
            .unwrap();

            assert!(delete_account(&second.id).await.is_err());
            set_default_account(&accounts[0].id).await.unwrap();
            delete_account(&second.id).await.unwrap();
            assert_eq!(list_subscriptions().await.unwrap()[0].account_id, accounts[0].id);
            assert_eq!(list_transactions().await.unwrap()[0].account_id, accounts[0].id);

            assert!(update_account(&Uuid::new_v4(), &UpdateAccountRequest::default()).await.is_err());
            assert!(delete_account(&Uuid::new_v4()).await.is_err());
            assert!(set_default_account(&Uuid::new_v4()).await.is_err());
            assert_eq!(subscription.id, list_subscriptions().await.unwrap()[0].id);
            assert_eq!(transaction.id, list_transactions().await.unwrap()[0].id);
        });
    }

    #[test]
    fn local_accounts_repair_missing_defaults_and_ids() {
        let _test = LocalTest::new();
        run(async {
            let first = Account {
                id: Uuid::new_v4(),
                user_id: Uuid::nil(),
                name: "First".into(),
                icon: None,
                color: None,
                is_default: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            let second = Account {
                id: Uuid::new_v4(),
                ..first.clone()
            };
            save_collection("accounts", &vec![first.clone(), second.clone()]).unwrap();

            let subscription = stored_subscription(second.id, today());
            let transaction = stored_transaction(Uuid::new_v4());
            save_collection("subscriptions", &vec![subscription]).unwrap();
            save_collection("transactions", &vec![transaction]).unwrap();

            let accounts = list_accounts().await.unwrap();
            assert!(accounts[0].is_default);
            assert!(!accounts[1].is_default);
            assert_eq!(list_subscriptions().await.unwrap()[0].account_id, second.id);
            assert_eq!(list_transactions().await.unwrap()[0].account_id, first.id);
        });
    }

    #[test]
    fn local_categories_manage_system_default_and_crud() {
        let _test = LocalTest::new();
        run(async {
            let categories = list_categories().await.unwrap();
            assert_eq!(categories.len(), 1);
            assert!(categories[0].is_system);
            assert!(categories[0].is_default);
            let system_id = categories[0].id;

            let custom = create_category(&CreateCategoryRequest {
                name: "Food".into(),
                icon: Some("Utensils".into()),
                color: Some("#f97316".into()),
            })
            .await
            .unwrap();
            assert!(!custom.is_system);
            assert!(!custom.is_default);

            let updated = update_category(
                &custom.id,
                &UpdateCategoryRequest {
                    name: Some("Groceries".into()),
                    icon: Some("ShoppingCart".into()),
                    color: None,
                },
            )
            .await
            .unwrap();
            assert_eq!(updated.name, "Groceries");
            assert_eq!(updated.icon.as_deref(), Some("ShoppingCart"));
            assert_eq!(updated.color.as_deref(), Some("#f97316"));

            assert!(update_category(&system_id, &UpdateCategoryRequest { name: Some("Nope".into()), icon: None, color: None }).await.is_err());
            assert!(delete_category(&system_id).await.is_err());
            assert!(set_default_category(&custom.id).await.is_err());
            set_default_category(&system_id).await.unwrap();
            assert!(list_categories().await.unwrap().iter().find(|category| category.id == custom.id).unwrap().is_default == false);

            delete_category(&custom.id).await.unwrap();
            assert_eq!(list_categories().await.unwrap().len(), 1);
            assert!(update_category(&Uuid::new_v4(), &UpdateCategoryRequest { name: None, icon: None, color: None }).await.is_err());
            assert!(delete_category(&Uuid::new_v4()).await.is_err());
            assert!(set_default_category(&Uuid::new_v4()).await.is_err());
        });
    }

    #[test]
    fn local_legacy_categories_are_merged_into_the_system_default() {
        let _test = LocalTest::new();
        run(async {
            let old_system = Category {
                id: Uuid::new_v4(),
                user_id: Uuid::nil(),
                name: "Old system".into(),
                icon: Some("Star".into()),
                color: Some("#000000".into()),
                is_system: true,
                is_default: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            let custom = Category {
                id: Uuid::new_v4(),
                user_id: Uuid::nil(),
                name: "Custom".into(),
                icon: None,
                color: None,
                is_system: false,
                is_default: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            save_collection("categories", &vec![old_system.clone(), custom.clone()]).unwrap();
            save_collection("subscriptions", &vec![stored_subscription(old_system.id, today())]).unwrap();
            save_collection("transactions", &vec![stored_transaction(old_system.id)]).unwrap();
            save_collection("debts", &vec![stored_debt(old_system.id)]).unwrap();

            let categories = list_categories().await.unwrap();
            assert_eq!(categories.len(), 2);
            let new_system = categories.iter().find(|category| category.is_system).unwrap();
            assert_eq!(new_system.name, "Uncategorized");
            assert!(new_system.is_default);
            assert!(!categories.iter().find(|category| category.id == custom.id).unwrap().is_default);
            assert_eq!(list_subscriptions().await.unwrap()[0].category_id, new_system.id);
            assert_eq!(list_transactions().await.unwrap()[0].category_id, new_system.id);
            assert_eq!(list_debts().await.unwrap()[0].category_id, new_system.id);
            assert!(load_tombstones().iter().any(|item| item.kind == "category" && item.id == old_system.id));
        });
    }
    #[test]
    fn local_manual_debt_validates_and_tracks_payments() {
        let _test = LocalTest::new();
        run(async {
            assert!(create_debt(&debt_request("invalid")).await.is_err());
            let mut invalid = debt_request("manual");
            invalid.total_amount = Decimal::ZERO;
            assert!(create_debt(&invalid).await.is_err());
            let mut unnamed = debt_request("manual");
            unnamed.name = "  ".into();
            assert!(create_debt(&unnamed).await.is_err());

            let debt = create_debt(&debt_request("manual")).await.unwrap();
            assert!(!debt.account_id.is_nil());
            assert!(!debt.category_id.is_nil());
            assert_eq!(debt.currency, "EUR");
            assert_eq!(debt.paid_amount, Decimal::ZERO);

            assert!(update_debt(&debt.id, &UpdateDebtRequest { debt_type: Some("bad".into()), ..Default::default() }).await.is_err());
            assert!(update_debt(&debt.id, &UpdateDebtRequest { total_amount: Some(Decimal::ZERO), ..Default::default() }).await.is_err());

            let updated = update_debt(
                &debt.id,
                &UpdateDebtRequest {
                    name: Some("Renamed loan".into()),
                    total_amount: Some(dec!(200)),
                    currency: Some("usd".into()),
                    notes: Some("tracked".into()),
                    debt_type: Some("manual".into()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            assert_eq!(updated.name, "Renamed loan");
            assert_eq!(updated.total_amount, dec!(200));
            assert_eq!(updated.currency, "USD");
            assert_eq!(updated.notes.as_deref(), Some("tracked"));

            assert!(record_debt_payment(&debt.id, &RecordDebtPaymentRequest {
                amount: Decimal::ZERO,
                paid_on: today(),
                note: None,
            }).await.is_err());
            assert!(record_debt_payment(&debt.id, &RecordDebtPaymentRequest {
                amount: dec!(201),
                paid_on: today(),
                note: None,
            }).await.is_err());

            let paid = record_debt_payment(&debt.id, &RecordDebtPaymentRequest {
                amount: dec!(50),
                paid_on: today(),
                note: Some("first".into()),
            }).await.unwrap();
            assert_eq!(paid.remaining_amount, dec!(150));
            assert_eq!(list_debt_payments(&debt.id).await.unwrap().len(), 1);

            let reduced = update_debt(
                &debt.id,
                &UpdateDebtRequest {
                    total_amount: Some(dec!(50)),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            assert_eq!(reduced.remaining_amount, Decimal::ZERO);
            assert!(!reduced.is_active);
            assert!(record_debt_payment(&debt.id, &RecordDebtPaymentRequest {
                amount: dec!(1),
                paid_on: today(),
                note: None,
            }).await.is_err());

            delete_debt(&debt.id).await.unwrap();
            assert!(list_debt_payments(&debt.id).await.unwrap().is_empty());
        });
    }

    #[test]
    fn local_automatic_debt_creates_updates_and_closes_its_subscription() {
        let _test = LocalTest::new();
        run(async {
            let mut invalid_day = debt_request("automatic");
            invalid_day.payment_days = vec![32];
            assert!(create_debt(&invalid_day).await.is_err());

            let mut missing_monthly = debt_request("automatic");
            missing_monthly.monthly_payment = None;
            assert!(create_debt(&missing_monthly).await.is_err());

            let mut zero_monthly = debt_request("automatic");
            zero_monthly.monthly_payment = Some(Decimal::ZERO);
            assert!(create_debt(&zero_monthly).await.is_err());

            let debt = create_debt(&debt_request("automatic")).await.unwrap();
            assert!(debt.subscription_id.is_some());
            assert_eq!(debt.payment_days, vec![today().day() as u8]);
            assert_eq!(list_subscriptions().await.unwrap().len(), 1);

            let updated = update_debt(
                &debt.id,
                &UpdateDebtRequest {
                    name: Some("Updated automatic".into()),
                    total_amount: Some(dec!(120)),
                    monthly_payment: Some(dec!(20)),
                    payment_days: Some(vec![1]),
                    debt_type: Some("automatic".into()),
                    notes: Some("automatic".into()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            assert_eq!(updated.monthly_payment, Some(dec!(20)));
            assert_eq!(list_subscriptions().await.unwrap()[0].amount, dec!(20));

            let manual = update_debt(
                &debt.id,
                &UpdateDebtRequest {
                    debt_type: Some("manual".into()),
                    clear_notes: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            assert!(manual.subscription_id.is_none());
            assert!(manual.notes.is_none());
            assert!(list_subscriptions().await.unwrap().is_empty());

            let automatic = update_debt(
                &debt.id,
                &UpdateDebtRequest {
                    debt_type: Some("automatic".into()),
                    monthly_payment: Some(dec!(20)),
                    payment_days: Some(vec![today().day() as u8]),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            assert!(automatic.subscription_id.is_some());

            record_debt_payment(&debt.id, &RecordDebtPaymentRequest {
                amount: dec!(60),
                paid_on: today(),
                note: None,
            }).await.unwrap();
            let closed = record_debt_payment(&debt.id, &RecordDebtPaymentRequest {
                amount: dec!(60),
                paid_on: today(),
                note: None,
            }).await.unwrap();
            assert!(!closed.is_active);
            assert_eq!(list_debt_payments(&debt.id).await.unwrap().len(), 2);
            let subscription = list_subscriptions().await.unwrap();
            assert_eq!(subscription.len(), 1);
            assert!(!subscription[0].is_active);

            delete_debt(&debt.id).await.unwrap();
            assert!(list_debt_payments(&debt.id).await.unwrap().is_empty());
        });
    }

    #[test]
    fn local_backup_export_and_import_round_trip() {
        let _test = LocalTest::new();
        run(async {
            let accounts = list_accounts().await.unwrap();
            let categories = list_categories().await.unwrap();
            let subscription = create_subscription(&subscription_request(today())).await.unwrap();
            let transaction = create_transaction(&transaction_request(today())).await.unwrap();
            let debt = create_debt(&debt_request("manual")).await.unwrap();
            record_debt_payment(&debt.id, &RecordDebtPaymentRequest {
                amount: dec!(10),
                paid_on: today(),
                note: Some("backup payment".into()),
            }).await.unwrap();

            let backup = export_backup().await.unwrap();
            assert_eq!(backup["format_version"], 1);
            assert_eq!(backup["accounts"].as_array().unwrap().len(), accounts.len());
            assert_eq!(backup["categories"].as_array().unwrap().len(), categories.len());
            assert_eq!(backup["subscriptions"].as_array().unwrap().len(), 1);
            assert_eq!(backup["transactions"].as_array().unwrap().len(), 1);
            assert_eq!(backup["debts"].as_array().unwrap().len(), 1);
            assert_eq!(backup["debt_payments"].as_array().unwrap().len(), 1);

            let mut changed = backup.clone();
            changed["accounts"][0]["user_id"] = serde_json::json!(Uuid::new_v4());
            changed["categories"][0]["user_id"] = serde_json::json!(Uuid::new_v4());
            changed["debts"][0]["user_id"] = serde_json::json!(Uuid::new_v4());
            changed["debts"][0]["paid_amount"] = serde_json::json!(0);
            let counts = import_backup(&changed).await.unwrap();
            assert_eq!(counts["accounts"], accounts.len());
            assert_eq!(counts["categories"], categories.len());
            assert_eq!(counts["subscriptions"], 1);
            assert_eq!(counts["transactions"], 1);
            assert_eq!(counts["debts"], 1);
            assert_eq!(counts["debt_payments"], 1);
            assert_eq!(list_subscriptions().await.unwrap().len(), 1);
            assert_eq!(list_transactions().await.unwrap().len(), 1);
            assert_eq!(list_debts().await.unwrap()[0].paid_amount, dec!(10));
            assert_eq!(list_accounts().await.unwrap()[0].user_id, Uuid::nil());

            assert!(import_backup(&serde_json::json!({"format_version": 2})).await.is_err());
            assert!(import_backup(&serde_json::json!({
                "format_version": 1,
                "accounts": [],
                "categories": [],
            })).await.is_err());
            assert!(import_backup(&serde_json::json!({
                "format_version": 1,
                "accounts": "bad",
                "categories": [],
            })).await.is_err());

            let _ = (subscription.id, transaction.id);
        });
    }

    #[test]
    fn local_report_summary_sums_subscriptions_and_transactions() {
        let _test = LocalTest::new();
        run(async {
            let category = list_categories().await.unwrap()[0].clone();
            let start = date(2024, 1, 1);
            let subscription = create_subscription(&CreateSubscriptionRequest {
                category_id: Some(category.id),
                start_date: start,
                schedule_day: Some(1),
                ..subscription_request(start)
            })
            .await
            .unwrap();
            let transaction = create_transaction(&CreateTransactionRequest {
                category_id: Some(category.id),
                date: date(2024, 1, 15),
                amount: dec!(5),
                ..transaction_request(date(2024, 1, 15))
            })
            .await
            .unwrap();

            let report = get_report_summary(2024, 1).await.unwrap();
            assert_eq!(report.monthly_summary.subscriptions_total, dec!(9.99));
            assert_eq!(report.monthly_summary.transactions_total, dec!(5));
            assert_eq!(report.monthly_summary.overall_total, dec!(14.99));
            assert_eq!(report.monthly_summary.subscriptions_count, 1);
            assert_eq!(report.monthly_summary.transactions_count, 1);
            assert_eq!(report.category_breakdown.len(), 1);
            assert_eq!(report.category_breakdown[0].category_name, "Uncategorized");
            assert_eq!(report.category_breakdown[0].count, 1);

            let empty = get_report_summary(2023, 2).await.unwrap();
            assert_eq!(empty.monthly_summary.overall_total, Decimal::ZERO);
            let _ = (subscription.id, transaction.id);
        });
    }

    #[test]
    fn local_sync_snapshot_and_merge_keep_the_newest_data() {
        let _test = LocalTest::new();
        run(async {
            let account = list_accounts().await.unwrap()[0].clone();
            let category = list_categories().await.unwrap()[0].clone();
            let subscription = create_subscription(&subscription_request(today())).await.unwrap();
            let transaction = create_transaction(&transaction_request(today())).await.unwrap();
            let debt = create_debt(&debt_request("manual")).await.unwrap();

            let snapshot = sync_snapshot().await.unwrap();
            let document: SyncDocument = serde_json::from_str(&snapshot).unwrap();
            assert!(!document.device_id.is_empty());
            assert_eq!(document.subscriptions.len(), 1);
            assert_eq!(document.transactions.len(), 1);
            assert_eq!(document.debts.len(), 1);
            let second = sync_snapshot().await.unwrap();
            let second_document: SyncDocument = serde_json::from_str(&second).unwrap();
            assert_eq!(document.device_id, second_document.device_id);

            let mut newer = subscription.clone();
            newer.name = "Remote newer".into();
            newer.updated_at = subscription.updated_at + chrono::Duration::minutes(1);
            let older = Transaction {
                updated_at: transaction.updated_at - chrono::Duration::minutes(1),
                name: "Remote older".into(),
                ..transaction.clone()
            };
            let fresh = Transaction {
                id: Uuid::new_v4(),
                name: "Remote fresh".into(),
                updated_at: transaction.updated_at + chrono::Duration::minutes(1),
                ..transaction.clone()
            };
            let remote = SyncDocument {
                format_version: 1,
                device_id: "remote-device".into(),
                subscriptions: vec![newer],
                transactions: vec![older, fresh.clone()],
                accounts: vec![account.clone()],
                categories: vec![category.clone()],
                debts: Vec::new(),
                debt_payments: Vec::new(),
                tombstones: vec![Tombstone {
                    kind: "transaction".into(),
                    id: transaction.id,
                    deleted_at: Utc::now(),
                }],
            };
            let merged = sync_merge(&[
                "not json".into(),
                serde_json::to_string(&remote).unwrap(),
            ])
            .await
            .unwrap();
            let merged: SyncDocument = serde_json::from_str(&merged).unwrap();
            assert_eq!(merged.subscriptions.iter().find(|item| item.id == subscription.id).unwrap().name, "Remote newer");
            assert!(merged.transactions.iter().all(|item| item.id != transaction.id));
            assert!(merged.transactions.iter().any(|item| item.id == fresh.id));
            assert_eq!(merged.debts.len(), 1);
            assert_eq!(merged.debts[0].id, debt.id);

            let empty_merge = sync_merge(&[]).await.unwrap();
            let empty_merge: SyncDocument = serde_json::from_str(&empty_merge).unwrap();
            assert_eq!(empty_merge.device_id, document.device_id);
        });
    }

    #[test]
    fn storage_facade_and_backup_api_use_the_offline_store() {
        let _test = LocalTest::new();
        crate::api::client::force_offline_for_tests();
        run(async {
            use crate::storage::Storage;

            let categories = Storage::list_categories().await.unwrap();
            let category = Storage::create_category(&CreateCategoryRequest {
                name: "Facade category".into(),
                icon: None,
                color: None,
            })
            .await
            .unwrap();
            Storage::update_category(
                &category.id,
                &UpdateCategoryRequest {
                    name: Some("Facade renamed".into()),
                    icon: None,
                    color: None,
                },
            )
            .await
            .unwrap();
            Storage::set_default_category(&categories[0].id).await.unwrap();

            let accounts = Storage::list_accounts().await.unwrap();
            let account = Storage::create_account(&CreateAccountRequest {
                name: "Facade account".into(),
                icon: None,
                color: None,
            })
            .await
            .unwrap();
            Storage::update_account(
                &account.id,
                &UpdateAccountRequest {
                    name: Some("Facade account renamed".into()),
                    icon: None,
                    color: None,
                },
            )
            .await
            .unwrap();
            Storage::set_default_account(&accounts[0].id).await.unwrap();

            let subscription = Storage::create_subscription(&subscription_request(today()))
                .await
                .unwrap();
            Storage::update_subscription(
                &subscription.id,
                &UpdateSubscriptionRequest {
                    name: Some("Facade subscription".into()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            assert_eq!(Storage::list_subscriptions().await.unwrap().len(), 1);
            assert!(!Storage::calendar_month(today().year(), today().month())
                .await
                .unwrap()
                .is_empty());

            let transaction = Storage::create_transaction(&transaction_request(today()))
                .await
                .unwrap();
            Storage::update_transaction(
                &transaction.id,
                &UpdateTransactionRequest {
                    name: Some("Facade transaction".into()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            assert_eq!(Storage::list_transactions().await.unwrap().len(), 1);

            let debt = Storage::create_debt(&debt_request("manual")).await.unwrap();
            Storage::update_debt(
                &debt.id,
                &UpdateDebtRequest {
                    name: Some("Facade debt".into()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            Storage::record_debt_payment(
                &debt.id,
                &RecordDebtPaymentRequest {
                    amount: dec!(10),
                    paid_on: today(),
                    note: None,
                },
            )
            .await
            .unwrap();
            assert_eq!(Storage::list_debts().await.unwrap().len(), 1);
            assert_eq!(Storage::list_debt_payments(&debt.id).await.unwrap().len(), 1);
            assert_eq!(
                Storage::get_report_summary(today().year(), today().month())
                    .await
                    .unwrap()
                    .monthly_summary
                    .transactions_count,
                1
            );

            Storage::delete_debt(&debt.id).await.unwrap();
            Storage::delete_transaction(&transaction.id).await.unwrap();
            Storage::delete_subscription(&subscription.id).await.unwrap();
            Storage::delete_account(&account.id).await.unwrap();
            Storage::delete_category(&category.id).await.unwrap();

            let backup = crate::api::backup::export().await.unwrap();
            assert_eq!(backup["format_version"], 1);
            let counts = crate::api::backup::import(&backup).await.unwrap();
            assert_eq!(counts["accounts"], 1);
            assert_eq!(counts["categories"], 1);
            crate::api::client::reset_offline_for_tests();
        });
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
    for debt in &mut debts {
        debt.user_id = Uuid::nil();
        // Server backups omit the derived paid_amount field.
        debt.paid_amount = (debt.total_amount - debt.remaining_amount).max(Decimal::ZERO);
    }
    for payment in &mut debt_payments { payment.user_id = Uuid::nil(); }

    normalize_defaults(&mut accounts);
    // Restoring a backup must preserve every category from the backup,
    // including legacy system categories from older installations.
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
        let dir = data_dir().ok_or_else(|| ApiError::Network("No data directory".into()))?;
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
