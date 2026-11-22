//! Authenticated REST access for the native screens.

use chrono::Local;
use serde::{Serialize, de::DeserializeOwned};

use crate::dashboard_data::{
    build_account_budgets, build_category_budgets, upcoming_payments, AccountBudget, CategoryBudget,
    DashboardData, UpcomingPayment,
};
use crate::models::{Account, AccountWithCounts, AdminUser, AiSettings, CalendarItem, Category, CategoryWithCounts, Debt, DebtPayment, NotificationPreferences, Rates, ReportResponse, Subscription, Transaction};
use crate::settings::{SettingsResponse, UpdateSettingsRequest};
use crate::subscriptions::CreateSubscriptionRequest;
use crate::transactions::CreateTransactionRequest;
use crate::debts::{CreateDebtRequest, RecordDebtPaymentRequest};
use nsh_core::prelude::*;

pub struct Client {
    server: String,
    token: String,
}

impl Client {
    pub fn new(server: String, token: String) -> Self {
        Self { server, token }
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        let request = HttpRequest::get(format!("{}{path}", self.server.trim_end_matches('/')))
            .header("Authorization", format!("Bearer {}", self.token));
        let response = fetch(request).await?;
        if !response.ok() {
            return Err(format!("The server answered {}", response.status()));
        }
        serde_json::from_str(&response.text())
            .map_err(|_| "The server returned an unexpected response".to_string())
    }

    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T, String> {
        let json = serde_json::to_string(body).map_err(|error| error.to_string())?;
        let request = HttpRequest::post(format!("{}{path}", self.server.trim_end_matches('/')), json)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json");
        let response = fetch(request).await?;
        if !response.ok() {
            return Err(format!("The server answered {}", response.status()));
        }
        serde_json::from_str(&response.text())
            .map_err(|_| "The server returned an unexpected response".to_string())
    }

    pub async fn put<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T, String> {
        let json = serde_json::to_string(body).map_err(|error| error.to_string())?;
        let request = HttpRequest::new("PUT", format!("{}{path}", self.server.trim_end_matches('/')))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .body(json);
        let response = fetch(request).await?;
        if !response.ok() {
            return Err(format!("The server answered {}", response.status()));
        }
        serde_json::from_str(&response.text())
            .map_err(|_| "The server returned an unexpected response".to_string())
    }

    pub async fn delete(&self, path: &str) -> Result<(), String> {
        let request = HttpRequest::new("DELETE", format!("{}{path}", self.server.trim_end_matches('/')))
            .header("Authorization", format!("Bearer {}", self.token));
        let response = fetch(request).await?;
        if response.ok() {
            Ok(())
        } else {
            Err(format!("The server answered {}", response.status()))
        }
    }

