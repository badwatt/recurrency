//! Mock implementations of all service traits for testing.
//!
//! Usage:
//! ```rust,ignore
//! use crate::shared::mocks::*;
//!
//! let mock_auth = MockAuthService::success();
//! let result = mock_auth.register(req).await;
//! ```

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::models::{
    AuthResponse, Claims, CreateUserRequest, LoginRequest, UpdateSettingsRequest, User, UserResponse,
};
use crate::auth::service::AuthServiceTrait;
use crate::categories::models::{Category, CreateCategoryRequest, UpdateCategoryRequest};
use crate::notifications::models::Notification;
use crate::preferences::models::{PreferencesResponse, UpdatePreferencesRequest};
use crate::reports::models::{CategoryBreakdown, MonthlySummary, ReportResponse, SpendingTrend};
use crate::shared::error::{AppError, AppResult};
use crate::subscriptions::models::{Subscription, SubscriptionResponse};
use crate::transactions::models::{Transaction, TransactionResponse};

/// Configuration for mock behavior
#[derive(Clone, Debug)]
pub struct MockConfig {
    pub should_fail: bool,
    pub error_message: String,
    pub delay_ms: u64,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            should_fail: false,
            error_message: "Mock error".to_string(),
            delay_ms: 0,
        }
    }
}

impl MockConfig {
    pub fn success() -> Self {
        Self {
            should_fail: false,
            error_message: String::new(),
            delay_ms: 0,
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            should_fail: true,
            error_message: message.into(),
            delay_ms: 0,
        }
    }

    pub fn delayed(ms: u64) -> Self {
        Self {
            should_fail: false,
            error_message: String::new(),
            delay_ms: ms,
        }
    }
}

// ============================================================================
// Auth Service Mock
// ============================================================================

pub struct MockAuthService {
    pub config: MockConfig,
    pub registered_users: Arc<std::sync::Mutex<Vec<User>>>,
    pub login_calls: Arc<std::sync::Mutex<Vec<LoginRequest>>>,
    pub register_calls: Arc<std::sync::Mutex<Vec<CreateUserRequest>>>,
}

impl Default for MockAuthService {
    fn default() -> Self {
        Self::success()
    }
}

