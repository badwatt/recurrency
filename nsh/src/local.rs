//! Device-local mode: the same screens without a server, stored in the app's
//! local storage. It mirrors the REST resources the UI consumes.

use chrono::{Local, Utc};
use nsh_core::storage::Storage;
use rust_decimal::Decimal;
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

use crate::auth::{AuthSession, AuthUser};
use crate::models::{
    Account, AccountWithCounts, Category, CategoryWithCounts, Debt, DebtPayment, Rates,
    Subscription, Transaction,
};
use crate::subscriptions::CreateSubscriptionRequest;
use crate::transactions::CreateTransactionRequest;

const MODE_KEY: &str = "recurrency.local-mode";
const TOKEN: &str = "local";
const PREFIX: &str = "recurrency.local.";

pub fn enabled() -> bool {
    Storage::get_item(MODE_KEY).as_deref() == Some("1")
}

pub fn enable() {
    let _ = Storage::set_item(MODE_KEY, "1");
    seed();
}

pub fn disable() {
    let _ = Storage::remove_item(MODE_KEY);
}

pub fn session() -> AuthSession {
    AuthSession {
        token: TOKEN.to_string(),
        user: AuthUser {
            id: Uuid::nil().to_string(),
            email: "local@recurrency".to_string(),
            avatar: None,
            name: Some("Local user".to_string()),
            currency: "EUR".to_string(),
            verified: true,
            is_admin: true,
        },
        requires_verification: false,
    }
}

fn key(name: &str) -> String {
    format!("{PREFIX}{name}")
}