    async fn rates(&self, currency: &str) -> Rates {
        self.get(&format!("/api/currency/rates?base={currency}"))
            .await
            .unwrap_or_default()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SubscriptionsData {
    pub subscriptions: Vec<Subscription>,
    pub accounts: Vec<Account>,
    pub categories: Vec<Category>,
    pub rates: Rates,
}

pub async fn load_dashboard(server: String, token: String, currency: String) -> Result<DashboardData, String> {
    let api = Client::new(server, token);
    let accounts: Vec<Account> = api.get("/api/accounts").await?;
    let subscriptions: Vec<Subscription> = api.get("/api/subscriptions").await?;
    let transactions: Vec<Transaction> = api.get("/api/transactions").await?;
    let categories: Vec<Category> = api.get("/api/categories").await?;
    let rates = api.rates(&currency).await;

    let today = Local::now().date_naive();
    let account_budgets: Vec<AccountBudget> =
        build_account_budgets(&accounts, &subscriptions, &transactions, today, &rates);
    let category_budgets: Vec<CategoryBudget> =
        build_category_budgets(&categories, &subscriptions, &transactions, today, &rates);
    let upcoming: Vec<UpcomingPayment> = upcoming_payments(&subscriptions, &transactions, &accounts, today);
    let active_subscriptions = subscriptions.iter().filter(|subscription| subscription.is_active).count();

    Ok(DashboardData {
        accounts: account_budgets,
        categories: category_budgets,
        upcoming,
        rates,
        active_subscriptions,
    })
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TransactionsData {
    pub transactions: Vec<Transaction>,
    pub accounts: Vec<Account>,
    pub categories: Vec<Category>,
    pub rates: Rates,
}

pub async fn load_transactions(
    server: String,
    token: String,
    currency: String,
) -> Result<TransactionsData, String> {
    let api = Client::new(server, token);
    let transactions: Vec<Transaction> = api.get("/api/transactions").await?;
    let accounts: Vec<Account> = api.get("/api/accounts").await?;
    let categories: Vec<Category> = api.get("/api/categories").await?;
    let rates = api.rates(&currency).await;
    Ok(TransactionsData {
        transactions,
        accounts,
        categories,
        rates,
    })
}

pub async fn create_transaction(
    server: String,
    token: String,
    request: &CreateTransactionRequest,
) -> Result<Transaction, String> {
    Client::new(server, token)
        .post("/api/transactions", request)
        .await
}

pub async fn update_transaction<B: Serialize>(
    server: String,
    token: String,
    id: uuid::Uuid,
    request: &B,
) -> Result<Transaction, String> {
    Client::new(server, token)
        .put(&format!("/api/transactions/{id}"), request)
        .await
}

pub async fn delete_transaction(
    server: String,
    token: String,
    id: uuid::Uuid,
) -> Result<(), String> {
    Client::new(server, token)
        .delete(&format!("/api/transactions/{id}"))
        .await
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DebtsData {
    pub debts: Vec<Debt>,
    pub accounts: Vec<Account>,
    pub categories: Vec<Category>,
    pub rates: Rates,
}

pub async fn load_debts(
    server: String,
    token: String,
    currency: String,
) -> Result<DebtsData, String> {
    let api = Client::new(server, token);
    let debts: Vec<Debt> = api.get("/api/debts").await?;
    let accounts: Vec<Account> = api.get("/api/accounts").await?;
    let categories: Vec<Category> = api.get("/api/categories").await?;
    let rates = api.rates(&currency).await;
    Ok(DebtsData {
        debts,
        accounts,
        categories,
        rates,
    })
}

pub async fn create_debt(
    server: String,
    token: String,
    request: &CreateDebtRequest,
) -> Result<Debt, String> {
    Client::new(server, token)
        .post("/api/debts", request)
        .await
}

pub async fn update_debt<B: Serialize>(
    server: String,
    token: String,
    id: uuid::Uuid,
    request: &B,
) -> Result<Debt, String> {
    Client::new(server, token)
        .put(&format!("/api/debts/{id}"), request)
        .await
}

pub async fn delete_debt(server: String, token: String, id: uuid::Uuid) -> Result<(), String> {
    Client::new(server, token)
        .delete(&format!("/api/debts/{id}"))
        .await
}

pub async fn load_debt_payments(
    server: String,
    token: String,
    debt_id: uuid::Uuid,
) -> Result<Vec<DebtPayment>, String> {
    Client::new(server, token)
        .get(&format!("/api/debts/{debt_id}/payments"))
        .await
}

pub async fn record_debt_payment(
    server: String,
    token: String,
    debt_id: uuid::Uuid,
    request: &RecordDebtPaymentRequest,
) -> Result<DebtPayment, String> {
    Client::new(server, token)
        .post(&format!("/api/debts/{debt_id}/payments"), request)
        .await
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CalendarData {
    pub items: Vec<CalendarItem>,
    pub transactions: Vec<Transaction>,
    pub rates: Rates,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReportsData {
    pub report: ReportResponse,
    pub categories: Vec<Category>,
}

pub async fn load_notification_preferences(
    server: String,
    token: String,
) -> Result<NotificationPreferences, String> {
    Client::new(server, token)
        .get("/api/settings/notifications")
        .await
}

pub async fn update_notification_preferences<B: Serialize>(
    server: String,
    token: String,
    request: &B,
) -> Result<NotificationPreferences, String> {
    Client::new(server, token)
        .put("/api/settings/notifications", request)
        .await
}

pub async fn load_ai_settings(server: String, token: String) -> Result<AiSettings, String> {
    Client::new(server, token).get("/api/ai/settings").await
}

pub async fn update_ai_settings<B: Serialize>(
    server: String,
    token: String,
    request: &B,
) -> Result<AiSettings, String> {
    Client::new(server, token).put("/api/ai/settings", request).await
}

pub async fn load_money_counts(
    server: String,
    token: String,
) -> Result<(Vec<AccountWithCounts>, Vec<CategoryWithCounts>), String> {
    let api = Client::new(server, token);
    let accounts: Vec<AccountWithCounts> = api.get("/api/accounts/with-counts").await?;
    let categories: Vec<CategoryWithCounts> = api.get("/api/categories/with-counts").await?;
    Ok((accounts, categories))
}

pub async fn load_admin_users(server: String, token: String) -> Result<Vec<AdminUser>, String> {
    Client::new(server, token).get("/api/admin/users").await
}

pub async fn export_backup(server: String, token: String) -> Result<String, String> {
    let request = HttpRequest::get(format!("{}/api/backup", server.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {token}"));
    let response = fetch(request).await?;
    if response.ok() {
        Ok(response.text())
    } else {
        Err(format!("The server answered {}", response.status()))
    }
}

pub async fn update_settings(
    server: String,
    token: String,
    request: &UpdateSettingsRequest,
) -> Result<SettingsResponse, String> {
    Client::new(server, token)
        .put("/api/settings", request)
        .await
}

pub async fn load_report(
    server: String,
    token: String,
    year: i32,
    month: u32,
) -> Result<ReportsData, String> {
    let api = Client::new(server, token);
    let report: ReportResponse = api
        .get(&format!("/api/reports/summary?year={year}&month={month}"))
        .await?;
    let categories: Vec<Category> = api.get("/api/categories").await?;
    Ok(ReportsData { report, categories })
}

pub async fn load_calendar(
    server: String,
    token: String,
    currency: String,
    year: i32,
    month: u32,
) -> Result<CalendarData, String> {
    let api = Client::new(server, token);
    let items: Vec<CalendarItem> = api
        .get(&format!("/api/subscriptions/calendar?year={year}&month={month}"))
        .await?;
    let transactions: Vec<Transaction> = api.get("/api/transactions").await?;
    let rates = api.rates(&currency).await;
    Ok(CalendarData {
        items,
        transactions,
        rates,
    })
}

pub async fn load_subscriptions(
    server: String,
    token: String,
    currency: String,
) -> Result<SubscriptionsData, String> {
    let api = Client::new(server, token);
    let subscriptions: Vec<Subscription> = api.get("/api/subscriptions").await?;
    let accounts: Vec<Account> = api.get("/api/accounts").await?;
    let categories: Vec<Category> = api.get("/api/categories").await?;
    let rates = api.rates(&currency).await;
    Ok(SubscriptionsData {
        subscriptions,
        accounts,
        categories,
        rates,
    })
}

pub async fn create_subscription(
    server: String,
    token: String,
    request: &CreateSubscriptionRequest,
) -> Result<Subscription, String> {
    Client::new(server, token)
        .post("/api/subscriptions", request)
        .await
}

pub async fn update_subscription<B: Serialize>(
    server: String,
    token: String,
    id: uuid::Uuid,
    request: &B,
) -> Result<Subscription, String> {
    Client::new(server, token)
        .put(&format!("/api/subscriptions/{id}"), request)
        .await
}

pub async fn delete_subscription(
    server: String,
    token: String,
    id: uuid::Uuid,
) -> Result<(), String> {
    Client::new(server, token)
        .delete(&format!("/api/subscriptions/{id}"))
        .await
}
