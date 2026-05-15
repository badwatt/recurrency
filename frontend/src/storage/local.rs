use crate::models::category::{Category, CreateCategoryRequest, UpdateCategoryRequest};
use crate::models::subscription::{CalendarItem, CreateSubscriptionRequest, Subscription, UpdateSubscriptionRequest};
use crate::models::transaction::{CreateTransactionRequest, Transaction, UpdateTransactionRequest};
use crate::api::client::ApiError;
use chrono::{Datelike, Utc};

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

// ── Default categories ──

const DEFAULT_CATEGORIES: &[(&str, &str, &str)] = &[
    ("Streaming", "tv", "#ef4444"),
    ("Music", "music", "#f97316"),
    ("Cloud", "cloud", "#3b82f6"),
    ("Gaming", "gamepad-2", "#8b5cf6"),
    ("News", "newspaper", "#6b7280"),
    ("Fitness", "dumbbell", "#22c55e"),
    ("Education", "graduation-cap", "#eab308"),
    ("Utilities", "zap", "#06b6d4"),
    ("Insurance", "shield", "#64748b"),
    ("Other", "ellipsis", "#9ca3af"),
];

fn default_categories() -> Vec<Category> {
    DEFAULT_CATEGORIES.iter().map(|(name, icon, color)| Category {
        id: Uuid::new_v4(),
        user_id: uuid::Uuid::nil(),
        name: name.to_string(),
        icon: Some(icon.to_string()),
        color: Some(color.to_string()),
        is_system: true,
        is_default: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }).collect()
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
    let mut cache = SUBSCRIPTIONS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("subscriptions");
    }
    Ok(cache.clone().unwrap_or_default())
}

pub async fn create_subscription(req: &CreateSubscriptionRequest) -> Result<Subscription, ApiError> {
    let sub = Subscription {
        id: Uuid::new_v4(),
        user_id: Uuid::nil(),
        category_id: req.category_id.unwrap_or(Uuid::nil()),
        name: req.name.clone(),
        amount: req.amount,
        currency: req.currency.clone(),
        frequency: req.frequency.clone(),
        start_date: req.start_date,
        next_billing_date: None,
        is_active: true,
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
    if let Some(frequency) = &req.frequency { sub.frequency = frequency.clone(); }
    if let Some(start_date) = req.start_date { sub.start_date = start_date; }
    if let Some(next_billing_date) = req.next_billing_date { sub.next_billing_date = Some(next_billing_date); }
    if let Some(category_id) = req.category_id { sub.category_id = category_id; }
    if let Some(is_active) = req.is_active { sub.is_active = is_active; }
    if let Some(notes) = &req.notes { sub.notes = Some(notes.clone()); }
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
    save_collection("subscriptions", list)
}

pub async fn calendar_month(year: i32, month: u32) -> Result<Vec<CalendarItem>, ApiError> {
    let subs = list_subscriptions().await?;
    Ok(subs.into_iter().filter(|s| s.is_active).map(|s| {
        let billing_date = compute_billing_date(&s, year, month);
        CalendarItem {
            subscription_id: s.id,
            name: s.name,
            amount: s.amount,
            currency: s.currency,
            frequency: s.frequency,
            billing_date,
            is_active: s.is_active,
            logo_url: s.logo_url,
        }
    }).collect())
}

fn compute_billing_date(sub: &Subscription, year: i32, month: u32) -> chrono::NaiveDate {
    // Use next_billing_date if it falls in the target month, otherwise use start_date day
    if let Some(nbd) = sub.next_billing_date && nbd.year() == year && nbd.month() == month {
        return nbd;
    }
    let day = sub.start_date.day().min(28u32);
    chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap_or(sub.start_date)
}

// ── Transaction operations ──

pub async fn list_transactions() -> Result<Vec<Transaction>, ApiError> {
    let mut cache = TRANSACTIONS.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("transactions");
    }
    Ok(cache.clone().unwrap_or_default())
}

pub async fn create_transaction(req: &CreateTransactionRequest) -> Result<Transaction, ApiError> {
    let txn = Transaction {
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
    if let Some(category_id) = req.category_id { txn.category_id = category_id; }
    if let Some(notes) = &req.notes { txn.notes = Some(notes.clone()); }
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
    save_collection("transactions", list)
}

// ── Category operations ──

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
    list.retain(|c| &c.id != id);
    save_collection("categories", list)
}

pub async fn set_default_category(id: &Uuid) -> Result<(), ApiError> {
    let mut cache = CATEGORIES.lock().map_err(|e| ApiError::Network(e.to_string()))?;
    if cache.is_none() {
        *cache = load_collection("categories");
    }
    let list = cache.as_mut().ok_or_else(|| ApiError::Network("No categories".into()))?;
    // Unset all defaults
    for cat in list.iter_mut() {
        cat.is_default = false;
    }
    // Set the target
    if let Some(cat) = list.iter_mut().find(|c| &c.id == id) {
        cat.is_default = true;
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

    #[test]
    fn default_categories_count() {
        let cats = default_categories();
        assert_eq!(cats.len(), DEFAULT_CATEGORIES.len());
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
        assert!(cats.iter().any(|c| c.name == "Streaming"));
        assert!(cats.iter().any(|c| c.name == "Other"));
    }

    #[test]
    fn compute_billing_date_uses_start_day() {
        let sub = Subscription {
            id: Uuid::new_v4(),
            user_id: Uuid::nil(),
            category_id: Uuid::nil(),
            name: "Test".into(),
            amount: Decimal::new(10, 2),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            next_billing_date: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let result = compute_billing_date(&sub, 2024, 6);
        assert_eq!(result.day(), 15);
    }

    #[test]
    fn compute_billing_date_clamps_day() {
        let sub = Subscription {
            id: Uuid::new_v4(),
            user_id: Uuid::nil(),
            category_id: Uuid::nil(),
            name: "Test".into(),
            amount: Decimal::new(10, 2),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            next_billing_date: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let result = compute_billing_date(&sub, 2024, 2);
        assert_eq!(result.day(), 28); // Feb clamped to 28
    }

    #[test]
    fn local_storage_module_compiles() {
        assert!(true);
    }
}