fn load<T: DeserializeOwned>(name: &str) -> Vec<T> {
    Storage::get_item(&key(name))
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn save<T: Serialize>(name: &str, items: &[T]) -> Result<(), String> {
    let json = serde_json::to_string(items).map_err(|error| error.to_string())?;
    Storage::set_item(key(name), json).map_err(|error| error.to_string())
}

fn seed() {
    if Storage::get_item(&key("accounts")).is_none() {
        let account = Account {
            id: Uuid::from_u128(1),
            name: "General".to_string(),
            icon: Some("wallet".to_string()),
            color: Some("#6b7280".to_string()),
            is_default: true,
        };
        let category = Category {
            id: Uuid::from_u128(2),
            name: "Uncategorized".to_string(),
            icon: Some("tag".to_string()),
            color: Some("#6b7280".to_string()),
            is_system: true,
            is_default: true,
        };
        let _ = save("accounts", &[account]);
        let _ = save("categories", &[category]);
        let _ = save("subscriptions", &Vec::<Subscription>::new());
        let _ = save("transactions", &Vec::<Transaction>::new());
        let _ = save("debts", &Vec::<Debt>::new());
        let _ = save("debt_payments", &Vec::<DebtPayment>::new());
    }
}

pub fn rates() -> Rates {
    Rates {
        base: "EUR".to_string(),
        rates: std::collections::HashMap::new(),
    }
}

pub fn accounts() -> Vec<Account> {
    seed();
    let items = load("accounts");
    if items.is_empty() {
        let account = Account {
            id: Uuid::from_u128(1),
            name: "General".to_string(),
            icon: Some("wallet".to_string()),
            color: Some("#6b7280".to_string()),
            is_default: true,
        };
        let _ = save("accounts", &[account.clone()]);
        vec![account]
    } else {
        items
    }
}
pub fn categories() -> Vec<Category> {
    seed();
    let items = load("categories");
    if items.is_empty() {
        let category = Category {
            id: Uuid::from_u128(2),
            name: "Uncategorized".to_string(),
            icon: Some("tag".to_string()),
            color: Some("#6b7280".to_string()),
            is_system: true,
            is_default: true,
        };
        let _ = save("categories", &[category.clone()]);
        vec![category]
    } else {
        items
    }
}
pub fn subscriptions() -> Vec<Subscription> {
    seed();
    load("subscriptions")
}
pub fn transactions() -> Vec<Transaction> {
    seed();
    load("transactions")
}
pub fn debts() -> Vec<Debt> {
    seed();
    load("debts")
}
pub fn payments() -> Vec<DebtPayment> {
    load("debt_payments")
}

pub fn money_counts() -> (Vec<AccountWithCounts>, Vec<CategoryWithCounts>) {
    let subscriptions = subscriptions();
    let transactions = transactions();
    let accounts = accounts()
        .into_iter()
        .map(|account| AccountWithCounts {
            subscription_count: subscriptions
                .iter()
                .filter(|item| item.account_id == account.id)
                .count() as i64,
            transaction_count: transactions
                .iter()
                .filter(|item| item.account_id == account.id)
                .count() as i64,
            account,
        })
        .collect();
    let categories = categories()
        .into_iter()
        .map(|category| CategoryWithCounts {
            subscription_count: subscriptions
                .iter()
                .filter(|item| item.category_id == category.id)
                .count() as i64,
            transaction_count: transactions
                .iter()
                .filter(|item| item.category_id == category.id)
                .count() as i64,
            category,
        })
        .collect();
    (accounts, categories)
}

fn category_id() -> Uuid {
    categories()
        .first()
        .map_or_else(Uuid::new_v4, |category| category.id)
}

pub fn create_subscription(req: &CreateSubscriptionRequest) -> Result<Subscription, String> {
    let now = Utc::now();
    let item = Subscription {
        id: Uuid::new_v4(),
        account_id: req.account_id,
        category_id: req.category_id.unwrap_or_else(category_id),
        name: req.name.clone(),
        amount: req.amount,
        currency: req.currency.clone(),
        recurrence: req.recurrence.clone(),
        start_date: req.start_date,
        next_billing_date: Some(req.start_date),
        end_date: None,
        is_active: req.is_active,
        notes: None,
        logo_url: None,
        created_at: Some(now),
        updated_at: Some(now),
    };
    let mut items = subscriptions();
    items.push(item.clone());
    save("subscriptions", &items)?;
    Ok(item)
}

pub fn update_entity<T: Serialize + DeserializeOwned + Clone>(
    name: &str,
    id: Uuid,
    patch: &serde_json::Value,
) -> Result<T, String> {
    let mut items: Vec<T> = load(name);
    let index = items.iter().position(|_| true);
    let _ = index;
    // The caller passes a typed collection; find by serializing each item.
    let found = items.iter().position(|item| {
        serde_json::to_value(item)
            .ok()
            .and_then(|value| {
                value
                    .get("id")
                    .and_then(|id| id.as_str())
                    .map(str::to_string)
            })
            .as_deref()
            == Some(id.to_string().as_str())
    });
    let Some(index) = found else {
        return Err("Item not found".to_string());
    };
    let mut value = serde_json::to_value(&items[index]).map_err(|error| error.to_string())?;
    if let (Some(target), Some(source)) = (value.as_object_mut(), patch.as_object()) {
        for (key, value) in source {
            target.insert(key.clone(), value.clone());
        }
    }
    let updated: T = serde_json::from_value(value).map_err(|error| error.to_string())?;
    items[index] = updated.clone();
    save(name, &items)?;
    Ok(updated)
}

pub fn delete_entity(name: &str, id: Uuid) -> Result<(), String> {
    let mut value: Vec<serde_json::Value> = load(name);
    value.retain(|item| item.get("id").and_then(|id| id.as_str()) != Some(id.to_string().as_str()));
    save(name, &value)
}

pub fn create_transaction(req: &CreateTransactionRequest) -> Result<Transaction, String> {
    let now = Utc::now();
    let item = Transaction {
        id: Uuid::new_v4(),
        account_id: req.account_id,
        category_id: req.category_id.unwrap_or_else(category_id),
        name: req.name.clone(),
        amount: req.amount,
        currency: req.currency.clone(),
        date: req.date,
        notes: req.notes.clone(),
        logo_url: req.logo_url.clone(),
        created_at: Some(now),
        updated_at: Some(now),
    };
    let mut items = transactions();
    items.push(item.clone());
    save("transactions", &items)?;
    Ok(item)
}

pub fn set_default_account(id: Uuid) -> Result<Account, String> {
    let mut items = accounts();
    let mut found = None;
    for account in &mut items {
        account.is_default = account.id == id;
        if account.id == id {
            found = Some(account.clone());
        }
    }
    if let Some(account) = found {
        save("accounts", &items)?;
        Ok(account)
    } else {
        Err("Account not found.".to_string())
    }
}

pub fn create_account(
    name: &str,
    icon: Option<&str>,
    color: Option<&str>,
) -> Result<Account, String> {
    let item = Account {
        id: Uuid::new_v4(),
        name: name.to_string(),
        icon: icon.map(str::to_string),
        color: color.map(str::to_string),
        is_default: accounts().is_empty(),
    };
    let mut items = accounts();
    items.push(item.clone());
    save("accounts", &items)?;
    Ok(item)
}

pub fn create_category(
    name: &str,
    icon: Option<&str>,
    color: Option<&str>,
) -> Result<Category, String> {
    let item = Category {
        id: Uuid::new_v4(),
        name: name.to_string(),
        icon: icon.map(str::to_string),
        color: color.map(str::to_string),
        is_system: false,
        is_default: false,
    };
    let mut items = categories();
    items.push(item.clone());
    save("categories", &items)?;
    Ok(item)
}

pub fn create_debt(req: &crate::debts::CreateDebtRequest) -> Result<Debt, String> {
    let now = Utc::now();
    let item = Debt {
        id: Uuid::new_v4(),
        account_id: req.account_id,
        category_id: req.category_id.unwrap_or_else(category_id),
        name: req.name.clone(),
        currency: req.currency.clone(),
        total_amount: req.total_amount,
        remaining_amount: req.total_amount,
        paid_amount: Decimal::ZERO,
        debt_type: req.debt_type.clone(),
        monthly_payment: req.monthly_payment,
        payment_days: req.payment_days.clone(),
        start_date: req.start_date.unwrap_or_else(today),
        next_payment_date: None,
        subscription_id: None,
        is_active: true,
        notes: None,
        logo_url: None,
        created_at: Some(now),
        updated_at: Some(now),
    };
    let mut items = debts();
    items.push(item.clone());
    save("debts", &items)?;
    Ok(item)
}

pub fn record_payment(
    debt_id: Uuid,
    req: &crate::debts::RecordDebtPaymentRequest,
) -> Result<DebtPayment, String> {
    let items = payments();
    let payment = DebtPayment {
        id: Uuid::new_v4(),
        debt_id,
        amount: req.amount,
        paid_on: req.paid_on,
        note: req.note.clone(),
    };
    let mut next = items;
    next.push(payment.clone());
    save("debt_payments", &next)?;
    let mut debts = self::debts();
    let Some(debt) = debts.iter_mut().find(|debt| debt.id == debt_id) else {
        return Err("Debt not found".to_string());
    };
    debt.remaining_amount = (debt.remaining_amount - req.amount).max(Decimal::ZERO);
    debt.paid_amount = debt.total_amount - debt.remaining_amount;
    debt.updated_at = Some(Utc::now());
    save("debts", &debts)?;
    Ok(payment)
}

pub fn today() -> chrono::NaiveDate {
    Local::now().date_naive()
}

fn month_range(year: i32, month: u32) -> (chrono::NaiveDate, chrono::NaiveDate) {
    let start = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default();
    let end = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap_or(start);
    (start, end.pred_opt().unwrap_or(start))
}

pub fn calendar(year: i32, month: u32) -> (Vec<crate::models::CalendarItem>, Vec<Transaction>) {
    let (from, to) = month_range(year, month);
    let items = subscriptions()
        .into_iter()
        .filter(|subscription| subscription.is_active)
        .flat_map(|subscription| {
            crate::recurrence::occurrences_between(
                subscription.start_date,
                &subscription.recurrence,
                from,
                to,
                subscription.end_date,
                subscription.amount,
            )
            .into_iter()
            .map(move |(date, amount)| crate::models::CalendarItem {
                subscription_id: subscription.id,
                name: subscription.name.clone(),
                amount,
                currency: subscription.currency.clone(),
                frequency: subscription.recurrence.unit.clone(),
                billing_date: date,
                is_active: subscription.is_active,
                logo_url: subscription.logo_url.clone(),
            })
        })
        .collect();
    (items, transactions())
}

pub fn report(year: i32, month: u32) -> crate::models::ReportResponse {
    let categories = categories();
    let subscriptions = subscriptions();
    let transactions = transactions();
    let (from, to) = month_range(year, month);
    let mut subscription_total = Decimal::ZERO;
    let mut subscription_count = 0i64;
    let mut breakdown: Vec<crate::models::CategoryBreakdown> = Vec::new();
    for subscription in &subscriptions {
        for (_, amount) in crate::recurrence::occurrences_between(
            subscription.start_date,
            &subscription.recurrence,
            from,
            to,
            subscription.end_date,
            subscription.amount,
        ) {
            subscription_total += amount;
            subscription_count += 1;
            add_breakdown(
                &mut breakdown,
                &categories,
                subscription.category_id,
                amount,
            );
        }
    }
    let mut transaction_total = Decimal::ZERO;
    let mut transaction_count = 0i64;
    for transaction in &transactions {
        if transaction.date >= from && transaction.date <= to {
            transaction_total += transaction.amount;
            transaction_count += 1;
            add_breakdown(
                &mut breakdown,
                &categories,
                transaction.category_id,
                transaction.amount,
            );
        }
    }
    let total = subscription_total + transaction_total;
    for entry in &mut breakdown {
        entry.percentage = if total > Decimal::ZERO {
            (entry.total / total * Decimal::from(100))
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
        } else {
            0.0
        };
    }
    let mut trends = Vec::new();
    for offset in (0..10).rev() {
        let mut y = year;
        let mut m = month as i32 - offset;
        while m < 1 {
            m += 12;
            y -= 1;
        }
        let (start, end) = month_range(y, m as u32);
        let mut total = Decimal::ZERO;
        for subscription in &subscriptions {
            total += crate::recurrence::occurrences_between(
                subscription.start_date,
                &subscription.recurrence,
                start,
                end,
                subscription.end_date,
                subscription.amount,
            )
            .into_iter()
            .map(|(_, amount)| amount)
            .sum::<Decimal>();
        }
        total += transactions
            .iter()
            .filter(|transaction| transaction.date >= start && transaction.date <= end)
            .map(|transaction| transaction.amount)
            .sum::<Decimal>();
        trends.push(crate::models::SpendingTrend {
            period: format!("{y}-{m:02}"),
            total,
        });
    }
    crate::models::ReportResponse {
        monthly_summary: crate::models::MonthlySummary {
            year,
            month,
            subscriptions_total: subscription_total,
            transactions_total: transaction_total,
            overall_total: total,
            subscriptions_count: subscription_count,
            transactions_count: transaction_count,
        },
        category_breakdown: breakdown,
        recent_trends: trends,
    }
}

fn add_breakdown(
    entries: &mut Vec<crate::models::CategoryBreakdown>,
    categories: &[Category],
    category_id: Uuid,
    amount: Decimal,
) {
    let entry = entries
        .iter_mut()
        .find(|entry| entry.category_id.as_deref() == Some(category_id.to_string().as_str()));
    if let Some(entry) = entry {
        entry.total += amount;
        entry.count += 1;
    } else {
        let name = categories
            .iter()
            .find(|category| category.id == category_id)
            .map_or_else(
                || "Uncategorized".to_string(),
                |category| category.name.clone(),
            );
        entries.push(crate::models::CategoryBreakdown {
            category_id: Some(category_id.to_string()),
            category_name: name,
            total: amount,
            count: 1,
            percentage: 0.0,
        });
    }
}

pub fn reset() {
    for name in [
        "accounts",
        "categories",
        "subscriptions",
        "transactions",
        "debts",
        "debt_payments",
    ] {
        let _ = Storage::remove_item(&key(name));
    }
    disable();
}

pub fn export_backup() -> serde_json::Value {
    serde_json::json!({
        "version": 1,
        "accounts": accounts(),
        "categories": categories(),
        "subscriptions": subscriptions(),
        "transactions": transactions(),
        "debts": debts(),
        "debt_payments": payments(),
    })
}

pub fn import_backup(backup: &serde_json::Value) -> Result<(), String> {
    for name in [
        "accounts",
        "categories",
        "subscriptions",
        "transactions",
        "debts",
        "debt_payments",
    ] {
        if let Some(items) = backup.get(name) {
            let json = serde_json::to_string(items).map_err(|error| error.to_string())?;
            Storage::set_item(key(name), json).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_account_json_round_trips() {
        let account = Account {
            id: Uuid::from_u128(1),
            name: "General".to_string(),
            icon: Some("wallet".to_string()),
            color: Some("#6b7280".to_string()),
            is_default: true,
        };
        let json = serde_json::to_string(&[account.clone()]).expect("account serializes");
        let parsed: Vec<Account> = serde_json::from_str(&json).expect("account parses");
        assert_eq!(parsed, vec![account]);
    }
}
