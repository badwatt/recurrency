//! Integration tests for auth, subscriptions, and categories with real database.
//!
//! Requires TEST_DATABASE_URL env var or .env.test file.
//! Run with: TEST_DATABASE_URL=postgres://... cargo test --test integration_tests

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use recurrency_api::auth::models::{CreateUserRequest, LoginRequest};
use recurrency_api::auth::service::AuthService;
use recurrency_api::categories::models::CreateCategoryRequest;
use recurrency_api::categories::service::CategoryService;
use recurrency_api::shared::error::AppError;
use recurrency_api::subscriptions::models::{CreateSubscriptionRequest, UpdateSubscriptionRequest};
use recurrency_api::subscriptions::service::SubscriptionService;

// ── Test Fixtures ─────────────────────────────────────────────────────────────

/// Creates a test user and returns (user_id, token)
async fn create_test_user(pool: &PgPool) -> (Uuid, String) {
    let email = format!("test_{}@integration.test", Uuid::new_v4());
    let req = CreateUserRequest {
        email: email.clone(),
        password: "testpassword123".to_string(),
        name: "Test User".to_string(),
    };
    let auth_response = AuthService::register(pool, req)
        .await
        .expect("Failed to register user");

    // Auto-verify since SMTP not configured in tests
    sqlx::query("UPDATE users SET verified = true WHERE id = $1")
        .bind(auth_response.user.id)
        .execute(pool)
        .await
        .expect("Failed to verify user");

    // Re-login to get valid token
    let login_req = LoginRequest {
        email,
        password: "testpassword123".to_string(),
    };
    let auth_response = AuthService::login(pool, login_req)
        .await
        .expect("Failed to login");

    (auth_response.user.id, auth_response.token)
}

/// Creates a subscription via service
async fn create_subscription(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    amount: Decimal,
    frequency: &str,
) -> recurrency_api::subscriptions::models::Subscription {
    let req = CreateSubscriptionRequest {
        name: name.to_string(),
        amount,
        currency: "USD".to_string(),
        frequency: frequency.to_string(),
        start_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        category_id: None,
        notes: None,
        logo_url: None,
    };
    SubscriptionService::create(pool, user_id, req).await.unwrap()
}

// ── Auth Tests ─────────────────────────────────────────────────────────────────

#[sqlx::test]
async fn auth_register_creates_user(pool: PgPool) {
    let email = format!("register_{}@test.com", Uuid::new_v4());
    let req = CreateUserRequest {
        email: email.clone(),
        password: "testpassword123".to_string(),
        name: "Test User".to_string(),
    };

    let response = AuthService::register(&pool, req).await.unwrap();

    assert!(!response.token.is_empty());
    assert_eq!(response.user.email, email);
    assert_eq!(response.user.name.as_deref(), Some("Test User"));
}

