//! Authenticated REST access for the native screens.

use chrono::Local;
use serde::{Serialize, de::DeserializeOwned};

use crate::dashboard_data::{
    build_account_budgets, build_category_budgets, upcoming_payments, AccountBudget, CategoryBudget,
    DashboardData, UpcomingPayment,
};
use crate::models::{Account, Category, Rates, Subscription, Transaction};
use crate::subscriptions::CreateSubscriptionRequest;
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
