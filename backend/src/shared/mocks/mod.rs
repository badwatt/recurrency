#![allow(dead_code)]

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
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::models::{
    AuthResponse, Claims, CreateUserRequest, LoginRequest, UpdateSettingsRequest, User,
    UserResponse,
};
use crate::auth::service::AuthServiceTrait;
use crate::categories::models::{Category, CreateCategoryRequest, UpdateCategoryRequest};
use crate::categories::service::CategoryWithCounts;
use crate::notifications::models::{
    Notification, NotificationPreferenceResponse, NotificationResponse, UpdatePreferenceRequest,
};
use crate::preferences::models::{PreferencesResponse, UpdatePreferencesRequest};
use crate::reports::models::{CategoryBreakdown, MonthlySummary, ReportResponse, SpendingTrend};
use crate::shared::error::{AppError, AppResult};
use crate::subscriptions::models::{
    CreateSubscriptionRequest, Subscription, UpdateSubscriptionRequest,
};
use crate::subscriptions::service::CalendarItem;
use crate::transactions::models::{
    CreateTransactionRequest, Transaction, UpdateTransactionRequest,
};

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
// Auth Service Mock (trait defined in auth/service.rs)
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

    pub fn make_user(&self, id: Uuid, email: &str, name: &str, verified: bool) -> User {
        User {
            id,
            email: email.to_string(),
            password_hash: "mock_hash".to_string(),
            name: Some(name.to_string()),
            currency: "USD".to_string(),
            verified,
            verification_code: if verified {
                None
            } else {
                Some("123456".to_string())
            },
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
            return Err(AppError::BadRequest(
                "Invalid verification code".to_string(),
            ));
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
// Category Service Trait + Mock
// ============================================================================

#[async_trait]
pub trait CategoryServiceTrait: Send + Sync {
    async fn create_uncategorized(&self, user_id: Uuid) -> AppResult<()>;
    async fn create(&self, user_id: Uuid, req: CreateCategoryRequest) -> AppResult<Category>;
    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<Category>>;
    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<Category>;
    async fn update(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: UpdateCategoryRequest,
    ) -> AppResult<Category>;
    async fn get_default(&self, user_id: Uuid) -> AppResult<Category>;
    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<()>;
    async fn set_default(&self, user_id: Uuid, id: Uuid) -> AppResult<Category>;
    async fn get_with_counts(&self, user_id: Uuid) -> AppResult<Vec<CategoryWithCounts>>;
}

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

    pub fn make_category(&self, id: Uuid, user_id: Uuid, name: &str) -> Category {
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
    async fn create_uncategorized(&self, user_id: Uuid) -> AppResult<()> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let mut cats = self.categories.lock().unwrap();
        // Only create if user has no categories yet
        if !cats.iter().any(|c| c.user_id == user_id) {
            cats.push(Category {
                id: Uuid::new_v4(),
                user_id,
                name: "Uncategorized".to_string(),
                icon: None,
                color: None,
                is_system: true,
                is_default: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }
        Ok(())
    }

    async fn create(&self, user_id: Uuid, req: CreateCategoryRequest) -> AppResult<Category> {
        self.create_calls
            .lock()
            .unwrap()
            .push((user_id, req.clone()));

        if self.config.should_fail {
            return Err(AppError::BadRequest(self.config.error_message.clone()));
        }

        let mut cats = self.categories.lock().unwrap();
        let is_default = !cats.iter().any(|c| c.user_id == user_id);

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
        Ok(cats
            .iter()
            .filter(|c| c.user_id == user_id)
            .cloned()
            .collect())
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
        self.update_calls
            .lock()
            .unwrap()
            .push((user_id, id, req.clone()));

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

    async fn get_default(&self, user_id: Uuid) -> AppResult<Category> {
        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let cats = self.categories.lock().unwrap();
        cats.iter()
            .find(|c| c.user_id == user_id && c.is_default)
            .cloned()
            .ok_or_else(|| AppError::NotFound("Default category not found".to_string()))
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
            return Err(AppError::BadRequest(
                "Cannot delete the default category".to_string(),
            ));
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

    async fn get_with_counts(&self, user_id: Uuid) -> AppResult<Vec<CategoryWithCounts>> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let cats = self.categories.lock().unwrap();
        let user_cats: Vec<_> = cats
            .iter()
            .filter(|c| c.user_id == user_id)
            .cloned()
            .collect();
        Ok(user_cats
            .into_iter()
            .map(|c| CategoryWithCounts {
                category: c.into(),
                subscription_count: 0,
                transaction_count: 0,
            })
            .collect())
    }
}

// ============================================================================
// Subscription Service Trait + Mock
// ============================================================================

#[async_trait]
pub trait SubscriptionServiceTrait: Send + Sync {
    async fn create(
        &self,
        user_id: Uuid,
        req: CreateSubscriptionRequest,
    ) -> AppResult<Subscription>;
    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<Subscription>>;
    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<Subscription>;
    async fn update(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: UpdateSubscriptionRequest,
    ) -> AppResult<Subscription>;
    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<()>;
    async fn get_upcoming(&self, user_id: Uuid, days: i64) -> AppResult<Vec<Subscription>>;
    async fn get_calendar(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<Vec<CalendarItem>>;
}

pub struct MockSubscriptionService {
    pub config: MockConfig,
    pub subscriptions: Arc<std::sync::Mutex<Vec<Subscription>>>,
    pub create_calls: Arc<std::sync::Mutex<Vec<(Uuid, CreateSubscriptionRequest)>>>,
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
        req: CreateSubscriptionRequest,
    ) -> AppResult<Subscription> {
        self.create_calls
            .lock()
            .unwrap()
            .push((user_id, req.clone()));

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

        self.subscriptions
            .lock()
            .unwrap()
            .push(subscription.clone());
        Ok(subscription)
    }

    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<Subscription>> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let subs = self.subscriptions.lock().unwrap();
        Ok(subs
            .iter()
            .filter(|s| s.user_id == user_id)
            .cloned()
            .collect())
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
        req: UpdateSubscriptionRequest,
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
        if let Some(currency) = req.currency {
            sub.currency = currency;
        }
        if let Some(frequency) = req.frequency {
            sub.frequency = frequency;
        }
        if let Some(start_date) = req.start_date {
            sub.start_date = start_date;
        }
        if let Some(next_billing) = req.next_billing_date {
            sub.next_billing_date = Some(next_billing);
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

    async fn get_upcoming(&self, user_id: Uuid, _days: i64) -> AppResult<Vec<Subscription>> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let subs = self.subscriptions.lock().unwrap();
        Ok(subs
            .iter()
            .filter(|s| s.user_id == user_id && s.is_active)
            .cloned()
            .collect())
    }

    async fn get_calendar(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<Vec<CalendarItem>> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let subs = self.subscriptions.lock().unwrap();
        Ok(subs
            .iter()
            .filter(|s| s.user_id == user_id && s.is_active)
            .map(|s| CalendarItem {
                subscription_id: s.id,
                name: s.name.clone(),
                amount: s.amount,
                currency: s.currency.clone(),
                frequency: s.frequency.clone(),
                billing_date: NaiveDate::from_ymd_opt(year, month, 15)
                    .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, 1, 1).unwrap()),
                is_active: s.is_active,
                logo_url: s.logo_url.clone(),
            })
            .collect())
    }
}

// ============================================================================
// Transaction Service Trait + Mock
// ============================================================================

#[async_trait]
pub trait TransactionServiceTrait: Send + Sync {
    async fn create(&self, user_id: Uuid, req: CreateTransactionRequest) -> AppResult<Transaction>;
    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<Transaction>>;
    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<Transaction>;
    async fn update(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: UpdateTransactionRequest,
    ) -> AppResult<Transaction>;
    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<()>;
    async fn get_by_date_range(
        &self,
        user_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> AppResult<Vec<Transaction>>;
}

pub struct MockTransactionService {
    pub config: MockConfig,
    pub transactions: Arc<std::sync::Mutex<Vec<Transaction>>>,
    pub create_calls: Arc<std::sync::Mutex<Vec<(Uuid, CreateTransactionRequest)>>>,
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
    async fn create(&self, user_id: Uuid, req: CreateTransactionRequest) -> AppResult<Transaction> {
        self.create_calls
            .lock()
            .unwrap()
            .push((user_id, req.clone()));

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
        Ok(trans
            .iter()
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<Transaction> {
        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let trans = self.transactions.lock().unwrap();
        trans
            .iter()
            .find(|t| t.id == id && t.user_id == user_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("Transaction not found".to_string()))
    }

    async fn update(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: UpdateTransactionRequest,
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

    async fn get_by_date_range(
        &self,
        user_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> AppResult<Vec<Transaction>> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let trans = self.transactions.lock().unwrap();
        Ok(trans
            .iter()
            .filter(|t| t.user_id == user_id && t.date >= from && t.date <= to)
            .cloned()
            .collect())
    }
}

// ============================================================================
// Notification Service Trait + Mock
// ============================================================================

#[async_trait]
pub trait NotificationServiceTrait: Send + Sync {
    async fn list(&self, user_id: Uuid) -> AppResult<NotificationResponse>;
    async fn mark_read(&self, user_id: Uuid, id: Uuid) -> AppResult<bool>;
    async fn mark_all_read(&self, user_id: Uuid) -> AppResult<i64>;
    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<bool>;
    async fn get_notification_preferences(
        &self,
        user_id: Uuid,
    ) -> AppResult<Vec<NotificationPreferenceResponse>>;
    async fn update_notification_preferences(
        &self,
        user_id: Uuid,
        prefs: Vec<UpdatePreferenceRequest>,
    ) -> AppResult<Vec<NotificationPreferenceResponse>>;
}

pub struct MockNotificationService {
    pub config: MockConfig,
    pub notifications: Arc<std::sync::Mutex<Vec<Notification>>>,
    pub preferences: Arc<std::sync::Mutex<Vec<NotificationPreferenceResponse>>>,
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
            preferences: Arc::new(std::sync::Mutex::new(vec![
                NotificationPreferenceResponse {
                    channel: "email".to_string(),
                    enabled: false,
                    reminder_days_before: 3,
                },
                NotificationPreferenceResponse {
                    channel: "push".to_string(),
                    enabled: false,
                    reminder_days_before: 3,
                },
                NotificationPreferenceResponse {
                    channel: "system".to_string(),
                    enabled: false,
                    reminder_days_before: 3,
                },
            ])),
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
    async fn list(&self, user_id: Uuid) -> AppResult<NotificationResponse> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let notifs = self.notifications.lock().unwrap();
        let user_notifs: Vec<_> = notifs
            .iter()
            .filter(|n| n.user_id == user_id)
            .cloned()
            .collect();
        let unread_count = user_notifs.iter().filter(|n| !n.is_read).count() as i64;

        Ok(NotificationResponse {
            notifications: user_notifs,
            unread_count,
        })
    }

    async fn mark_read(&self, user_id: Uuid, id: Uuid) -> AppResult<bool> {
        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let mut notifs = self.notifications.lock().unwrap();
        let notif = notifs
            .iter_mut()
            .find(|n| n.id == id && n.user_id == user_id)
            .ok_or_else(|| AppError::NotFound("Notification not found".to_string()))?;

        notif.is_read = true;
        Ok(true)
    }

    async fn mark_all_read(&self, user_id: Uuid) -> AppResult<i64> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let mut count: i64 = 0;
        let mut notifs = self.notifications.lock().unwrap();
        for n in notifs.iter_mut() {
            if n.user_id == user_id && !n.is_read {
                n.is_read = true;
                count += 1;
            }
        }
        Ok(count)
    }

    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<bool> {
        if self.config.should_fail {
            return Err(AppError::NotFound(self.config.error_message.clone()));
        }

        let mut notifs = self.notifications.lock().unwrap();
        let existed = notifs.iter().any(|n| n.id == id && n.user_id == user_id);
        notifs.retain(|n| !(n.id == id && n.user_id == user_id));

        if !existed {
            return Err(AppError::NotFound("Notification not found".to_string()));
        }
        Ok(true)
    }

    async fn get_notification_preferences(
        &self,
        _user_id: Uuid,
    ) -> AppResult<Vec<NotificationPreferenceResponse>> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let prefs = self.preferences.lock().unwrap();
        Ok(prefs.clone())
    }

    async fn update_notification_preferences(
        &self,
        _user_id: Uuid,
        updates: Vec<UpdatePreferenceRequest>,
    ) -> AppResult<Vec<NotificationPreferenceResponse>> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        let mut prefs = self.preferences.lock().unwrap();
        for req in updates {
            if let Some(pref) = prefs.iter_mut().find(|p| p.channel == req.channel) {
                pref.enabled = req.enabled;
            }
        }
        Ok(prefs.clone())
    }
}

// ============================================================================
// Report Service Trait + Mock
// ============================================================================

#[async_trait]
pub trait ReportServiceTrait: Send + Sync {
    async fn get_monthly_summary(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<MonthlySummary>;
    async fn get_category_breakdown(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<Vec<CategoryBreakdown>>;
    async fn get_recent_trends(
        &self,
        user_id: Uuid,
        months_count: i32,
    ) -> AppResult<Vec<SpendingTrend>>;
    async fn get_full_report(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<ReportResponse>;
}

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

fn make_mock_summary(year: i32, month: u32) -> MonthlySummary {
    MonthlySummary {
        year,
        month,
        subscriptions_total: Decimal::from_str("500.00").unwrap(),
        transactions_total: Decimal::from_str("200.00").unwrap(),
        overall_total: Decimal::from_str("700.00").unwrap(),
        subscriptions_count: 5,
        transactions_count: 10,
    }
}

fn make_mock_breakdown() -> Vec<CategoryBreakdown> {
    vec![
        CategoryBreakdown {
            category_id: Some("cat-1".to_string()),
            category_name: "Streaming".to_string(),
            total: Decimal::from_str("500.00").unwrap(),
            count: 5,
            percentage: 71.4,
        },
        CategoryBreakdown {
            category_id: Some("cat-2".to_string()),
            category_name: "Food".to_string(),
            total: Decimal::from_str("200.00").unwrap(),
            count: 10,
            percentage: 28.6,
        },
    ]
}

fn make_mock_trends(month: u32, year: i32, count: i32) -> Vec<SpendingTrend> {
    (0..count)
        .map(|i| SpendingTrend {
            period: if month > i as u32 {
                format!("{}/{:02}", month - i as u32, year % 100)
            } else {
                format!("{}/{}", 12 - i as u32 + month, (year - 1) % 100)
            },
            total: Decimal::from_str("700.00").unwrap(),
        })
        .collect()
}

#[async_trait]
impl ReportServiceTrait for MockReportService {
    async fn get_monthly_summary(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<MonthlySummary> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }
        let _ = user_id;
        Ok(make_mock_summary(year, month))
    }

    async fn get_category_breakdown(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<Vec<CategoryBreakdown>> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }
        let _ = (user_id, year, month);
        Ok(make_mock_breakdown())
    }

    async fn get_recent_trends(
        &self,
        user_id: Uuid,
        months_count: i32,
    ) -> AppResult<Vec<SpendingTrend>> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }
        let _ = user_id;
        Ok(make_mock_trends(6, 2024, months_count))
    }

    async fn get_full_report(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<ReportResponse> {
        if self.config.should_fail {
            return Err(AppError::Internal(self.config.error_message.clone()));
        }

        Ok(ReportResponse {
            monthly_summary: self.get_monthly_summary(user_id, year, month).await?,
            category_breakdown: self.get_category_breakdown(user_id, year, month).await?,
            recent_trends: self.get_recent_trends(user_id, 6).await?,
        })
    }
}

// ============================================================================
// Preferences Service Trait + Mock
// ============================================================================

#[async_trait]
pub trait PreferencesServiceTrait: Send + Sync {
    async fn get_preferences(&self, user_id: Uuid) -> AppResult<PreferencesResponse>;
    async fn update_preferences(
        &self,
        user_id: Uuid,
        req: UpdatePreferencesRequest,
    ) -> AppResult<PreferencesResponse>;
}

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
        if let Some(first_day) = req.first_day_of_week {
            updated.first_day_of_week = first_day;
        }
        if let Some(date_fmt) = req.date_format {
            updated.date_format = date_fmt;
        }

        prefs.insert(user_id, updated.clone());
        Ok(updated)
    }
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

        #[test]
        fn clone_preserves_state() {
            let config = MockConfig::failure("err");
            let cloned = config.clone();
            assert!(cloned.should_fail);
            assert_eq!(cloned.error_message, "err");
        }

        #[test]
        fn debug_format() {
            let config = MockConfig::success();
            let debug = format!("{:?}", config);
            assert!(debug.contains("should_fail"));
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
            assert!(resp.requires_verification);
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
        async fn register_records_call() {
            let mock = MockAuthService::success();
            let req = make_register_request("rec@test.com");
            mock.register(req.clone()).await.unwrap();
            let calls = mock.register_calls.lock().unwrap();
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].email, "rec@test.com");
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
        async fn login_records_call() {
            let mock = MockAuthService::success();
            let login_req = LoginRequest {
                email: "rec@login.com".to_string(),
                password: "pass".to_string(),
            };
            mock.login(login_req.clone()).await.ok();
            let calls = mock.login_calls.lock().unwrap();
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].email, "rec@login.com");
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
            let result = mock
                .verify_email(Uuid::new_v4(), "123456".to_string())
                .await;
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
        async fn verify_email_already_verified() {
            let mock = MockAuthService::success();
            let req = make_register_request("alreadyv@test.com");
            let user_id = mock.register(req).await.unwrap().user.id;
            mock.verify_email(user_id, "123456".to_string())
                .await
                .unwrap();

            let result = mock.verify_email(user_id, "123456".to_string()).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::BadRequest(_)));
        }

        #[tokio::test]
        async fn verify_email_failure_config() {
            let mock = MockAuthService::failure("Verify failed");
            let result = mock
                .verify_email(Uuid::new_v4(), "123456".to_string())
                .await;
            assert!(result.is_err());
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
        async fn verify_token_extracts_user_id() {
            let mock = MockAuthService::success();
            let req = make_register_request("extract@test.com");
            let resp = mock.register(req).await.unwrap();
            let claims = mock.verify_token(&resp.token).await.unwrap();
            assert_eq!(claims.sub, resp.user.id);
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
        async fn update_settings_user_not_found() {
            let mock = MockAuthService::success();
            let update_req = UpdateSettingsRequest {
                name: Some("X".to_string()),
                currency: None,
            };
            let result = mock.update_settings(Uuid::new_v4(), update_req).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn update_settings_failure_config() {
            let mock = MockAuthService::failure("No update");
            let update_req = UpdateSettingsRequest {
                name: Some("X".to_string()),
                currency: None,
            };
            let result = mock.update_settings(Uuid::new_v4(), update_req).await;
            assert!(result.is_err());
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
            mock.verify_email(user_id, "123456".to_string())
                .await
                .unwrap();

            let result = mock.resend_verification(user_id).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn resend_verification_user_not_found() {
            let mock = MockAuthService::success();
            let result = mock.resend_verification(Uuid::new_v4()).await;
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

        #[tokio::test]
        async fn make_user_verified() {
            let mock = MockAuthService::success();
            let id = Uuid::new_v4();
            let user = mock.make_user(id, "v@t.com", "Verified", true);
            assert!(user.verified);
            assert!(user.verification_code.is_none());
        }

        #[tokio::test]
        async fn make_user_unverified() {
            let mock = MockAuthService::success();
            let id = Uuid::new_v4();
            let user = mock.make_user(id, "u@t.com", "Unverified", false);
            assert!(!user.verified);
            assert!(user.verification_code.is_some());
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
            assert!(cat.is_default);
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
        async fn create_second_not_default() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            mock.create(user_id, make_category_request("First"))
                .await
                .unwrap();
            let second = mock
                .create(user_id, make_category_request("Second"))
                .await
                .unwrap();
            assert!(!second.is_default);
        }

        #[tokio::test]
        async fn create_failure() {
            let mock = MockCategoryService::failure("Category creation failed");
            let req = make_category_request("Fail");
            let result = mock.create(Uuid::new_v4(), req).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn create_records_call() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            mock.create(user_id, make_category_request("Rec"))
                .await
                .unwrap();
            let calls = mock.create_calls.lock().unwrap();
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].0, user_id);
        }

        #[tokio::test]
        async fn create_uncategorized_success() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();

            let result = mock.create_uncategorized(user_id).await;
            assert!(result.is_ok());

            let cats = mock.get_all(user_id).await.unwrap();
            assert_eq!(cats.len(), 1);
            assert_eq!(cats[0].name, "Uncategorized");
            assert!(cats[0].is_system);
            assert!(cats[0].is_default);
        }

        #[tokio::test]
        async fn create_uncategorized_idempotent() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();

            mock.create_uncategorized(user_id).await.unwrap();
            mock.create_uncategorized(user_id).await.unwrap();

            let cats = mock.get_all(user_id).await.unwrap();
            assert_eq!(cats.len(), 1);
        }

        #[tokio::test]
        async fn create_uncategorized_failure() {
            let mock = MockCategoryService::failure("DB error");
            let result = mock.create_uncategorized(Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_all_returns_user_categories() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();

            mock.create(user_id, make_category_request("Cat 1"))
                .await
                .unwrap();
            mock.create(user_id, make_category_request("Cat 2"))
                .await
                .unwrap();

            let result = mock.get_all(user_id).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 2);
        }

        #[tokio::test]
        async fn get_all_filters_by_user() {
            let mock = MockCategoryService::success();
            let user1 = Uuid::new_v4();
            let user2 = Uuid::new_v4();

            mock.create(user1, make_category_request("U1 Cat"))
                .await
                .unwrap();
            mock.create(user2, make_category_request("U2 Cat"))
                .await
                .unwrap();

            let result = mock.get_all(user1).await.unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].name, "U1 Cat");
        }

        #[tokio::test]
        async fn get_all_failure() {
            let mock = MockCategoryService::failure("DB error");
            let result = mock.get_all(Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_by_id_success() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            let cat = mock
                .create(user_id, make_category_request("Find Me"))
                .await
                .unwrap();

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
        async fn get_by_id_failure() {
            let mock = MockCategoryService::failure("DB error");
            let result = mock.get_by_id(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn update_success() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            let cat = mock
                .create(user_id, make_category_request("Original"))
                .await
                .unwrap();

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
        async fn update_records_call() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            let cat = mock
                .create(user_id, make_category_request("Orig"))
                .await
                .unwrap();

            let req = UpdateCategoryRequest {
                name: Some("New".to_string()),
                icon: serde_json::Value::Null,
                color: serde_json::Value::Null,
            };
            mock.update(user_id, cat.id, req).await.unwrap();
            let calls = mock.update_calls.lock().unwrap();
            assert_eq!(calls.len(), 1);
        }

        #[tokio::test]
        async fn update_not_found() {
            let mock = MockCategoryService::success();
            let req = UpdateCategoryRequest {
                name: Some("X".to_string()),
                icon: serde_json::Value::Null,
                color: serde_json::Value::Null,
            };
            let result = mock.update(Uuid::new_v4(), Uuid::new_v4(), req).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_default_success() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            let cat = mock
                .create(user_id, make_category_request("Default"))
                .await
                .unwrap();

            let result = mock.get_default(user_id).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().id, cat.id);
        }

        #[tokio::test]
        async fn get_default_after_set_default() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            let _cat1 = mock
                .create(user_id, make_category_request("First"))
                .await
                .unwrap();
            let cat2 = mock
                .create(user_id, make_category_request("Second"))
                .await
                .unwrap();
            mock.set_default(user_id, cat2.id).await.unwrap();

            let default = mock.get_default(user_id).await.unwrap();
            assert_eq!(default.id, cat2.id);
        }

        #[tokio::test]
        async fn get_default_not_found() {
            let mock = MockCategoryService::success();
            let result = mock.get_default(Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_default_failure() {
            let mock = MockCategoryService::failure("DB error");
            let result = mock.get_default(Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn delete_success() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            mock.create(user_id, make_category_request("Default"))
                .await
                .unwrap();
            let cat = mock
                .create(user_id, make_category_request("To Delete"))
                .await
                .unwrap();
            assert!(!cat.is_default);

            let result = mock.delete(user_id, cat.id).await;
            assert!(result.is_ok());
            assert!(mock.get_by_id(user_id, cat.id).await.is_err());
        }

        #[tokio::test]
        async fn delete_default_fails() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            let cat = mock
                .create(user_id, make_category_request("Default"))
                .await
                .unwrap();

            let result = mock.delete(user_id, cat.id).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::BadRequest(_)));
        }

        #[tokio::test]
        async fn delete_not_found() {
            let mock = MockCategoryService::success();
            let result = mock.delete(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn delete_records_call() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            mock.create(user_id, make_category_request("Default"))
                .await
                .unwrap();
            let cat = mock
                .create(user_id, make_category_request("Del"))
                .await
                .unwrap();
            mock.delete(user_id, cat.id).await.unwrap();
            let calls = mock.delete_calls.lock().unwrap();
            assert_eq!(calls.len(), 1);
        }

        #[tokio::test]
        async fn set_default_success() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();

            let cat1 = mock
                .create(user_id, make_category_request("First"))
                .await
                .unwrap();
            let cat2 = mock
                .create(user_id, make_category_request("Second"))
                .await
                .unwrap();

            let result = mock.set_default(user_id, cat2.id).await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_default);

            let first = mock.get_by_id(user_id, cat1.id).await.unwrap();
            assert!(!first.is_default);
        }

        #[tokio::test]
        async fn set_default_not_found() {
            let mock = MockCategoryService::success();
            let result = mock.set_default(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn set_default_failure() {
            let mock = MockCategoryService::failure("DB error");
            let result = mock.set_default(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_with_counts_success() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            mock.create(user_id, make_category_request("Cat1"))
                .await
                .unwrap();
            mock.create(user_id, make_category_request("Cat2"))
                .await
                .unwrap();

            let result = mock.get_with_counts(user_id).await;
            assert!(result.is_ok());
            let counts = result.unwrap();
            assert_eq!(counts.len(), 2);
            assert_eq!(counts[0].subscription_count, 0);
            assert_eq!(counts[0].transaction_count, 0);
        }

        #[tokio::test]
        async fn get_with_counts_empty() {
            let mock = MockCategoryService::success();
            let result = mock.get_with_counts(Uuid::new_v4()).await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }

        #[tokio::test]
        async fn get_with_counts_failure() {
            let mock = MockCategoryService::failure("DB error");
            let result = mock.get_with_counts(Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn add_category_method() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            let cat = Category {
                id: Uuid::new_v4(),
                user_id,
                name: "Direct".to_string(),
                icon: None,
                color: None,
                is_system: false,
                is_default: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            mock.add_category(cat);
            let cats = mock.get_all(user_id).await.unwrap();
            assert_eq!(cats.len(), 1);
            assert_eq!(cats[0].name, "Direct");
        }

        #[tokio::test]
        async fn with_categories_constructor() {
            let user_id = Uuid::new_v4();
            let cat = Category {
                id: Uuid::new_v4(),
                user_id,
                name: "Pre".to_string(),
                icon: None,
                color: None,
                is_system: false,
                is_default: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            let mock = MockCategoryService::with_categories(vec![cat]);
            let cats = mock.get_all(user_id).await.unwrap();
            assert_eq!(cats.len(), 1);
        }

        #[tokio::test]
        async fn make_category_helper() {
            let mock = MockCategoryService::success();
            let id = Uuid::new_v4();
            let user_id = Uuid::new_v4();
            let cat = mock.make_category(id, user_id, "HelperCat");
            assert_eq!(cat.name, "HelperCat");
            assert_eq!(cat.id, id);
            assert_eq!(cat.user_id, user_id);
        }
    }

    // ── MockSubscriptionService Tests ─────────────────────────────────────

    mod mock_subscription_service {
        use super::*;

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
        async fn create_records_call() {
            let mock = MockSubscriptionService::success();
            let user_id = Uuid::new_v4();
            mock.create(user_id, make_subscription_request("Rec"))
                .await
                .unwrap();
            let calls = mock.create_calls.lock().unwrap();
            assert_eq!(calls.len(), 1);
        }

        #[tokio::test]
        async fn get_all_returns_user_subscriptions() {
            let mock = MockSubscriptionService::success();
            let user_id = Uuid::new_v4();

            mock.create(user_id, make_subscription_request("Sub 1"))
                .await
                .unwrap();
            mock.create(user_id, make_subscription_request("Sub 2"))
                .await
                .unwrap();

            let result = mock.get_all(user_id).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 2);
        }

        #[tokio::test]
        async fn get_all_filters_by_user() {
            let mock = MockSubscriptionService::success();
            let user1 = Uuid::new_v4();
            let user2 = Uuid::new_v4();

            mock.create(user1, make_subscription_request("U1"))
                .await
                .unwrap();
            mock.create(user2, make_subscription_request("U2"))
                .await
                .unwrap();

            let result = mock.get_all(user1).await.unwrap();
            assert_eq!(result.len(), 1);
        }

        #[tokio::test]
        async fn get_all_failure() {
            let mock = MockSubscriptionService::failure("DB error");
            let result = mock.get_all(Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_by_id_success() {
            let mock = MockSubscriptionService::success();
            let user_id = Uuid::new_v4();
            let sub = mock
                .create(user_id, make_subscription_request("Find Me"))
                .await
                .unwrap();

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
        async fn update_success() {
            let mock = MockSubscriptionService::success();
            let user_id = Uuid::new_v4();
            let sub = mock
                .create(user_id, make_subscription_request("Original"))
                .await
                .unwrap();

            let req = UpdateSubscriptionRequest {
                name: Some("Updated".to_string()),
                amount: None,
                currency: None,
                frequency: None,
                start_date: None,
                next_billing_date: None,
                category_id: None,
                is_active: Some(false),
                notes: None,
                logo_url: None,
            };
            let result = mock.update(user_id, sub.id, req).await;
            assert!(result.is_ok());
            let updated = result.unwrap();
            assert_eq!(updated.name, "Updated");
            assert!(!updated.is_active);
        }

        #[tokio::test]
        async fn update_not_found() {
            let mock = MockSubscriptionService::success();
            let req = UpdateSubscriptionRequest {
                name: Some("X".to_string()),
                amount: None,
                currency: None,
                frequency: None,
                start_date: None,
                next_billing_date: None,
                category_id: None,
                is_active: None,
                notes: None,
                logo_url: None,
            };
            let result = mock.update(Uuid::new_v4(), Uuid::new_v4(), req).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn update_failure() {
            let mock = MockSubscriptionService::failure("DB error");
            let req = UpdateSubscriptionRequest {
                name: None,
                amount: None,
                currency: None,
                frequency: None,
                start_date: None,
                next_billing_date: None,
                category_id: None,
                is_active: None,
                notes: None,
                logo_url: None,
            };
            let result = mock.update(Uuid::new_v4(), Uuid::new_v4(), req).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn delete_success() {
            let mock = MockSubscriptionService::success();
            let user_id = Uuid::new_v4();
            let sub = mock
                .create(user_id, make_subscription_request("To Delete"))
                .await
                .unwrap();

            let result = mock.delete(user_id, sub.id).await;
            assert!(result.is_ok());
            assert!(mock.get_by_id(user_id, sub.id).await.is_err());
        }

        #[tokio::test]
        async fn delete_not_found() {
            let mock = MockSubscriptionService::success();
            let result = mock.delete(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_upcoming_returns_active_subs() {
            let mock = MockSubscriptionService::success();
            let user_id = Uuid::new_v4();
            let sub = mock
                .create(user_id, make_subscription_request("Active"))
                .await
                .unwrap();
            assert!(sub.is_active);

            let result = mock.get_upcoming(user_id, 7).await;
            assert!(result.is_ok());
            let upcoming = result.unwrap();
            assert_eq!(upcoming.len(), 1);
            assert_eq!(upcoming[0].name, "Active");
        }

        #[tokio::test]
        async fn get_upcoming_no_results() {
            let mock = MockSubscriptionService::success();
            let result = mock.get_upcoming(Uuid::new_v4(), 7).await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }

        #[tokio::test]
        async fn get_upcoming_failure() {
            let mock = MockSubscriptionService::failure("DB error");
            let result = mock.get_upcoming(Uuid::new_v4(), 7).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_calendar_success() {
            let mock = MockSubscriptionService::success();
            let user_id = Uuid::new_v4();
            mock.create(user_id, make_subscription_request("Cal"))
                .await
                .unwrap();

            let result = mock.get_calendar(user_id, 2024, 6).await;
            assert!(result.is_ok());
            let items = result.unwrap();
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].name, "Cal");
            assert!(items[0].is_active);
        }

        #[tokio::test]
        async fn get_calendar_empty() {
            let mock = MockSubscriptionService::success();
            let result = mock.get_calendar(Uuid::new_v4(), 2024, 1).await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }

        #[tokio::test]
        async fn get_calendar_failure() {
            let mock = MockSubscriptionService::failure("DB error");
            let result = mock.get_calendar(Uuid::new_v4(), 2024, 1).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn add_subscription_method() {
            let mock = MockSubscriptionService::success();
            let user_id = Uuid::new_v4();
            let sub = Subscription {
                id: Uuid::new_v4(),
                user_id,
                category_id: Uuid::new_v4(),
                name: "Direct".to_string(),
                amount: Decimal::TEN,
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
            mock.add_subscription(sub);
            let result = mock.get_all(user_id).await.unwrap();
            assert_eq!(result.len(), 1);
        }

        #[tokio::test]
        async fn with_subscriptions_constructor() {
            let user_id = Uuid::new_v4();
            let sub = Subscription {
                id: Uuid::new_v4(),
                user_id,
                category_id: Uuid::new_v4(),
                name: "Pre".to_string(),
                amount: Decimal::TEN,
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
            let mock = MockSubscriptionService::with_subscriptions(vec![sub]);
            let result = mock.get_all(user_id).await.unwrap();
            assert_eq!(result.len(), 1);
        }
    }

    // ── MockTransactionService Tests ─────────────────────────────────────

    mod mock_transaction_service {
        use super::*;

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
        async fn create_records_call() {
            let mock = MockTransactionService::success();
            let user_id = Uuid::new_v4();
            mock.create(user_id, make_transaction_request("Rec"))
                .await
                .unwrap();
            let calls = mock.create_calls.lock().unwrap();
            assert_eq!(calls.len(), 1);
        }

        #[tokio::test]
        async fn get_all_returns_user_transactions() {
            let mock = MockTransactionService::success();
            let user_id = Uuid::new_v4();

            mock.create(user_id, make_transaction_request("Trans 1"))
                .await
                .unwrap();
            mock.create(user_id, make_transaction_request("Trans 2"))
                .await
                .unwrap();

            let result = mock.get_all(user_id).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 2);
        }

        #[tokio::test]
        async fn get_all_filters_by_user() {
            let mock = MockTransactionService::success();
            let user1 = Uuid::new_v4();
            let user2 = Uuid::new_v4();

            mock.create(user1, make_transaction_request("U1"))
                .await
                .unwrap();
            mock.create(user2, make_transaction_request("U2"))
                .await
                .unwrap();

            let result = mock.get_all(user1).await.unwrap();
            assert_eq!(result.len(), 1);
        }

        #[tokio::test]
        async fn get_all_failure() {
            let mock = MockTransactionService::failure("DB error");
            let result = mock.get_all(Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_by_id_success() {
            let mock = MockTransactionService::success();
            let user_id = Uuid::new_v4();
            let trans = mock
                .create(user_id, make_transaction_request("Find Me"))
                .await
                .unwrap();

            let result = mock.get_by_id(user_id, trans.id).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().name, "Find Me");
        }

        #[tokio::test]
        async fn get_by_id_not_found() {
            let mock = MockTransactionService::success();
            let result = mock.get_by_id(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_by_id_failure() {
            let mock = MockTransactionService::failure("DB error");
            let result = mock.get_by_id(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn update_success() {
            let mock = MockTransactionService::success();
            let user_id = Uuid::new_v4();
            let trans = mock
                .create(user_id, make_transaction_request("Original"))
                .await
                .unwrap();

            let req = UpdateTransactionRequest {
                name: Some("Updated".to_string()),
                amount: Some(Decimal::from_str("50.00").unwrap()),
                currency: None,
                date: None,
                category_id: None,
                notes: None,
                logo_url: None,
            };
            let result = mock.update(user_id, trans.id, req).await;
            assert!(result.is_ok());
            let updated = result.unwrap();
            assert_eq!(updated.name, "Updated");
        }

        #[tokio::test]
        async fn update_not_found() {
            let mock = MockTransactionService::success();
            let req = UpdateTransactionRequest {
                name: Some("X".to_string()),
                amount: None,
                currency: None,
                date: None,
                category_id: None,
                notes: None,
                logo_url: None,
            };
            let result = mock.update(Uuid::new_v4(), Uuid::new_v4(), req).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn update_failure() {
            let mock = MockTransactionService::failure("DB error");
            let req = UpdateTransactionRequest {
                name: None,
                amount: None,
                currency: None,
                date: None,
                category_id: None,
                notes: None,
                logo_url: None,
            };
            let result = mock.update(Uuid::new_v4(), Uuid::new_v4(), req).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn delete_success() {
            let mock = MockTransactionService::success();
            let user_id = Uuid::new_v4();
            let trans = mock
                .create(user_id, make_transaction_request("To Delete"))
                .await
                .unwrap();

            let result = mock.delete(user_id, trans.id).await;
            assert!(result.is_ok());
            assert!(mock.get_by_id(user_id, trans.id).await.is_err());
        }

        #[tokio::test]
        async fn delete_not_found() {
            let mock = MockTransactionService::success();
            let result = mock.delete(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn delete_failure() {
            let mock = MockTransactionService::failure("DB error");
            let result = mock.delete(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_by_date_range_success() {
            let mock = MockTransactionService::success();
            let user_id = Uuid::new_v4();

            let req1 = CreateTransactionRequest {
                name: "June".to_string(),
                amount: Decimal::from_str("50.00").unwrap(),
                currency: "USD".to_string(),
                date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
                category_id: Some(Uuid::new_v4()),
                notes: None,
                logo_url: None,
            };
            let req2 = CreateTransactionRequest {
                name: "July".to_string(),
                amount: Decimal::from_str("75.00").unwrap(),
                currency: "USD".to_string(),
                date: NaiveDate::from_ymd_opt(2024, 7, 20).unwrap(),
                category_id: Some(Uuid::new_v4()),
                notes: None,
                logo_url: None,
            };
            mock.create(user_id, req1).await.unwrap();
            mock.create(user_id, req2).await.unwrap();

            let from = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
            let to = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();

            let result = mock.get_by_date_range(user_id, from, to).await;
            assert!(result.is_ok());
            let filtered = result.unwrap();
            assert_eq!(filtered.len(), 1);
            assert_eq!(filtered[0].name, "June");
        }

        #[tokio::test]
        async fn get_by_date_range_empty() {
            let mock = MockTransactionService::success();
            let user_id = Uuid::new_v4();
            let from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
            let to = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

            let result = mock.get_by_date_range(user_id, from, to).await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }

        #[tokio::test]
        async fn get_by_date_range_failure() {
            let mock = MockTransactionService::failure("DB error");
            let from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
            let to = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
            let result = mock.get_by_date_range(Uuid::new_v4(), from, to).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn add_transaction_method() {
            let mock = MockTransactionService::success();
            let user_id = Uuid::new_v4();
            let t = Transaction {
                id: Uuid::new_v4(),
                user_id,
                category_id: Uuid::new_v4(),
                name: "Direct".to_string(),
                amount: Decimal::TEN,
                currency: "USD".to_string(),
                date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                notes: None,
                logo_url: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            mock.add_transaction(t);
            let result = mock.get_all(user_id).await.unwrap();
            assert_eq!(result.len(), 1);
        }

        #[tokio::test]
        async fn with_transactions_constructor() {
            let user_id = Uuid::new_v4();
            let t = Transaction {
                id: Uuid::new_v4(),
                user_id,
                category_id: Uuid::new_v4(),
                name: "Pre".to_string(),
                amount: Decimal::TEN,
                currency: "USD".to_string(),
                date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                notes: None,
                logo_url: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            let mock = MockTransactionService::with_transactions(vec![t]);
            let result = mock.get_all(user_id).await.unwrap();
            assert_eq!(result.len(), 1);
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
        async fn list_success() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();
            mock.add_notification(make_notification(user_id, "Test"));
            mock.add_notification(make_notification(user_id, "Test 2"));

            let result = mock.list(user_id).await;
            assert!(result.is_ok());
            let resp = result.unwrap();
            assert_eq!(resp.notifications.len(), 2);
            assert_eq!(resp.unread_count, 2);
        }

        #[tokio::test]
        async fn list_empty() {
            let mock = MockNotificationService::success();
            let result = mock.list(Uuid::new_v4()).await;
            assert!(result.is_ok());
            assert!(result.unwrap().notifications.is_empty());
        }

        #[tokio::test]
        async fn list_failure() {
            let mock = MockNotificationService::failure("Database error");
            let result = mock.list(Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn list_filters_by_user() {
            let mock = MockNotificationService::success();
            let user1 = Uuid::new_v4();
            let user2 = Uuid::new_v4();
            mock.add_notification(make_notification(user1, "U1"));
            mock.add_notification(make_notification(user2, "U2"));

            let result = mock.list(user1).await.unwrap();
            assert_eq!(result.notifications.len(), 1);
            assert_eq!(result.notifications[0].title, "U1");
        }

        #[tokio::test]
        async fn mark_read_success() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();
            let notif = make_notification(user_id, "To Mark");
            let notif_id = notif.id;
            mock.add_notification(notif);

            let result = mock.mark_read(user_id, notif_id).await;
            assert!(result.is_ok());
            assert!(result.unwrap());

            let resp = mock.list(user_id).await.unwrap();
            let marked = resp
                .notifications
                .iter()
                .find(|n| n.id == notif_id)
                .unwrap();
            assert!(marked.is_read);
        }

        #[tokio::test]
        async fn mark_read_not_found() {
            let mock = MockNotificationService::success();
            let result = mock.mark_read(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn mark_read_failure() {
            let mock = MockNotificationService::failure("DB error");
            let result = mock.mark_read(Uuid::new_v4(), Uuid::new_v4()).await;
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
            assert_eq!(result.unwrap(), 2);

            let resp = mock.list(user_id).await.unwrap();
            assert_eq!(resp.unread_count, 0);
        }

        #[tokio::test]
        async fn mark_all_read_no_unread() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();
            let result = mock.mark_all_read(user_id).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 0);
        }

        #[tokio::test]
        async fn mark_all_read_failure() {
            let mock = MockNotificationService::failure("DB error");
            let result = mock.mark_all_read(Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn delete_success() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();
            let notif = make_notification(user_id, "To Delete");
            let notif_id = notif.id;
            mock.add_notification(notif);

            let result = mock.delete(user_id, notif_id).await;
            assert!(result.is_ok());
            assert!(result.unwrap());

            let resp = mock.list(user_id).await.unwrap();
            assert!(resp.notifications.iter().all(|n| n.id != notif_id));
        }

        #[tokio::test]
        async fn delete_not_found() {
            let mock = MockNotificationService::success();
            let result = mock.delete(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn delete_failure() {
            let mock = MockNotificationService::failure("DB error");
            let result = mock.delete(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_notification_preferences_default() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();

            let result = mock.get_notification_preferences(user_id).await;
            assert!(result.is_ok());
            let prefs = result.unwrap();
            assert_eq!(prefs.len(), 3);
            assert!(prefs.iter().all(|p| !p.enabled));
        }

        #[tokio::test]
        async fn get_notification_preferences_failure() {
            let mock = MockNotificationService::failure("DB error");
            let result = mock.get_notification_preferences(Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn update_notification_preferences_enable_email() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();

            let updates = vec![UpdatePreferenceRequest {
                channel: "email".to_string(),
                enabled: true,
            }];
            let result = mock.update_notification_preferences(user_id, updates).await;
            assert!(result.is_ok());

            let prefs = result.unwrap();
            let email = prefs.iter().find(|p| p.channel == "email").unwrap();
            assert!(email.enabled);
        }

        #[tokio::test]
        async fn update_notification_preferences_multiple() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();

            let updates = vec![
                UpdatePreferenceRequest {
                    channel: "email".to_string(),
                    enabled: true,
                },
                UpdatePreferenceRequest {
                    channel: "push".to_string(),
                    enabled: true,
                },
            ];
            let result = mock.update_notification_preferences(user_id, updates).await;
            assert!(result.is_ok());

            let prefs = result.unwrap();
            assert!(prefs.iter().find(|p| p.channel == "email").unwrap().enabled);
            assert!(prefs.iter().find(|p| p.channel == "push").unwrap().enabled);
            assert!(
                !prefs
                    .iter()
                    .find(|p| p.channel == "system")
                    .unwrap()
                    .enabled
            );
        }

        #[tokio::test]
        async fn update_notification_preferences_failure() {
            let mock = MockNotificationService::failure("DB error");
            let updates = vec![UpdatePreferenceRequest {
                channel: "email".to_string(),
                enabled: true,
            }];
            let result = mock
                .update_notification_preferences(Uuid::new_v4(), updates)
                .await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn add_notification_method() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();
            mock.add_notification(make_notification(user_id, "Direct"));
            let resp = mock.list(user_id).await.unwrap();
            assert_eq!(resp.notifications.len(), 1);
        }

        #[tokio::test]
        async fn with_notifications_constructor() {
            let user_id = Uuid::new_v4();
            let notif = make_notification(user_id, "Pre");
            let mock = MockNotificationService::with_notifications(vec![notif]);
            let resp = mock.list(user_id).await.unwrap();
            assert_eq!(resp.notifications.len(), 1);
        }

        #[tokio::test]
        async fn mark_all_read_only_affects_target_user() {
            let mock = MockNotificationService::success();
            let user1 = Uuid::new_v4();
            let user2 = Uuid::new_v4();
            mock.add_notification(make_notification(user1, "U1"));
            mock.add_notification(make_notification(user2, "U2"));

            mock.mark_all_read(user1).await.unwrap();
            let resp2 = mock.list(user2).await.unwrap();
            assert_eq!(resp2.unread_count, 1);
        }
    }

    // ── MockReportService Tests ─────────────────────────────────────────────

    mod mock_report_service {
        use super::*;

        #[tokio::test]
        async fn get_monthly_summary_success() {
            let mock = MockReportService::success();
            let user_id = Uuid::new_v4();

            let result = mock.get_monthly_summary(user_id, 2024, 6).await;
            assert!(result.is_ok());
            let summary = result.unwrap();
            assert_eq!(summary.year, 2024);
            assert_eq!(summary.month, 6);
            assert_eq!(
                summary.subscriptions_total,
                Decimal::from_str("500.00").unwrap()
            );
            assert_eq!(
                summary.transactions_total,
                Decimal::from_str("200.00").unwrap()
            );
            assert_eq!(summary.overall_total, Decimal::from_str("700.00").unwrap());
        }

        #[tokio::test]
        async fn get_monthly_summary_failure() {
            let mock = MockReportService::failure("Report generation failed");
            let result = mock.get_monthly_summary(Uuid::new_v4(), 2024, 6).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_category_breakdown_success() {
            let mock = MockReportService::success();
            let result = mock.get_category_breakdown(Uuid::new_v4(), 2024, 6).await;
            assert!(result.is_ok());
            let breakdown = result.unwrap();
            assert_eq!(breakdown.len(), 2);
            assert_eq!(breakdown[0].category_name, "Streaming");
            assert_eq!(breakdown[1].category_name, "Food");
        }

        #[tokio::test]
        async fn get_category_breakdown_failure() {
            let mock = MockReportService::failure("DB error");
            let result = mock.get_category_breakdown(Uuid::new_v4(), 2024, 6).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_recent_trends_success() {
            let mock = MockReportService::success();
            let result = mock.get_recent_trends(Uuid::new_v4(), 3).await;
            assert!(result.is_ok());
            let trends = result.unwrap();
            assert_eq!(trends.len(), 3);
        }

        #[tokio::test]
        async fn get_recent_trends_failure() {
            let mock = MockReportService::failure("DB error");
            let result = mock.get_recent_trends(Uuid::new_v4(), 6).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_full_report_success() {
            let mock = MockReportService::success();
            let user_id = Uuid::new_v4();

            let result = mock.get_full_report(user_id, 2024, 6).await;
            assert!(result.is_ok());
            let report = result.unwrap();
            assert_eq!(report.monthly_summary.year, 2024);
            assert_eq!(report.monthly_summary.month, 6);
            assert!(!report.category_breakdown.is_empty());
            assert!(!report.recent_trends.is_empty());
        }

        #[tokio::test]
        async fn get_full_report_failure() {
            let mock = MockReportService::failure("Report generation failed");
            let result = mock.get_full_report(Uuid::new_v4(), 2024, 6).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn get_monthly_summary_different_months() {
            let mock = MockReportService::success();
            for m in 1..=12u32 {
                let result = mock.get_monthly_summary(Uuid::new_v4(), 2024, m).await;
                assert!(result.is_ok());
                assert_eq!(result.unwrap().month, m);
            }
        }

        #[tokio::test]
        async fn get_recent_trends_zero_count() {
            let mock = MockReportService::success();
            let result = mock.get_recent_trends(Uuid::new_v4(), 0).await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
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
                first_day_of_week: None,
                date_format: None,
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
                first_day_of_week: None,
                date_format: None,
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
                first_day_of_week: None,
                date_format: None,
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

            let req1 = UpdatePreferencesRequest {
                theme: Some("dark".to_string()),
                subscriptions_view: Some("list".to_string()),
                first_day_of_week: None,
                date_format: None,
            };
            mock.update_preferences(user_id, req1).await.unwrap();

            let req2 = UpdatePreferencesRequest {
                theme: Some("light".to_string()),
                subscriptions_view: None,
                first_day_of_week: None,
                date_format: None,
            };
            let result = mock.update_preferences(user_id, req2).await.unwrap();

            assert_eq!(result.theme, "light");
            assert_eq!(result.subscriptions_view, "list");
        }

        #[tokio::test]
        async fn update_preferences_failure() {
            let mock = MockPreferencesService::failure("Update failed");
            let req = UpdatePreferencesRequest {
                theme: Some("dark".to_string()),
                subscriptions_view: None,
                first_day_of_week: None,
                date_format: None,
            };
            let result = mock.update_preferences(Uuid::new_v4(), req).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn update_preferences_both_none() {
            let mock = MockPreferencesService::success();
            let user_id = Uuid::new_v4();
            // Set initial
            let req1 = UpdatePreferencesRequest {
                theme: Some("dark".to_string()),
                subscriptions_view: Some("list".to_string()),
                first_day_of_week: None,
                date_format: None,
            };
            mock.update_preferences(user_id, req1).await.unwrap();

            let req2 = UpdatePreferencesRequest {
                theme: None,
                subscriptions_view: None,
                first_day_of_week: None,
                date_format: None,
            };
            let result = mock.update_preferences(user_id, req2).await.unwrap();
            assert_eq!(result.theme, "dark");
            assert_eq!(result.subscriptions_view, "list");
        }

        #[tokio::test]
        async fn different_users_isolated() {
            let mock = MockPreferencesService::success();
            let user1 = Uuid::new_v4();
            let user2 = Uuid::new_v4();

            mock.update_preferences(
                user1,
                UpdatePreferencesRequest {
                    theme: Some("dark".to_string()),
                    subscriptions_view: None,
                first_day_of_week: None,
                date_format: None,
                },
            )
            .await
            .unwrap();
            mock.update_preferences(
                user2,
                UpdatePreferencesRequest {
                    theme: Some("light".to_string()),
                    subscriptions_view: None,
                first_day_of_week: None,
                date_format: None,
                },
            )
            .await
            .unwrap();

            let p1 = mock.get_preferences(user1).await.unwrap();
            let p2 = mock.get_preferences(user2).await.unwrap();
            assert_eq!(p1.theme, "dark");
            assert_eq!(p2.theme, "light");
        }
    }

    // ── Clone Derive Tests ─────────────────────────────────────────────────

    mod model_clone_derive {
        use super::*;
        use crate::auth::models::User;
        use crate::categories::models::Category;
        use crate::notifications::models::Notification;
        use crate::subscriptions::models::Subscription;
        use crate::transactions::models::Transaction;
        use chrono::NaiveDate;
        use rust_decimal::Decimal;
        use sqlx::types::chrono::Utc;
        use uuid::Uuid;

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

        #[test]
        fn notification_clone_preserves_fields() {
            let notif = Notification {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                notification_type: "billing".to_string(),
                title: "Clone Notif".to_string(),
                message: Some("msg".to_string()),
                is_read: false,
                created_at: Utc::now(),
            };
            let cloned = notif.clone();
            assert_eq!(cloned.title, notif.title);
            assert_eq!(cloned.is_read, notif.is_read);
        }
    }

    // ── Default Trait Tests ───────────────────────────────────────────────────

    mod default_impls {
        use super::*;

        #[test]
        fn mock_auth_service_default() {
            let mock: MockAuthService = Default::default();
            assert!(!mock.config.should_fail);
        }

        #[test]
        fn mock_category_service_default() {
            let mock: MockCategoryService = Default::default();
            assert!(!mock.config.should_fail);
        }

        #[test]
        fn mock_subscription_service_default() {
            let mock: MockSubscriptionService = Default::default();
            assert!(!mock.config.should_fail);
        }

        #[test]
        fn mock_transaction_service_default() {
            let mock: MockTransactionService = Default::default();
            assert!(!mock.config.should_fail);
        }

        #[test]
        fn mock_notification_service_default() {
            let mock: MockNotificationService = Default::default();
            assert!(!mock.config.should_fail);
        }

        #[test]
        fn mock_report_service_default() {
            let mock: MockReportService = Default::default();
            assert!(!mock.config.should_fail);
        }

        #[test]
        fn mock_preferences_service_default() {
            let mock: MockPreferencesService = Default::default();
            assert!(!mock.config.should_fail);
        }
    }

    // ── Extra Error Path Tests ───────────────────────────────────────────────

    mod error_paths {
        use super::*;

        #[tokio::test]
        async fn auth_get_user_by_id_failure() {
            let mock = MockAuthService::failure("no user");
            let result = mock.get_user_by_id(Uuid::new_v4()).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
        }

        #[tokio::test]
        async fn auth_resend_verification_failure() {
            let mock = MockAuthService::failure("resend fail");
            let result = mock.resend_verification(Uuid::new_v4()).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::Internal(_)));
        }

        #[tokio::test]
        async fn category_update_failure_config() {
            let mock = MockCategoryService::failure("db gone");
            let req = UpdateCategoryRequest {
                name: Some("X".to_string()),
                icon: serde_json::Value::Null,
                color: serde_json::Value::Null,
            };
            let result = mock.update(Uuid::new_v4(), Uuid::new_v4(), req).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
        }

        #[tokio::test]
        async fn category_get_by_id_failure_config() {
            let mock = MockCategoryService::failure("db gone");
            let result = mock.get_by_id(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
        }

        #[tokio::test]
        async fn subscription_get_by_id_failure_config() {
            let mock = MockSubscriptionService::failure("db gone");
            let result = mock.get_by_id(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
        }

        #[tokio::test]
        async fn category_delete_failure_config() {
            let mock = MockCategoryService::failure("db gone");
            let result = mock.delete(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
        }

        #[tokio::test]
        async fn subscription_update_amount() {
            let mock = MockSubscriptionService::success();
            let user_id = Uuid::new_v4();
            let sub = mock
                .create(
                    user_id,
                    CreateSubscriptionRequest {
                        name: "Netflix".to_string(),
                        amount: Decimal::from_str("9.99").unwrap(),
                        currency: "USD".to_string(),
                        frequency: "monthly".to_string(),
                        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        category_id: None,
                        notes: None,
                        logo_url: None,
                    },
                )
                .await
                .unwrap();

            let req = UpdateSubscriptionRequest {
                name: None,
                amount: Some(Decimal::from_str("19.99").unwrap()),
                currency: None,
                frequency: None,
                start_date: None,
                next_billing_date: None,
                category_id: None,
                is_active: None,
                notes: None,
                logo_url: None,
            };
            let result = mock.update(user_id, sub.id, req).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().amount, Decimal::from_str("19.99").unwrap());
        }

        #[tokio::test]
        async fn subscription_delete_failure_config() {
            let mock = MockSubscriptionService::failure("db gone");
            let result = mock.delete(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
        }

        #[tokio::test]
        async fn transaction_delete_failure_config() {
            let mock = MockTransactionService::failure("db gone");
            let result = mock.delete(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
        }

        #[tokio::test]
        async fn report_trends_with_month_wrap() {
            let _mock = MockReportService::success();
            // month = 1, count = 3, should wrap to previous year
            let trends = make_mock_trends(1, 2024, 3);
            assert_eq!(trends.len(), 3);
            // First trend should be current month (1/24)
            assert_eq!(trends[0].period, "1/24");
        }
    }

    // ── ReportService Additional Tests ─────────────────────────────────────

    mod report_service_additional {
        use super::*;

        #[tokio::test]
        async fn report_get_monthly_summary_success() {
            let mock = MockReportService::success();
            let user_id = Uuid::new_v4();
            let result = mock.get_monthly_summary(user_id, 2024, 6).await;
            assert!(result.is_ok());
            let summary = result.unwrap();
            assert_eq!(summary.year, 2024);
            assert_eq!(summary.month, 6);
        }

        #[tokio::test]
        async fn report_get_monthly_summary_failure() {
            let mock = MockReportService::failure("db error");
            let result = mock.get_monthly_summary(Uuid::new_v4(), 2024, 6).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn report_get_category_breakdown_success() {
            let mock = MockReportService::success();
            let user_id = Uuid::new_v4();
            let result = mock.get_category_breakdown(user_id, 2024, 6).await;
            assert!(result.is_ok());
            let breakdown = result.unwrap();
            assert!(!breakdown.is_empty());
        }

        #[tokio::test]
        async fn report_get_category_breakdown_failure() {
            let mock = MockReportService::failure("db error");
            let result = mock.get_category_breakdown(Uuid::new_v4(), 2024, 6).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn report_get_recent_trends_success() {
            let mock = MockReportService::success();
            let user_id = Uuid::new_v4();
            let result = mock.get_recent_trends(user_id, 6).await;
            assert!(result.is_ok());
            let trends = result.unwrap();
            assert_eq!(trends.len(), 6);
        }

        #[tokio::test]
        async fn report_get_recent_trends_failure() {
            let mock = MockReportService::failure("db error");
            let result = mock.get_recent_trends(Uuid::new_v4(), 6).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn report_get_full_report_success() {
            let mock = MockReportService::success();
            let user_id = Uuid::new_v4();
            let result = mock.get_full_report(user_id, 2024, 6).await;
            assert!(result.is_ok());
            let report = result.unwrap();
            assert_eq!(report.monthly_summary.year, 2024);
            assert_eq!(report.monthly_summary.month, 6);
        }

        #[tokio::test]
        async fn report_get_full_report_failure() {
            let mock = MockReportService::failure("db error");
            let result = mock.get_full_report(Uuid::new_v4(), 2024, 6).await;
            assert!(result.is_err());
        }
    }

    // ── PreferencesService Tests ─────────────────────────────────────────────

    mod preferences_service_tests {
        use super::*;

        #[tokio::test]
        async fn preferences_get_success() {
            let mock = MockPreferencesService::success();
            let user_id = Uuid::new_v4();
            let result = mock.get_preferences(user_id).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn preferences_get_failure() {
            let mock = MockPreferencesService::failure("not found");
            let result = mock.get_preferences(Uuid::new_v4()).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
        }

        #[tokio::test]
        async fn preferences_update_success() {
            let mock = MockPreferencesService::success();
            let user_id = Uuid::new_v4();
            let req = UpdatePreferencesRequest {
                theme: Some("dark".to_string()),
                subscriptions_view: Some("list".to_string()),
                first_day_of_week: None,
                date_format: None,
            };
            let result = mock.update_preferences(user_id, req).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn preferences_update_failure() {
            let mock = MockPreferencesService::failure("update failed");
            let req = UpdatePreferencesRequest {
                theme: Some("light".to_string()),
                subscriptions_view: Some("grid".to_string()),
                first_day_of_week: None,
                date_format: None,
            };
            let result = mock.update_preferences(Uuid::new_v4(), req).await;
            assert!(result.is_err());
        }
    }

    // ── NotificationService Tests ───────────────────────────────────────────

    mod notification_service_tests {
        use super::*;

        #[tokio::test]
        async fn notification_list_success() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();
            let result = mock.list(user_id).await;
            assert!(result.is_ok());
            let resp = result.unwrap();
            assert_eq!(resp.notifications.len(), 0);
            assert_eq!(resp.unread_count, 0);
        }

        #[tokio::test]
        async fn notification_list_failure() {
            let mock = MockNotificationService::failure("list failed");
            let result = mock.list(Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn notification_mark_read_success() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();

            // Pre-populate with notification
            {
                let mut notifs = mock.notifications.lock().unwrap();
                notifs.push(Notification {
                    id: Uuid::new_v4(),
                    user_id,
                    notification_type: "test".to_string(),
                    title: "Test".to_string(),
                    message: Some("msg".to_string()),
                    is_read: false,
                    created_at: Utc::now(),
                });
            }

            let notif_id = mock.notifications.lock().unwrap()[0].id;
            let result = mock.mark_read(user_id, notif_id).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn notification_mark_read_failure() {
            let mock = MockNotificationService::failure("mark read failed");
            let result = mock.mark_read(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn notification_mark_all_read_success() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();
            let result = mock.mark_all_read(user_id).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn notification_mark_all_read_failure() {
            let mock = MockNotificationService::failure("mark all failed");
            let result = mock.mark_all_read(Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn notification_delete_success() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();

            // Pre-populate with notification
            {
                let mut notifs = mock.notifications.lock().unwrap();
                notifs.push(Notification {
                    id: Uuid::new_v4(),
                    user_id,
                    notification_type: "test".to_string(),
                    title: "Test".to_string(),
                    message: Some("msg".to_string()),
                    is_read: false,
                    created_at: Utc::now(),
                });
            }

            let notif_id = mock.notifications.lock().unwrap()[0].id;
            let result = mock.delete(user_id, notif_id).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn notification_delete_failure() {
            let mock = MockNotificationService::failure("delete failed");
            let result = mock.delete(Uuid::new_v4(), Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn notification_get_preferences_success() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();
            let result = mock.get_notification_preferences(user_id).await;
            assert!(result.is_ok());
            let prefs = result.unwrap();
            assert!(!prefs.is_empty());
        }

        #[tokio::test]
        async fn notification_get_preferences_failure() {
            let mock = MockNotificationService::failure("get prefs failed");
            let result = mock.get_notification_preferences(Uuid::new_v4()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn notification_update_preferences_success() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();
            let prefs = vec![UpdatePreferenceRequest {
                channel: "email".to_string(),
                enabled: true,
            }];
            let result = mock.update_notification_preferences(user_id, prefs).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn notification_update_preferences_failure() {
            let mock = MockNotificationService::failure("update prefs failed");
            let result = mock
                .update_notification_preferences(
                    Uuid::new_v4(),
                    vec![UpdatePreferenceRequest {
                        channel: "push".to_string(),
                        enabled: false,
                    }],
                )
                .await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn notification_add_to_mock() {
            let mock = MockNotificationService::success();
            let user_id = Uuid::new_v4();
            let notif = Notification {
                id: Uuid::new_v4(),
                user_id,
                notification_type: "test".to_string(),
                title: "Test Notif".to_string(),
                message: None,
                is_read: false,
                created_at: Utc::now(),
            };
            mock.add_notification(notif);
            let result = mock.list(user_id).await.unwrap();
            assert_eq!(result.notifications.len(), 1);
        }
    }

    // ── SubscriptionService Additional Tests ────────────────────────────────

    mod subscription_service_additional {
        use super::*;

        #[tokio::test]
        async fn subscription_update_frequency() {
            let mock = MockSubscriptionService::success();
            let user_id = Uuid::new_v4();
            let sub = mock
                .create(
                    user_id,
                    CreateSubscriptionRequest {
                        name: "Test".to_string(),
                        amount: Decimal::from_str("10.00").unwrap(),
                        currency: "USD".to_string(),
                        frequency: "monthly".to_string(),
                        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        category_id: None,
                        notes: None,
                        logo_url: None,
                    },
                )
                .await
                .unwrap();

            let req = UpdateSubscriptionRequest {
                name: None,
                amount: None,
                currency: None,
                frequency: Some("yearly".to_string()),
                start_date: None,
                next_billing_date: None,
                category_id: None,
                is_active: None,
                notes: None,
                logo_url: None,
            };
            let result = mock.update(user_id, sub.id, req).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().frequency, "yearly");
        }

        #[tokio::test]
        async fn subscription_update_next_billing_date() {
            let mock = MockSubscriptionService::success();
            let user_id = Uuid::new_v4();
            let sub = mock
                .create(
                    user_id,
                    CreateSubscriptionRequest {
                        name: "Test".to_string(),
                        amount: Decimal::from_str("10.00").unwrap(),
                        currency: "USD".to_string(),
                        frequency: "monthly".to_string(),
                        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        category_id: None,
                        notes: None,
                        logo_url: None,
                    },
                )
                .await
                .unwrap();

            let req = UpdateSubscriptionRequest {
                name: None,
                amount: None,
                currency: None,
                frequency: None,
                start_date: None,
                next_billing_date: Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
                category_id: None,
                is_active: None,
                notes: None,
                logo_url: None,
            };
            let result = mock.update(user_id, sub.id, req).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn subscription_update_deactivate() {
            let mock = MockSubscriptionService::success();
            let user_id = Uuid::new_v4();
            let sub = mock
                .create(
                    user_id,
                    CreateSubscriptionRequest {
                        name: "Active".to_string(),
                        amount: Decimal::from_str("10.00").unwrap(),
                        currency: "USD".to_string(),
                        frequency: "monthly".to_string(),
                        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        category_id: None,
                        notes: None,
                        logo_url: None,
                    },
                )
                .await
                .unwrap();
            assert!(sub.is_active);

            let req = UpdateSubscriptionRequest {
                name: None,
                amount: None,
                currency: None,
                frequency: None,
                start_date: None,
                next_billing_date: None,
                category_id: None,
                is_active: Some(false),
                notes: None,
                logo_url: None,
            };
            let result = mock.update(user_id, sub.id, req).await;
            assert!(result.is_ok());
            assert!(!result.unwrap().is_active);
        }
    }

    // ── TransactionService Additional Tests ────────────────────────────────

    mod transaction_service_additional {
        use super::*;

        #[tokio::test]
        async fn transaction_get_by_date_range_success() {
            let mock = MockTransactionService::success();
            let user_id = Uuid::new_v4();
            let from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
            let to = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
            let result = mock.get_by_date_range(user_id, from, to).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn transaction_get_by_date_range_failure() {
            let mock = MockTransactionService::failure("db error");
            let result = mock
                .get_by_date_range(
                    Uuid::new_v4(),
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
                )
                .await;
            assert!(result.is_err());
        }
    }

    // ── CategoryService Additional Tests ───────────────────────────────────

    mod category_service_additional {
        use super::*;

        #[tokio::test]
        async fn category_create_uncategorized_idempotent_many_calls() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            for _ in 0..5 {
                mock.create_uncategorized(user_id).await.unwrap();
            }
            let cats = mock.get_all(user_id).await.unwrap();
            assert_eq!(cats.len(), 1);
        }

        #[tokio::test]
        async fn category_update_only_name() {
            let mock = MockCategoryService::success();
            let user_id = Uuid::new_v4();
            let cat = mock
                .create(
                    user_id,
                    CreateCategoryRequest {
                        name: "Original".to_string(),
                        icon: None,
                        color: None,
                    },
                )
                .await
                .unwrap();

            let req = UpdateCategoryRequest {
                name: Some("New Name".to_string()),
                icon: serde_json::Value::Null,
                color: serde_json::Value::Null,
            };
            let result = mock.update(user_id, cat.id, req).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().name, "New Name");
        }
    }
}