#[sqlx::test]
async fn auth_register_duplicate_email_fails(pool: PgPool) {
    let email = format!("dup_{}@test.com", Uuid::new_v4());
    let req = CreateUserRequest {
        email: email.clone(),
        password: "testpassword123".to_string(),
        name: "User".to_string(),
    };

    AuthService::register(&pool, req).await.unwrap();
    let result = AuthService::register(&pool, CreateUserRequest {
        email,
        password: "otherpassword123".to_string(),
        name: "Other".to_string(),
    }).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[sqlx::test]
async fn auth_login_valid_credentials(pool: PgPool) {
    let (user_id, token) = create_test_user(&pool).await;
    let user = AuthService::get_user_by_id(&pool, user_id).await.unwrap();

    assert!(!token.is_empty());
    assert!(user.verified);
}

#[sqlx::test]
async fn auth_login_invalid_password_fails(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let user = AuthService::get_user_by_id(&pool, user_id).await.unwrap();

    let result = AuthService::login(&pool, LoginRequest {
        email: user.email,
        password: "wrongpassword123".to_string(),
    }).await;

    assert!(matches!(result, Err(AppError::Unauthorized(_))));
}

#[sqlx::test]
async fn auth_login_nonexistent_user_fails(pool: PgPool) {
    let result = AuthService::login(&pool, LoginRequest {
        email: "nonexistent@test.com".to_string(),
        password: "anypassword123".to_string(),
    }).await;

    assert!(matches!(result, Err(AppError::Unauthorized(_))));
}

#[sqlx::test]
async fn auth_verify_token_valid(pool: PgPool) {
    let (_, token) = create_test_user(&pool).await;
    let claims = AuthService::verify_token(&token).await.unwrap();
    assert!(!claims.email.is_empty());
}

#[sqlx::test]
async fn auth_verify_token_invalid_fails() {
    let result = AuthService::verify_token("invalid.token.here").await;
    assert!(matches!(result, Err(AppError::Jwt(_))));
}

#[sqlx::test]
async fn auth_get_user_by_id(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let user = AuthService::get_user_by_id(&pool, user_id).await.unwrap();
    assert!(user.verified);
}

#[sqlx::test]
async fn auth_get_user_by_id_not_found(pool: PgPool) {
    let result = AuthService::get_user_by_id(&pool, Uuid::new_v4()).await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}

// ── Subscription Tests ────────────────────────────────────────────────────────

#[sqlx::test]
async fn subscription_create(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;

    let sub = create_subscription(&pool, user_id, "Netflix", Decimal::from(1599), "monthly").await;

    assert_eq!(sub.name, "Netflix");
    assert_eq!(sub.amount, Decimal::from(1599));
    assert_eq!(sub.frequency, "monthly");
    assert!(sub.is_active);
}

#[sqlx::test]
async fn subscription_create_multiple(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;

    let netflix = create_subscription(&pool, user_id, "Netflix", Decimal::from(1599), "monthly").await;
    let spotify = create_subscription(&pool, user_id, "Spotify", Decimal::from(999), "monthly").await;
    let gym = create_subscription(&pool, user_id, "Gym", Decimal::from(5000), "yearly").await;

    assert_ne!(netflix.id, spotify.id);
    assert_ne!(netflix.id, gym.id);
    assert_ne!(spotify.id, gym.id);
}

#[sqlx::test]
async fn subscription_get_all(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    create_subscription(&pool, user_id, "Netflix", Decimal::from(1599), "monthly").await;
    create_subscription(&pool, user_id, "Spotify", Decimal::from(999), "monthly").await;

    let subs = SubscriptionService::get_all(&pool, user_id).await.unwrap();
    assert_eq!(subs.len(), 2);
}

#[sqlx::test]
async fn subscription_get_all_empty(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let subs = SubscriptionService::get_all(&pool, user_id).await.unwrap();
    assert!(subs.is_empty());
}

#[sqlx::test]
async fn subscription_get_all_user_isolation(pool: PgPool) {
    let (user1_id, _) = create_test_user(&pool).await;
    let (user2_id, _) = create_test_user(&pool).await;

    create_subscription(&pool, user1_id, "User1Sub", Decimal::from(1000), "monthly").await;
    create_subscription(&pool, user2_id, "User2Sub", Decimal::from(2000), "monthly").await;

    let user1_subs = SubscriptionService::get_all(&pool, user1_id).await.unwrap();
    let user2_subs = SubscriptionService::get_all(&pool, user2_id).await.unwrap();

    assert_eq!(user1_subs.len(), 1);
    assert_eq!(user1_subs[0].name, "User1Sub");
    assert_eq!(user2_subs.len(), 1);
    assert_eq!(user2_subs[0].name, "User2Sub");
}

#[sqlx::test]
async fn subscription_get_by_id(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let created = create_subscription(&pool, user_id, "Netflix", Decimal::from(1599), "monthly").await;

    let sub = SubscriptionService::get_by_id(&pool, user_id, created.id).await.unwrap();
    assert_eq!(sub.id, created.id);
}

#[sqlx::test]
async fn subscription_get_by_id_wrong_user(pool: PgPool) {
    let (user1_id, _) = create_test_user(&pool).await;
    let (user2_id, _) = create_test_user(&pool).await;
    let sub = create_subscription(&pool, user1_id, "Netflix", Decimal::from(1599), "monthly").await;

    let result = SubscriptionService::get_by_id(&pool, user2_id, sub.id).await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}

#[sqlx::test]
async fn subscription_update_name(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let sub = create_subscription(&pool, user_id, "Netflix", Decimal::from(1599), "monthly").await;

    let req = UpdateSubscriptionRequest {
        name: Some("Disney+".to_string()),
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
    let updated = SubscriptionService::update(&pool, user_id, sub.id, req).await.unwrap();
    assert_eq!(updated.name, "Disney+");
}

#[sqlx::test]
async fn subscription_update_deactivate(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let sub = create_subscription(&pool, user_id, "Netflix", Decimal::from(1599), "monthly").await;

    let req = UpdateSubscriptionRequest {
        name: None, amount: None, currency: None, frequency: None,
        start_date: None, next_billing_date: None, category_id: None,
        is_active: Some(false), notes: None, logo_url: None,
    };
    let updated = SubscriptionService::update(&pool, user_id, sub.id, req).await.unwrap();
    assert!(!updated.is_active);
}

#[sqlx::test]
async fn subscription_delete(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let sub = create_subscription(&pool, user_id, "Netflix", Decimal::from(1599), "monthly").await;

    SubscriptionService::delete(&pool, user_id, sub.id).await.unwrap();

    let result = SubscriptionService::get_by_id(&pool, user_id, sub.id).await;
    assert!(result.is_err());
}

#[sqlx::test]
async fn subscription_delete_wrong_user(pool: PgPool) {
    let (user1_id, _) = create_test_user(&pool).await;
    let (user2_id, _) = create_test_user(&pool).await;
    let sub = create_subscription(&pool, user1_id, "Netflix", Decimal::from(1599), "monthly").await;

    let result = SubscriptionService::delete(&pool, user2_id, sub.id).await;
    assert!(matches!(result, Err(AppError::NotFound(_))));

    // Verify still exists for user1
    assert!(SubscriptionService::get_by_id(&pool, user1_id, sub.id).await.is_ok());
}

#[sqlx::test]
async fn subscription_get_upcoming(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    create_subscription(&pool, user_id, "Netflix", Decimal::from(1599), "monthly").await;

    let upcoming = SubscriptionService::get_upcoming(&pool, user_id, 30).await.unwrap();
    assert!(!upcoming.is_empty());
    assert!(upcoming.iter().all(|s| s.is_active));
}

#[sqlx::test]
async fn subscription_get_upcoming_excludes_inactive(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let inactive = create_subscription(&pool, user_id, "Canceled", Decimal::from(1000), "monthly").await;

    let req = UpdateSubscriptionRequest {
        name: None, amount: None, currency: None, frequency: None,
        start_date: None, next_billing_date: None, category_id: None,
        is_active: Some(false), notes: None, logo_url: None,
    };
    SubscriptionService::update(&pool, user_id, inactive.id, req).await.unwrap();

    let upcoming = SubscriptionService::get_upcoming(&pool, user_id, 365).await.unwrap();
    assert!(upcoming.iter().all(|s| s.is_active));
}

#[sqlx::test]
async fn subscription_get_calendar(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    create_subscription(&pool, user_id, "Netflix", Decimal::from(1599), "monthly").await;

    let calendar = SubscriptionService::get_calendar(&pool, user_id, 2024, 6).await.unwrap();
    assert!(!calendar.is_empty());
    for item in &calendar {
        assert!(item.billing_date.year() == 2024 && item.billing_date.month() == 6);
    }
}

#[sqlx::test]
async fn subscription_get_calendar_empty(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let calendar = SubscriptionService::get_calendar(&pool, user_id, 2024, 6).await.unwrap();
    assert!(calendar.is_empty());
}

// ── Category Tests ─────────────────────────────────────────────────────────────

#[sqlx::test]
async fn category_create(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;

    let req = CreateCategoryRequest {
        name: "Streaming".to_string(),
        icon: Some("play".to_string()),
        color: Some("#ff0000".to_string()),
    };
    let cat = CategoryService::create(&pool, user_id, req).await.unwrap();

    assert_eq!(cat.name, "Streaming");
    assert_eq!(cat.icon.as_deref(), Some("play"));
    assert!(!cat.is_system);
}

#[sqlx::test]
async fn category_create_first_is_default(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;

    let req = CreateCategoryRequest {
        name: "First Category".to_string(),
        icon: None,
        color: None,
    };
    let cat = CategoryService::create(&pool, user_id, req).await.unwrap();
    assert!(cat.is_default);
}

#[sqlx::test]
async fn category_get_all_includes_default(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let cats = CategoryService::get_all(&pool, user_id).await.unwrap();

    assert!(!cats.is_empty());
    let has_default = cats.iter().any(|c| c.is_default && c.is_system);
    assert!(has_default);
}

#[sqlx::test]
async fn category_delete_custom(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;

    let req = CreateCategoryRequest {
        name: "To Delete".to_string(),
        icon: None,
        color: None,
    };
    let cat = CategoryService::create(&pool, user_id, req).await.unwrap();

    CategoryService::delete(&pool, user_id, cat.id).await.unwrap();

    let result = CategoryService::get_by_id(&pool, user_id, cat.id).await;
    assert!(result.is_err());
}

#[sqlx::test]
async fn category_delete_default_fails(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let default_cat = CategoryService::get_default(&pool, user_id).await.unwrap();

    let result = CategoryService::delete(&pool, user_id, default_cat.id).await;
    assert!(matches!(result, Err(AppError::BadRequest(_))));
}

#[sqlx::test]
async fn category_subscription_auto_default(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let default_cat = CategoryService::get_default(&pool, user_id).await.unwrap();

    let sub = create_subscription(&pool, user_id, "Netflix", Decimal::from(1599), "monthly").await;
    assert_eq!(sub.category_id, default_cat.id);
}