impl MockAuthService {
    pub fn new(config: MockConfig) -> Self {
        Self {
            config,
            registered_users: Arc::new(std::sync::Mutex::new(Vec::new())),
            login_calls: Arc::new(std::sync::Mutex::new(Vec::new())),
            register_calls: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn success() -> Self {
        Self::new(MockConfig::success())
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self::new(MockConfig::failure(message))
    }

    fn make_user(&self, id: Uuid, email: &str, name: &str, verified: bool) -> User {
        User {
            id,
            email: email.to_string(),
            password_hash: "mock_hash".to_string(),
            name: Some(name.to_string()),
            currency: "USD".to_string(),
            verified,
            verification_code: if verified { None } else { Some("123456".to_string()) },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_auth_response(&self, user: User, requires_verification: bool) -> AuthResponse {
        AuthResponse {
            token: format!("mock_token_{}", user.id),
            user: user.into(),
            requires_verification,
        }
    }
}

#[async_trait]
impl AuthServiceTrait for MockAuthService {
    async fn register(&self, req: CreateUserRequest) -> AppResult<AuthResponse> {
        self.register_calls.lock().unwrap().push(req.clone());

        if self.config.should_fail {
            return Err(AppError::Conflict(self.config.error_message.clone()));
        }

        // Check for duplicate email
        {
            let users = self.registered_users.lock().unwrap();
            if users.iter().any(|u| u.email == req.email) {
                return Err(AppError::Conflict("Email already registered".to_string()));
            }
        }

        let user = User {
            id: Uuid::new_v4(),
            email: req.email,
            password_hash: "mock_hash".to_string(),
            name: Some(req.name.clone()),
            currency: "USD".to_string(),
            verified: false,
            verification_code: Some("123456".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.registered_users.lock().unwrap().push(user.clone());

        Ok(AuthResponse {
            token: format!("mock_token_{}", user.id),
            user: user.clone().into(),
            requires_verification: true,
        })
    }

    async fn login(&self, req: LoginRequest) -> AppResult<AuthResponse> {
        self.login_calls.lock().unwrap().push(req.clone());

        if self.config.should_fail {
            return Err(AppError::Unauthorized(self.config.error_message.clone()));
        }

        // Find user by email
        let users = self.registered_users.lock().unwrap();
        let user = users
            .iter()
            .find(|u| u.email == req.email)
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

        Ok(self.make_auth_response(user, false))
    }

    async fn verify_email(&self, user_id: Uuid, code: String) -> AppResult<UserResponse> {
        if self.config.should_fail {
            return Err(AppError::BadRequest(self.config.error_message.clone()));
        }

        let mut users = self.registered_users.lock().unwrap();
        let user = users
            .iter_mut()
            .find(|u| u.id == user_id)
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if user.verified {
            return Err(AppError::BadRequest("Already verified".to_string()));
        }

        if user.verification_code.as_ref() != Some(&code) {
            return Err(AppError::BadRequest("Invalid verification code".to_string()));
        }

        user.verified = true;
        user.verification_code = None;
        user.updated_at = Utc::now();

        Ok(user.clone().into())
    }

    async fn resend_verification(&self, user_id: Uuid) -> AppResult<String> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let mut users = self.registered_users.lock().unwrap();
        let user = users
            .iter_mut()
            .find(|u| u.id == user_id)
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if user.verified {
            return Err(AppError::BadRequest("Already verified".to_string()));
        }

        let code = "123456";
        user.verification_code = Some(code.to_string());
        Ok(code.to_string())
    }

    async fn get_user_by_id(&self, user_id: Uuid) -> AppResult<User> {
        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let users = self.registered_users.lock().unwrap();
        users
            .iter()
            .find(|u| u.id == user_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    async fn update_settings(&self, user_id: Uuid, req: UpdateSettingsRequest) -> AppResult<User> {
        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let mut users = self.registered_users.lock().unwrap();
        let user = users
            .iter_mut()
            .find(|u| u.id == user_id)
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if let Some(name) = req.name {
            user.name = Some(name);
        }
        if let Some(currency) = req.currency {
            user.currency = currency;
        }
        user.updated_at = Utc::now();

        Ok(user.clone())
    }

    async fn verify_token(&self, token: &str) -> AppResult<Claims> {
        if self.config.should_fail {
            return Err(AppError::Unauthorized(self.config.error_message.clone()));
        }

        // Parse mock token format: mock_token_<uuid>
        if !token.starts_with("mock_token_") {
            return Err(AppError::Unauthorized("Invalid token".to_string()));
        }

        let uuid_str = token.trim_start_matches("mock_token_");
        let user_id = Uuid::parse_str(uuid_str)
            .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

        Ok(Claims {
            sub: user_id,
            email: "mock@mock.com".to_string(),
            exp: (Utc::now().timestamp() + 86400) as usize,
            iat: Utc::now().timestamp() as usize,
        })
    }

    async fn get_settings(&self, user_id: Uuid) -> AppResult<UserResponse> {
        let user = self.get_user_by_id(user_id).await?;
        Ok(user.into())
    }
}

// ============================================================================
// Category Service Mock
// ============================================================================

pub struct MockCategoryService {
    pub config: MockConfig,
    pub categories: Arc<std::sync::Mutex<Vec<Category>>>,
    pub create_calls: Arc<std::sync::Mutex<Vec<(Uuid, CreateCategoryRequest)>>>,
    pub update_calls: Arc<std::sync::Mutex<Vec<(Uuid, Uuid, UpdateCategoryRequest)>>>,
    pub delete_calls: Arc<std::sync::Mutex<Vec<(Uuid, Uuid)>>>,
}

impl Default for MockCategoryService {
    fn default() -> Self {
        Self::success()
    }
}

impl MockCategoryService {
    pub fn new(config: MockConfig) -> Self {
        Self {
            config,
            categories: Arc::new(std::sync::Mutex::new(Vec::new())),
            create_calls: Arc::new(std::sync::Mutex::new(Vec::new())),
            update_calls: Arc::new(std::sync::Mutex::new(Vec::new())),
            delete_calls: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn success() -> Self {
        Self::new(MockConfig::success())
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self::new(MockConfig::failure(message))
    }

    pub fn with_categories(categories: Vec<Category>) -> Self {
        let mock = Self::success();
        *mock.categories.lock().unwrap() = categories;
        mock
    }

    fn make_category(&self, id: Uuid, user_id: Uuid, name: &str) -> Category {
        Category {
            id,
            user_id,
            name: name.to_string(),
            icon: Some("icon".to_string()),
            color: Some("#000".to_string()),
            is_system: false,
            is_default: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Add a category to the mock store
    pub fn add_category(&self, category: Category) {
        self.categories.lock().unwrap().push(category);
    }
}

#[async_trait]
impl CategoryServiceTrait for MockCategoryService {
    async fn create(
        &self,
        user_id: Uuid,
        req: CreateCategoryRequest,
    ) -> AppResult<Category> {
        self.create_calls.lock().unwrap().push((user_id, req.clone()));

        if self.config.should_fail {
            return Err(AppError::BadRequest(self.config.error_message.clone()));
        }

        let mut cats = self.categories.lock().unwrap();
        let is_default = cats.is_empty();

        let category = Category {
            id: Uuid::new_v4(),
            user_id,
            name: req.name,
            icon: req.icon,
            color: req.color,
            is_system: false,
            is_default,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        cats.push(category.clone());
        Ok(category)
    }

    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<Category>> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let cats = self.categories.lock().unwrap();
        Ok(cats.iter().filter(|c| c.user_id == user_id).cloned().collect())
    }

    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<Category> {
        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let cats = self.categories.lock().unwrap();
        cats.iter()
            .find(|c| c.id == id && c.user_id == user_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("Category not found".to_string()))
    }

    async fn update(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: UpdateCategoryRequest,
    ) -> AppResult<Category> {
        self.update_calls.lock().unwrap().push((user_id, id, req.clone()));

        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let mut cats = self.categories.lock().unwrap();
        let cat = cats
            .iter_mut()
            .find(|c| c.id == id && c.user_id == user_id)
            .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

        if let Some(name) = req.name {
            cat.name = name;
        }
        cat.updated_at = Utc::now();

        Ok(cat.clone())
    }

    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<()> {
        self.delete_calls.lock().unwrap().push((user_id, id));

        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let mut cats = self.categories.lock().unwrap();

        // Check if category exists and is not default
        let cat = cats
            .iter()
            .find(|c| c.id == id && c.user_id == user_id)
            .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

        if cat.is_default {
            return Err(AppError::BadRequest("Cannot delete the default category".to_string()));
        }

        cats.retain(|c| !(c.id == id && c.user_id == user_id));
        Ok(())
    }

    async fn set_default(&self, user_id: Uuid, id: Uuid) -> AppResult<Category> {
        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let mut cats = self.categories.lock().unwrap();

        // Clear existing defaults
        for c in cats.iter_mut() {
            if c.user_id == user_id {
                c.is_default = false;
            }
        }

        // Set new default
        let cat = cats
            .iter_mut()
            .find(|c| c.id == id && c.user_id == user_id)
            .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

        cat.is_default = true;
        cat.updated_at = Utc::now();

        Ok(cat.clone())
    }
}

// Category Service Trait definition
#[async_trait]
pub trait CategoryServiceTrait: Send + Sync {
    async fn create(&self, user_id: Uuid, req: CreateCategoryRequest) -> AppResult<Category>;
    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<Category>>;
    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<Category>;
    async fn update(&self, user_id: Uuid, id: Uuid, req: UpdateCategoryRequest) -> AppResult<Category>;
    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<()>;
    async fn set_default(&self, user_id: Uuid, id: Uuid) -> AppResult<Category>;
}

// ============================================================================
// Subscription Service Mock
// ============================================================================

pub struct MockSubscriptionService {
    pub config: MockConfig,
    pub subscriptions: Arc<std::sync::Mutex<Vec<Subscription>>>,
    pub create_calls: Arc<std::sync::Mutex<Vec<(Uuid, crate::subscriptions::models::CreateSubscriptionRequest)>>>,
}

impl Default for MockSubscriptionService {
    fn default() -> Self {
        Self::success()
    }
}

impl MockSubscriptionService {
    pub fn new(config: MockConfig) -> Self {
        Self {
            config,
            subscriptions: Arc::new(std::sync::Mutex::new(Vec::new())),
            create_calls: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn success() -> Self {
        Self::new(MockConfig::success())
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self::new(MockConfig::failure(message))
    }

    pub fn with_subscriptions(subscriptions: Vec<Subscription>) -> Self {
        let mock = Self::success();
        *mock.subscriptions.lock().unwrap() = subscriptions;
        mock
    }

    pub fn add_subscription(&self, subscription: Subscription) {
        self.subscriptions.lock().unwrap().push(subscription);
    }
}

#[async_trait]
impl SubscriptionServiceTrait for MockSubscriptionService {
    async fn create(
        &self,
        user_id: Uuid,
        req: crate::subscriptions::models::CreateSubscriptionRequest,
    ) -> AppResult<Subscription> {
        self.create_calls.lock().unwrap().push((user_id, req.clone()));

        if self.config.should_fail {
            return Err(AppError::BadRequest(self.config.error_message.clone()));
        }

        let subscription = Subscription {
            id: Uuid::new_v4(),
            user_id,
            category_id: req.category_id.unwrap_or(Uuid::nil()),
            name: req.name,
            amount: req.amount,
            currency: req.currency,
            frequency: req.frequency,
            start_date: req.start_date,
            next_billing_date: Some(req.start_date),
            is_active: true,
            notes: req.notes,
            logo_url: req.logo_url,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.subscriptions.lock().unwrap().push(subscription.clone());
        Ok(subscription)
    }

    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<Subscription>> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let subs = self.subscriptions.lock().unwrap();
        Ok(subs.iter().filter(|s| s.user_id == user_id).cloned().collect())
    }

    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<Subscription> {
        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let subs = self.subscriptions.lock().unwrap();
        subs.iter()
            .find(|s| s.id == id && s.user_id == user_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("Subscription not found".to_string()))
    }

    async fn update(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: crate::subscriptions::models::UpdateSubscriptionRequest,
    ) -> AppResult<Subscription> {
        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let mut subs = self.subscriptions.lock().unwrap();
        let sub = subs
            .iter_mut()
            .find(|s| s.id == id && s.user_id == user_id)
            .ok_or_else(|| AppError::NotFound("Subscription not found".to_string()))?;

        if let Some(name) = req.name {
            sub.name = name;
        }
        if let Some(amount) = req.amount {
            sub.amount = amount;
        }
        if let Some(active) = req.is_active {
            sub.is_active = active;
        }
        sub.updated_at = Utc::now();

        Ok(sub.clone())
    }

    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<()> {
        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let mut subs = self.subscriptions.lock().unwrap();
        let existed = subs.iter().any(|s| s.id == id && s.user_id == user_id);
        subs.retain(|s| !(s.id == id && s.user_id == user_id));

        if !existed {
            return Err(AppError::NotFound("Subscription not found".to_string()));
        }
        Ok(())
    }
}

// Subscription Service Trait
#[async_trait]
pub trait SubscriptionServiceTrait: Send + Sync {
    async fn create(
        &self,
        user_id: Uuid,
        req: crate::subscriptions::models::CreateSubscriptionRequest,
    ) -> AppResult<Subscription>;
    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<Subscription>>;
    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<Subscription>;
    async fn update(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: crate::subscriptions::models::UpdateSubscriptionRequest,
    ) -> AppResult<Subscription>;
    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<()>;
}

// ============================================================================
// Transaction Service Mock
// ============================================================================

pub struct MockTransactionService {
    pub config: MockConfig,
    pub transactions: Arc<std::sync::Mutex<Vec<Transaction>>>,
    pub create_calls: Arc<std::sync::Mutex<Vec<(Uuid, crate::transactions::models::CreateTransactionRequest)>>>,
}

impl Default for MockTransactionService {
    fn default() -> Self {
        Self::success()
    }
}

impl MockTransactionService {
    pub fn new(config: MockConfig) -> Self {
        Self {
            config,
            transactions: Arc::new(std::sync::Mutex::new(Vec::new())),
            create_calls: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn success() -> Self {
        Self::new(MockConfig::success())
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self::new(MockConfig::failure(message))
    }

    pub fn with_transactions(transactions: Vec<Transaction>) -> Self {
        let mock = Self::success();
        *mock.transactions.lock().unwrap() = transactions;
        mock
    }

    pub fn add_transaction(&self, transaction: Transaction) {
        self.transactions.lock().unwrap().push(transaction);
    }
}

#[async_trait]
impl TransactionServiceTrait for MockTransactionService {
    async fn create(
        &self,
        user_id: Uuid,
        req: crate::transactions::models::CreateTransactionRequest,
    ) -> AppResult<Transaction> {
        self.create_calls.lock().unwrap().push((user_id, req.clone()));

        if self.config.should_fail {
            return Err(AppError::BadRequest(self.config.error_message.clone()));
        }

        let transaction = Transaction {
            id: Uuid::new_v4(),
            user_id,
            category_id: req.category_id.unwrap_or(Uuid::nil()),
            name: req.name,
            amount: req.amount,
            currency: req.currency,
            date: req.date,
            notes: req.notes,
            logo_url: req.logo_url,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.transactions.lock().unwrap().push(transaction.clone());
        Ok(transaction)
    }

    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<Transaction>> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let trans = self.transactions.lock().unwrap();
        Ok(trans.iter().filter(|t| t.user_id == user_id).cloned().collect())
    }

    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<Transaction> {
        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let trans = self.transactions.lock().unwrap();
        trans.iter()
            .find(|t| t.id == id && t.user_id == user_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("Transaction not found".to_string()))
    }

    async fn update(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: crate::transactions::models::UpdateTransactionRequest,
    ) -> AppResult<Transaction> {
        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let mut trans = self.transactions.lock().unwrap();
        let t = trans
            .iter_mut()
            .find(|t| t.id == id && t.user_id == user_id)
            .ok_or_else(|| AppError::NotFound("Transaction not found".to_string()))?;

        if let Some(name) = req.name {
            t.name = name;
        }
        if let Some(amount) = req.amount {
            t.amount = amount;
        }
        t.updated_at = Utc::now();

        Ok(t.clone())
    }

    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<()> {
        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let mut trans = self.transactions.lock().unwrap();
        let existed = trans.iter().any(|t| t.id == id && t.user_id == user_id);
        trans.retain(|t| !(t.id == id && t.user_id == user_id));

        if !existed {
            return Err(AppError::NotFound("Transaction not found".to_string()));
        }
        Ok(())
    }
}

// Transaction Service Trait
#[async_trait]
pub trait TransactionServiceTrait: Send + Sync {
    async fn create(
        &self,
        user_id: Uuid,
        req: crate::transactions::models::CreateTransactionRequest,
    ) -> AppResult<Transaction>;
    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<Transaction>>;
    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<Transaction>;
    async fn update(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: crate::transactions::models::UpdateTransactionRequest,
    ) -> AppResult<Transaction>;
    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<()>;
}

// ============================================================================
// Notification Service Mock
// ============================================================================

pub struct MockNotificationService {
    pub config: MockConfig,
    pub notifications: Arc<std::sync::Mutex<Vec<Notification>>>,
}

impl Default for MockNotificationService {
    fn default() -> Self {
        Self::success()
    }
}

impl MockNotificationService {
    pub fn new(config: MockConfig) -> Self {
        Self {
            config,
            notifications: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn success() -> Self {
        Self::new(MockConfig::success())
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self::new(MockConfig::failure(message))
    }

    pub fn with_notifications(notifications: Vec<Notification>) -> Self {
        let mock = Self::success();
        *mock.notifications.lock().unwrap() = notifications;
        mock
    }

    pub fn add_notification(&self, notification: Notification) {
        self.notifications.lock().unwrap().push(notification);
    }
}

#[async_trait]
impl NotificationServiceTrait for MockNotificationService {
    async fn get_notifications(&self, user_id: Uuid) -> AppResult<crate::notifications::models::NotificationResponse> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let notifs = self.notifications.lock().unwrap();
        let user_notifs: Vec<_> = notifs.iter().filter(|n| n.user_id == user_id).cloned().collect();
        let unread_count = user_notifs.iter().filter(|n| !n.is_read).count() as i64;

        Ok(crate::notifications::models::NotificationResponse {
            notifications: user_notifs,
            unread_count,
        })
    }

    async fn mark_as_read(&self, user_id: Uuid, id: Uuid) -> AppResult<()> {
        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let mut notifs = self.notifications.lock().unwrap();
        let notif = notifs
            .iter_mut()
            .find(|n| n.id == id && n.user_id == user_id)
            .ok_or_else(|| AppError::NotFound("Notification not found".to_string()))?;

        notif.is_read = true;
        Ok(())
    }

    async fn mark_all_read(&self, user_id: Uuid) -> AppResult<()> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let mut notifs = self.notifications.lock().unwrap();
        for n in notifs.iter_mut() {
            if n.user_id == user_id {
                n.is_read = true;
            }
        }
        Ok(())
    }

    async fn send_notification(
        &self,
        user_id: Uuid,
        notification_type: String,
        title: String,
        message: Option<String>,
    ) -> AppResult<Notification> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let notification = Notification {
            id: Uuid::new_v4(),
            user_id,
            notification_type,
            title,
            message,
            is_read: false,
            created_at: Utc::now(),
        };

        self.notifications.lock().unwrap().push(notification.clone());
        Ok(notification)
    }
}

// Notification Service Trait
#[async_trait]
pub trait NotificationServiceTrait: Send + Sync {
    async fn get_notifications(&self, user_id: Uuid) -> AppResult<crate::notifications::models::NotificationResponse>;
    async fn mark_as_read(&self, user_id: Uuid, id: Uuid) -> AppResult<()>;
    async fn mark_all_read(&self, user_id: Uuid) -> AppResult<()>;
    async fn send_notification(
        &self,
        user_id: Uuid,
        notification_type: String,
        title: String,
        message: Option<String>,
    ) -> AppResult<Notification>;
}

// ============================================================================
// Report Service Mock
// ============================================================================

pub struct MockReportService {
    pub config: MockConfig,
}

impl Default for MockReportService {
    fn default() -> Self {
        Self::success()
    }
}

impl MockReportService {
    pub fn new(config: MockConfig) -> Self {
        Self { config }
    }

    pub fn success() -> Self {
        Self::new(MockConfig::success())
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self::new(MockConfig::failure(message))
    }
}

#[async_trait]
impl ReportServiceTrait for MockReportService {
    async fn get_summary(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<ReportResponse> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        Ok(ReportResponse {
            monthly_summary: MonthlySummary {
                year,
                month,
                subscriptions_total: Decimal::from_str("500.00").unwrap(),
                transactions_total: Decimal::from_str("200.00").unwrap(),
                overall_total: Decimal::from_str("700.00").unwrap(),
                subscriptions_count: 5,
                transactions_count: 10,
            },
            category_breakdown: vec![
                CategoryBreakdown {
                    category_id: Some("cat-1".to_string()),
                    category_name: "Streaming".to_string(),
                    total: Decimal::from_str("500.00").unwrap(),
                    count: 5,
                    percentage: 71.4,
                },
            ],
            recent_trends: vec![
                SpendingTrend {
                    period: format!("{}/{}", month - 1, year),
                    total: Decimal::from_str("650.00").unwrap(),
                },
            ],
        })
    }
}

// Report Service Trait
#[async_trait]
pub trait ReportServiceTrait: Send + Sync {
    async fn get_summary(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<ReportResponse>;
}

// ============================================================================
// Preferences Service Mock
// ============================================================================

pub struct MockPreferencesService {
    pub config: MockConfig,
    pub preferences: Arc<std::sync::Mutex<std::collections::HashMap<Uuid, PreferencesResponse>>>,
}

impl Default for MockPreferencesService {
    fn default() -> Self {
        Self::success()
    }
}

impl MockPreferencesService {
    pub fn new(config: MockConfig) -> Self {
        Self {
            config,
            preferences: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn success() -> Self {
        Self::new(MockConfig::success())
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self::new(MockConfig::failure(message))
    }
}

#[async_trait]
impl PreferencesServiceTrait for MockPreferencesService {
    async fn get_preferences(&self, user_id: Uuid) -> AppResult<PreferencesResponse> {
        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let prefs = self.preferences.lock().unwrap();
        Ok(prefs.get(&user_id).cloned().unwrap_or_default())
    }

    async fn update_preferences(
        &self,
        user_id: Uuid,
        req: UpdatePreferencesRequest,
    ) -> AppResult<PreferencesResponse> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let mut prefs = self.preferences.lock().unwrap();
        let current = prefs.get(&user_id).cloned().unwrap_or_default();

        let mut updated = current;
        if let Some(theme) = req.theme {
            updated.theme = theme;
        }
        if let Some(view) = req.subscriptions_view {
            updated.subscriptions_view = view;
        }

        prefs.insert(user_id, updated.clone());
        Ok(updated)
    }
}

// Preferences Service Trait
#[async_trait]
pub trait PreferencesServiceTrait: Send + Sync {
    async fn get_preferences(&self, user_id: Uuid) -> AppResult<PreferencesResponse>;
    async fn update_preferences(
        &self,
        user_id: Uuid,
        req: UpdatePreferencesRequest,
    ) -> AppResult<PreferencesResponse>;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── MockConfig Tests ──────────────────────────────────────────────────

    mod mock_config {
        use super::*;

        #[test]
        fn success_has_no_failure() {
            let config = MockConfig::success();
            assert!(!config.should_fail);
            assert!(config.error_message.is_empty());
        }

        #[test]
        fn failure_has_error() {
            let config = MockConfig::failure("test error");
            assert!(config.should_fail);
            assert_eq!(config.error_message, "test error");
        }

        #[test]
        fn delayed_config() {
            let config = MockConfig::delayed(100);
            assert!(!config.should_fail);
            assert_eq!(config.delay_ms, 100);
        }

        #[test]
        fn default_config_has_no_failure() {
            let config = MockConfig::default();
            assert!(!config.should_fail);
        }
    }

    // ── MockAuthService Tests ─────────────────────────────────────────────

    mod mock_auth_service {
        use super::*;
        use crate::auth::models::CreateUserRequest;

        fn make_register_request(email: &str) -> CreateUserRequest {
            CreateUserRequest {
                email: email.to_string(),
                password: "password123".to_string(),
                name: "Test User".to_string(),
            }
        }

        #[tokio::test]
        async fn register_success() {
            let mock = MockAuthService::success();
            let req = make_register_request("test@example.com");
            let result = mock.register(req).await;
            assert!(result.is_ok());
            let resp = result.unwrap();
            assert_eq!(resp.user.email, "test@example.com");
            assert!(resp.requires_verification); // Users start unverified in mock
        }

        #[tokio::test]
        async fn register_returns_token() {
            let mock = MockAuthService::success();
            let req = make_register_request("token@test.com");
            let result = mock.register(req).await.unwrap();
            assert!(!result.token.is_empty());
            assert!(result.token.starts_with("mock_token_"));
        }

        #[tokio::test]
        async fn register_duplicate_email_fails() {
            let mock = MockAuthService::success();
            let req = make_register_request("dup@test.com");

            mock.register(req.clone()).await.unwrap();
            let result = mock.register(req).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::Conflict(_)));
        }

        #[tokio::test]
        async fn register_failure_returns_error() {
            let mock = MockAuthService::failure("Registration failed");
            let req = make_register_request("fail@test.com");
            let result = mock.register(req).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn login_success() {
            let mock = MockAuthService::success();
            let req = make_register_request("login@test.com");
            mock.register(req.clone()).await.unwrap();

            let login_req = LoginRequest {
                email: "login@test.com".to_string(),
                password: "password123".to_string(),
            };
            let result = mock.login(login_req).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn login_unknown_user_fails() {
            let mock = MockAuthService::success();
            let login_req = LoginRequest {
                email: "nobody@test.com".to_string(),
                password: "password123".to_string(),
            };
            let result = mock.login(login_req).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::Unauthorized(_)));
        }

        #[tokio::test]
        async fn login_failure_returns_error() {
            let mock = MockAuthService::failure("Login disabled");
            let login_req = LoginRequest {
                email: "test@test.com".to_string(),
                password: "pass".to_string(),
            };
            let result = mock.login(login_req).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn verify_email_success() {
            let mock = MockAuthService::success();
            let req = make_register_request("verify@test.com");
            let user_id = mock.register(req).await.unwrap().user.id;

            let result = mock.verify_email(user_id, "123456".to_string()).await;
            assert!(result.is_ok());
            assert!(result.unwrap().verified);
        }

        #[tokio::test]
        async fn verify_email_invalid_user() {
            let mock = MockAuthService::success();
            let result = mock.verify_email(Uuid::new_v4(), "123456".to_string()).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
        }

        #[tokio::test]
        async fn verify_email_wrong_code() {
            let mock = MockAuthService::success();
            let req = make_register_request("code@test.com");
            let user_id = mock.register(req).await.unwrap().user.id;

            let result = mock.verify_email(user_id, "999999".to_string()).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::BadRequest(_)));
        }

        #[tokio::test]
        async fn verify_token_valid() {
            let mock = MockAuthService::success();
            let req = make_register_request("token_valid@test.com");
            let token = mock.register(req).await.unwrap().token;

            let result = mock.verify_token(&token).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn verify_token_invalid_format() {
            let mock = MockAuthService::success();
            let result = mock.verify_token("invalid_token").await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn verify_token_failure() {
            let mock = MockAuthService::failure("Token verification failed");
            let result = mock.verify_token("mock_token_invalid").await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_user_by_id_success() {
            let mock = MockAuthService::success();
            let req = make_register_request("getuser@test.com");
            let user_id = mock.register(req).await.unwrap().user.id;

            let result = mock.get_user_by_id(user_id).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().email, "getuser@test.com");
        }

        #[tokio::test]
        async fn get_user_by_id_not_found() {
            let mock = MockAuthService::success();
            let result = mock.get_user_by_id(Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn update_settings_success() {
            let mock = MockAuthService::success();
            let req = make_register_request("update@test.com");
            let user_id = mock.register(req).await.unwrap().user.id;

            let update_req = UpdateSettingsRequest {
                name: Some("New Name".to_string()),
                currency: Some("EUR".to_string()),
            };
            let result = mock.update_settings(user_id, update_req).await;
            assert!(result.is_ok());
            let user = result.unwrap();
            assert_eq!(user.name, Some("New Name".to_string()));
            assert_eq!(user.currency, "EUR");
        }

        #[tokio::test]
        async fn resend_verification_success() {
            let mock = MockAuthService::success();
            let req = make_register_request("resend@test.com");
            let user_id = mock.register(req).await.unwrap().user.id;

            let result = mock.resend_verification(user_id).await;
            assert!(result.is_ok());
            assert!(!result.unwrap().is_empty());
        }

        #[tokio::test]
        async fn resend_verification_already_verified() {
            let mock = MockAuthService::success();
            let req = make_register_request("already@test.com");
            let user_id = mock.register(req).await.unwrap().user.id;
            mock.verify_email(user_id, "123456".to_string()).await.unwrap();

            let result = mock.resend_verification(user_id).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_settings_returns_user_response() {
            let mock = MockAuthService::success();
            let req = make_register_request("settings@test.com");
            let user_id = mock.register(req).await.unwrap().user.id;

            let result = mock.get_settings(user_id).await;
            assert!(result.is_ok());
            let settings = result.unwrap();
            assert_eq!(settings.email, "settings@test.com");
        }
    }

    // ── MockCategoryService Tests ─────────────────────────────────────────

    mod mock_category_service {
        use super::*;

        fn make_category_request(name: &str) -> CreateCategoryRequest {
            CreateCategoryRequest {
                name: name.to_string(),
                icon: Some("star".to_string()),
                color: Some("#ff0000".to_string()),
            }
        }

        #[tokio::test]
        async fn create_success() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            let req = make_category_request("Test Category");

            let result = mock.create(user_id, req).await;
            assert!(result.is_ok());
            let cat = result.unwrap();
            assert_eq!(cat.name, "Test Category");
            assert!(cat.is_default); // First category becomes default
        }

        #[tokio::test]
        async fn create_first_becomes_default() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            let req = make_category_request("First");

            let result = mock.create(user_id, req).await;
            assert!(result.unwrap().is_default);
        }

        #[tokio::test]
        async fn create_failure() {
            let mock = MockCategoryService::failure("Category creation failed");
            let req = make_category_request("Fail");
            let result = mock.create(Uuid::new_v4(), req).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_all_returns_user_categories() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();

            mock.create(user_id, make_category_request("Cat 1")).await.unwrap();
            mock.create(user_id, make_category_request("Cat 2")).await.unwrap();

            let result = mock.get_all(user_id).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 2);
        }

        #[tokio::test]
        async fn get_all_filters_by_user() {
            let mock = MockCategoryService::success();
            let user1 = Uuid::new_v4();
            let user2 = Uuid::new_v4();

            mock.create(user1, make_category_request("U1 Cat")).await.unwrap();
            mock.create(user2, make_category_request("U2 Cat")).await.unwrap();

            let result = mock.get_all(user1).await.unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].name, "U1 Cat");
        }

        #[tokio::test]
        async fn get_by_id_success() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            let cat = mock.create(user_id, make_category_request("Find Me")).await.unwrap();

            let result = mock.get_by_id(user_id, cat.id).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().name, "Find Me");
        }

        #[tokio::test]
        async fn get_by_id_not_found() {
            let mock = MockCategoryService::success();
            let result = mock.get_by_id(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn update_success() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            let cat = mock.create(user_id, make_category_request("Original")).await.unwrap();

            let req = UpdateCategoryRequest {
                name: Some("Updated".to_string()),
                icon: serde_json::Value::Null,
                color: serde_json::Value::Null,
            };
            let result = mock.update(user_id, cat.id, req).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().name, "Updated");
        }

        #[tokio::test]
        async fn delete_success() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            
            // Create default category first
            mock.create(user_id, make_category_request("Default")).await.unwrap();
            // Create non-default category to delete
            let cat = mock.create(user_id, make_category_request("To Delete")).await.unwrap();
            assert!(!cat.is_default); // Second category is not default

            let result = mock.delete(user_id, cat.id).await;
            assert!(result.is_ok());

            // Verify it's gone
            let get_result = mock.get_by_id(user_id, cat.id).await;
            assert!(get_result.is_err());
        }

        #[tokio::test]
        async fn delete_default_fails() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            let cat = mock.create(user_id, make_category_request("Default")).await.unwrap();

            let result = mock.delete(user_id, cat.id).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::BadRequest(_)));
        }

        #[tokio::test]
        async fn set_default_success() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();

            let cat1 = mock.create(user_id, make_category_request("First")).await.unwrap();
            let cat2 = mock.create(user_id, make_category_request("Second")).await.unwrap();

            let result = mock.set_default(user_id, cat2.id).await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_default);

            // First should no longer be default
            let first = mock.get_by_id(user_id, cat1.id).await.unwrap();
            assert!(!first.is_default);
        }

        #[tokio::test]
        async fn set_default_not_found() {
            let mock = MockCategoryService::success();
            let result = mock.set_default(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }
    }

    // ── MockSubscriptionService Tests ─────────────────────────────────────

    mod mock_subscription_service {
        use super::*;
        use crate::subscriptions::models::CreateSubscriptionRequest;

        fn make_subscription_request(name: &str) -> CreateSubscriptionRequest {
            CreateSubscriptionRequest {
                name: name.to_string(),
                amount: Decimal::from_str("9.99").unwrap(),
                currency: "USD".to_string(),
                frequency: "monthly".to_string(),
                start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                category_id: Some(Uuid::new_v4()),
                notes: None,
                logo_url: None,
            }
        }

        #[tokio::test]
        async fn create_success() {
            let mock = MockSubscriptionService::success();
            let user_id = Uuid::new_v4();
            let req = make_subscription_request("Netflix");

            let result = mock.create(user_id, req).await;
            assert!(result.is_ok());
            let sub = result.unwrap();
            assert_eq!(sub.name, "Netflix");
            assert!(sub.is_active);
        }

        #[tokio::test]
        async fn create_failure() {
            let mock = MockSubscriptionService::failure("Subscription creation failed");
            let req = make_subscription_request("Fail");
            let result = mock.create(Uuid::new_v4(), req).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_all_returns_user_subscriptions() {
            let mock = MockSubscriptionService::success();
            let user_id = Uuid::new_v4();

            mock.create(user_id, make_subscription_request("Sub 1")).await.unwrap();
            mock.create(user_id, make_subscription_request("Sub 2")).await.unwrap();

            let result = mock.get_all(user_id).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 2);
        }

        #[tokio::test]
        async fn get_by_id_success() {
            let mock = MockSubscriptionService::success();
            let user_id = Uuid::new_v4();
            let sub = mock.create(user_id, make_subscription_request("Find Me")).await.unwrap();

            let result = mock.get_by_id(user_id, sub.id).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().name, "Find Me");
        }

        #[tokio::test]
        async fn get_by_id_not_found() {
            let mock = MockSubscriptionService::success();
            let result = mock.get_by_id(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn delete_success() {
            let mock = MockSubscriptionService::success();
            let user_id = Uuid::new_v4();
            let sub = mock.create(user_id, make_subscription_request("To Delete")).await.unwrap();

            let result = mock.delete(user_id, sub.id).await;
            assert!(result.is_ok());

            let get_result = mock.get_by_id(user_id, sub.id).await;
            assert!(get_result.is_err());
        }

        #[tokio::test]
        async fn delete_not_found() {
            let mock = MockSubscriptionService::success();
            let result = mock.delete(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }
    }

    // ── MockTransactionService Tests ─────────────────────────────────────

    mod mock_transaction_service {
        use super::*;
        use crate::transactions::models::CreateTransactionRequest;

        fn make_transaction_request(name: &str) -> CreateTransactionRequest {
            CreateTransactionRequest {
                name: name.to_string(),
                amount: Decimal::from_str("25.00").unwrap(),
                currency: "USD".to_string(),
                date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
                category_id: Some(Uuid::new_v4()),
                notes: Some("Test transaction".to_string()),
                logo_url: None,
            }
        }

        #[tokio::test]
        async fn create_success() {
            let mock = MockTransactionService::success();
            let user_id = Uuid::new_v4();
            let req = make_transaction_request("Coffee");

            let result = mock.create(user_id, req).await;
            assert!(result.is_ok());
            let trans = result.unwrap();
            assert_eq!(trans.name, "Coffee");
        }

        #[tokio::test]
        async fn create_failure() {
            let mock = MockTransactionService::failure("Transaction creation failed");
            let req = make_transaction_request("Fail");
            let result = mock.create(Uuid::new_v4(), req).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_all_returns_user_transactions() {
            let mock = MockTransactionService::success();
            let user_id = Uuid::new_v4();

            mock.create(user_id, make_transaction_request("Trans 1")).await.unwrap();
            mock.create(user_id, make_transaction_request("Trans 2")).await.unwrap();

            let result = mock.get_all(user_id).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 2);
        }

        #[tokio::test]
        async fn get_by_id_success() {
            let mock = MockTransactionService::success();
            let user_id = Uuid::new_v4();
            let trans = mock.create(user_id, make_transaction_request("Find Me")).await.unwrap();

            let result = mock.get_by_id(user_id, trans.id).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().name, "Find Me");
        }

        #[tokio::test]
        async fn delete_success() {
            let mock = MockTransactionService::success();
            let user_id = Uuid::new_v4();
            let trans = mock.create(user_id, make_transaction_request("To Delete")).await.unwrap();

            let result = mock.delete(user_id, trans.id).await;
            assert!(result.is_ok());

            let get_result = mock.get_by_id(user_id, trans.id).await;
            assert!(get_result.is_err());
        }
    }

    // ── MockNotificationService Tests ─────────────────────────────────────

    mod mock_notification_service {
        use super::*;

        fn make_notification(user_id: Uuid, title: &str) -> Notification {
            Notification {
                id: Uuid::new_v4(),
                user_id,
                notification_type: "reminder".to_string(),
                title: title.to_string(),
                message: Some("Test message".to_string()),
                is_read: false,
                created_at: Utc::now(),
            }
        }

        #[tokio::test]
        async fn get_notifications_success() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();
            mock.add_notification(make_notification(user_id, "Test"));
            mock.add_notification(make_notification(user_id, "Test 2"));

            let result = mock.get_notifications(user_id).await;
            assert!(result.is_ok());
            let resp = result.unwrap();
            assert_eq!(resp.notifications.len(), 2);
            assert_eq!(resp.unread_count, 2);
        }

        #[tokio::test]
        async fn get_notifications_empty() {
            let mock = MockNotificationService::success();
            let result = mock.get_notifications(Uuid::new_v4()).await;
            assert!(result.is_ok());
            assert!(result.unwrap().notifications.is_empty());
        }

        #[tokio::test]
        async fn get_notifications_failure() {
            let mock = MockNotificationService::failure("Database error");
            let result = mock.get_notifications(Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn mark_as_read_success() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();
            let notif = make_notification(user_id, "To Mark");
            let notif_id = notif.id;
            mock.add_notification(notif);

            let result = mock.mark_as_read(user_id, notif_id).await;
            assert!(result.is_ok());

            let resp = mock.get_notifications(user_id).await.unwrap();
            let marked = resp.notifications.iter().find(|n| n.id == notif_id).unwrap();
            assert!(marked.is_read);
        }

        #[tokio::test]
        async fn mark_as_read_not_found() {
            let mock = MockNotificationService::success();
            let result = mock.mark_as_read(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn mark_all_read_success() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();
            mock.add_notification(make_notification(user_id, "All 1"));
            mock.add_notification(make_notification(user_id, "All 2"));

            let result = mock.mark_all_read(user_id).await;
            assert!(result.is_ok());

            let resp = mock.get_notifications(user_id).await.unwrap();
            assert_eq!(resp.unread_count, 0);
        }

        #[tokio::test]
        async fn send_notification_creates_notification() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();

            let result = mock
                .send_notification(
                    user_id,
                    "billing".to_string(),
                    "Payment Due".to_string(),
                    Some("Your payment is due soon".to_string()),
                )
                .await;
            assert!(result.is_ok());
            let notif = result.unwrap();
            assert_eq!(notif.title, "Payment Due");
            assert!(!notif.is_read);
        }

        #[tokio::test]
        async fn send_notification_failure() {
            let mock = MockNotificationService::failure("Notification service down");
            let result = mock
                .send_notification(
                    Uuid::new_v4(),
                    "test".to_string(),
                    "Title".to_string(),
                    None,
                )
                .await;
            assert!(result.is_err());
        }
    }

    // ── MockReportService Tests ─────────────────────────────────────────────

    mod mock_report_service {
        use super::*;

        #[tokio::test]
        async fn get_summary_success() {
            let mock = MockReportService::success();
            let user_id = Uuid::new_v4();

            let result = mock.get_summary(user_id, 2024, 6).await;
            assert!(result.is_ok());
            let report = result.unwrap();

            assert_eq!(report.monthly_summary.year, 2024);
            assert_eq!(report.monthly_summary.month, 6);
            assert_eq!(report.monthly_summary.subscriptions_total, Decimal::from_str("500.00").unwrap());
            assert_eq!(report.monthly_summary.transactions_total, Decimal::from_str("200.00").unwrap());
        }

        #[tokio::test]
        async fn get_summary_failure() {
            let mock = MockReportService::failure("Report generation failed");
            let result = mock.get_summary(Uuid::new_v4(), 2024, 6).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_summary_has_category_breakdown() {
            let mock = MockReportService::success();
            let result = mock.get_summary(Uuid::new_v4(), 2024, 1).await.unwrap();
            assert!(!result.category_breakdown.is_empty());
            assert_eq!(result.category_breakdown[0].category_name, "Streaming");
        }

        #[tokio::test]
        async fn get_summary_has_recent_trends() {
            let mock = MockReportService::success();
            let result = mock.get_summary(Uuid::new_v4(), 2024, 2).await.unwrap();
            assert!(!result.recent_trends.is_empty());
        }
    }

    // ── MockPreferencesService Tests ──────────────────────────────────────

    mod mock_preferences_service {
        use super::*;

        #[tokio::test]
        async fn get_preferences_default() {
            let mock = MockPreferencesService::success();
            let user_id = Uuid::new_v4();

            let result = mock.get_preferences(user_id).await;
            assert!(result.is_ok());
            let prefs = result.unwrap();
            assert_eq!(prefs.theme, "system");
            assert_eq!(prefs.subscriptions_view, "grid");
        }

        #[tokio::test]
        async fn get_preferences_failure() {
            let mock = MockPreferencesService::failure("Preferences not found");
            let result = mock.get_preferences(Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn update_preferences_theme() {
            let mock = MockPreferencesService::success();
            let user_id = Uuid::new_v4();

            let req = UpdatePreferencesRequest {
                theme: Some("dark".to_string()),
                subscriptions_view: None,
            };
            let result = mock.update_preferences(user_id, req).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().theme, "dark");
        }

        #[tokio::test]
        async fn update_preferences_view() {
            let mock = MockPreferencesService::success();
            let user_id = Uuid::new_v4();

            let req = UpdatePreferencesRequest {
                theme: None,
                subscriptions_view: Some("list".to_string()),
            };
            let result = mock.update_preferences(user_id, req).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().subscriptions_view, "list");
        }

        #[tokio::test]
        async fn update_preferences_persists() {
            let mock = MockPreferencesService::success();
            let user_id = Uuid::new_v4();

            let req = UpdatePreferencesRequest {
                theme: Some("midnight".to_string()),
                subscriptions_view: Some("compact".to_string()),
            };
            mock.update_preferences(user_id, req).await.unwrap();

            let result = mock.get_preferences(user_id).await.unwrap();
            assert_eq!(result.theme, "midnight");
            assert_eq!(result.subscriptions_view, "compact");
        }

        #[tokio::test]
        async fn update_preferences_preserves_unchanged() {
            let mock = MockPreferencesService::success();
            let user_id = Uuid::new_v4();

            // Set initial
            let req1 = UpdatePreferencesRequest {
                theme: Some("dark".to_string()),
                subscriptions_view: Some("list".to_string()),
            };
            mock.update_preferences(user_id, req1).await.unwrap();

            // Update only theme
            let req2 = UpdatePreferencesRequest {
                theme: Some("light".to_string()),
                subscriptions_view: None,
            };
            let result = mock.update_preferences(user_id, req2).await.unwrap();

            assert_eq!(result.theme, "light");
            assert_eq!(result.subscriptions_view, "list"); // preserved
        }

        #[tokio::test]
        async fn update_preferences_failure() {
            let mock = MockPreferencesService::failure("Update failed");
            let req = UpdatePreferencesRequest {
                theme: Some("dark".to_string()),
                subscriptions_view: None,
            };
            let result = mock.update_preferences(Uuid::new_v4(), req).await;
            assert!(result.is_err());
        }
    }

    // ── Clone Derive Tests ─────────────────────────────────────────────────

    mod model_clone_derive {
        use super::*;
        use chrono::NaiveDate;

        #[test]
        fn user_clone_preserves_fields() {
            let user = User {
                id: Uuid::new_v4(),
                email: "clone@test.com".to_string(),
                password_hash: "hash".to_string(),
                name: Some("Clone".to_string()),
                currency: "USD".to_string(),
                verified: true,
                verification_code: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            let cloned = user.clone();
            assert_eq!(cloned.email, user.email);
            assert_eq!(cloned.name, user.name);
            assert_eq!(cloned.id, user.id);
        }

        #[test]
        fn category_clone_preserves_fields() {
            let cat = Category {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                name: "Clone Cat".to_string(),
                icon: Some("star".to_string()),
                color: Some("#000".to_string()),
                is_system: false,
                is_default: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            let cloned = cat.clone();
            assert_eq!(cloned.name, cat.name);
            assert_eq!(cloned.is_default, cat.is_default);
        }

        #[test]
        fn subscription_clone_preserves_fields() {
            let sub = Subscription {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                category_id: Uuid::new_v4(),
                name: "Clone Sub".to_string(),
                amount: Decimal::from_str("9.99").unwrap(),
                currency: "USD".to_string(),
                frequency: "monthly".to_string(),
                start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                next_billing_date: None,
                is_active: true,
                notes: None,
                logo_url: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            let cloned = sub.clone();
            assert_eq!(cloned.name, sub.name);
            assert_eq!(cloned.amount, sub.amount);
            assert_eq!(cloned.is_active, sub.is_active);
        }

        #[test]
        fn transaction_clone_preserves_fields() {
            let trans = Transaction {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                category_id: Uuid::new_v4(),
                name: "Clone Trans".to_string(),
                amount: Decimal::from_str("25.00").unwrap(),
                currency: "EUR".to_string(),
                date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
                notes: Some("Clone note".to_string()),
                logo_url: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            let cloned = trans.clone();
            assert_eq!(cloned.name, trans.name);
            assert_eq!(cloned.amount, trans.amount);
        }
    }
}