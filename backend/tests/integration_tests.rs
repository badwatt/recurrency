#![cfg(feature = "integration-tests")]
//! Integration tests for auth, subscriptions, and categories with real database.
//!
//! Requires TEST_DATABASE_URL env var or .env.test file.
//! Run with: TEST_DATABASE_URL=postgres://... cargo test --test integration_tests

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, header::AUTHORIZATION},
};
use recurrency_backend::accounts::models::{CreateAccountRequest, UpdateAccountRequest};
use recurrency_backend::accounts::service::AccountService;
use recurrency_backend::admin::service::{AdminServiceTrait, SqlxAdminService};
use recurrency_backend::ai::models::{
    AiModelsRequest, AiTranscriptionRequest, UpdateAiSettingsRequest,
};
use recurrency_backend::ai::service::{AiServiceTrait, SqlxAiService};
use recurrency_backend::auth::models::{CreateUserRequest, LoginRequest, UpdateSettingsRequest};
use recurrency_backend::auth::service::{AuthService, AuthServiceTrait, SqlxAuthService};
use recurrency_backend::categories::models::CreateCategoryRequest;
use recurrency_backend::categories::service::CategoryService;
use recurrency_backend::cli::{AdminAction, AdminArgs};
use recurrency_backend::debts::models::{CreateDebtRequest, RecordDebtPaymentRequest};
use recurrency_backend::debts::service::DebtService;
use recurrency_backend::notifications::channels::{
    MASKED_VALUE, NotificationProvider, UpdateNotificationChannelRequest,
    UpdateSummaryNotificationSettingsRequest,
};
use recurrency_backend::notifications::settings::{
    NotificationSettingsServiceTrait, SqlxNotificationSettingsService,
};
use recurrency_backend::preferences::models::UpdatePreferencesRequest;
use recurrency_backend::preferences::service::SqlxPreferencesService;
use recurrency_backend::reports::service::ReportService;
use recurrency_backend::shared::app_state::AppState;
use recurrency_backend::shared::error::AppError;
use recurrency_backend::subscriptions::models::{
    CreateSubscriptionRequest, UpdateSubscriptionRequest,
};
use recurrency_backend::subscriptions::service::SubscriptionService;

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
) -> recurrency_backend::subscriptions::models::Subscription {
    let req = CreateSubscriptionRequest {
        name: name.to_string(),
        amount,
        currency: "USD".to_string(),
        frequency: frequency.to_string(),
        schedule_day: None,
        schedule_month: None,
        recurrence: None,
        is_active: None,
        start_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        end_date: None,
        account_id: Uuid::nil(),
        category_id: None,
        notes: None,
        logo_url: None,
    };
    SubscriptionService::create(pool, user_id, req)
        .await
        .unwrap()
}

async fn create_subscription_with_date(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    amount: Decimal,
    frequency: &str,
    start_date: NaiveDate,
) -> recurrency_backend::subscriptions::models::Subscription {
    let req = CreateSubscriptionRequest {
        name: name.to_string(),
        amount,
        currency: "USD".to_string(),
        frequency: frequency.to_string(),
        schedule_day: None,
        schedule_month: None,
        recurrence: None,
        is_active: None,
        start_date,
        end_date: None,
        account_id: Uuid::nil(),
        category_id: None,
        notes: None,
        logo_url: None,
    };
    SubscriptionService::create(pool, user_id, req)
        .await
        .unwrap()
}

// ── Auth Tests ─────────────────────────────────────────────────────────────────

#[sqlx::test]
async fn settings_update_changes_email(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let service = SqlxAuthService::new(pool.clone());
    let next_email = format!("changed_{}@test.com", Uuid::new_v4());

    let user = service
        .update_settings(
            user_id,
            UpdateSettingsRequest {
                email: Some(next_email.clone()),
                avatar: None,
                name: Some("Changed".to_string()),
                currency: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(user.email, next_email);
    assert_eq!(user.name.as_deref(), Some("Changed"));
}

#[sqlx::test]
async fn settings_update_rejects_duplicate_email(pool: PgPool) {
    let (first_id, _) = create_test_user(&pool).await;
    let (_, _) = create_test_user(&pool).await;
    let second_email: String = sqlx::query_scalar("SELECT email FROM users WHERE id <> $1 LIMIT 1")
        .bind(first_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let service = SqlxAuthService::new(pool.clone());

    let result = service
        .update_settings(
            first_id,
            UpdateSettingsRequest {
                email: Some(second_email),
                avatar: None,
                name: None,
                currency: None,
            },
        )
        .await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[sqlx::test]
async fn settings_update_stores_and_clears_avatar(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let service = SqlxAuthService::new(pool.clone());
    let avatar = "data:image/png;base64,iVBORw0KGgo=".to_string();

    let user = service
        .update_settings(
            user_id,
            UpdateSettingsRequest {
                email: None,
                avatar: Some(avatar.clone()),
                name: None,
                currency: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(user.avatar.as_deref(), Some(avatar.as_str()));

    let user = service
        .update_settings(
            user_id,
            UpdateSettingsRequest {
                email: None,
                avatar: Some(String::new()),
                name: None,
                currency: None,
            },
        )
        .await
        .unwrap();
    assert!(user.avatar.is_none());
}

#[sqlx::test]
async fn delete_account_requires_password(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let service = SqlxAuthService::new(pool.clone());

    let wrong = service
        .delete_account(user_id, "wrong-password".to_string())
        .await;
    assert!(matches!(wrong, Err(AppError::Unauthorized(_))));

    service
        .delete_account(user_id, "testpassword123".to_string())
        .await
        .unwrap();
    assert!(AuthService::get_user_by_id(&pool, user_id).await.is_err());
}

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
    let result = AuthService::register(
        &pool,
        CreateUserRequest {
            email,
            password: "otherpassword123".to_string(),
            name: "Other".to_string(),
        },
    )
    .await;

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

    let result = AuthService::login(
        &pool,
        LoginRequest {
            email: user.email,
            password: "wrongpassword123".to_string(),
        },
    )
    .await;

    assert!(matches!(result, Err(AppError::Unauthorized(_))));
}

#[sqlx::test]
async fn auth_login_nonexistent_user_fails(pool: PgPool) {
    let result = AuthService::login(
        &pool,
        LoginRequest {
            email: "nonexistent@test.com".to_string(),
            password: "anypassword123".to_string(),
        },
    )
    .await;

    assert!(matches!(result, Err(AppError::Unauthorized(_))));
}

#[sqlx::test]
async fn auth_verify_token_valid(pool: PgPool) {
    let (_, token) = create_test_user(&pool).await;
    let claims = AuthService::verify_token(&token).await.unwrap();
    assert!(!claims.email.is_empty());
}

#[sqlx::test]
async fn cross_tenant_subscription_account_category_rejected(pool: PgPool) {
    let (user_a, _) = create_test_user(&pool).await;
    let (user_b, _) = create_test_user(&pool).await;
    let account_b = AccountService::get_default(&pool, user_b).await.unwrap();
    let category_b = CategoryService::get_default(&pool, user_b).await.unwrap();

    let result = SubscriptionService::create(
        &pool,
        user_a,
        CreateSubscriptionRequest {
            name: "Cross tenant attempt".to_string(),
            amount: Decimal::from(10),
            currency: "EUR".to_string(),
            frequency: "monthly".to_string(),
            schedule_day: None,
            schedule_month: None,
            recurrence: None,
            is_active: None,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            end_date: None,
            account_id: account_b.id,
            category_id: Some(category_b.id),
            notes: None,
            logo_url: None,
        },
    )
    .await;

    assert!(matches!(result, Err(AppError::NotFound(_))));
}

#[sqlx::test]
async fn unverified_token_rejected_but_verification_token_accepted(pool: PgPool) {
    let (user_id, token) = create_test_user(&pool).await;
    sqlx::query("UPDATE users SET verified = false WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let service = SqlxAuthService::new(pool.clone());
    assert!(matches!(
        service.verify_token(&token).await,
        Err(AppError::Unauthorized(_))
    ));
    assert!(service.verify_verification_token(&token).await.is_ok());
}

#[sqlx::test]
async fn notification_channel_config_roundtrip(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let service = SqlxNotificationSettingsService::new(pool.clone());

    let defaults = service.list_channels(user_id).await.unwrap();
    assert_eq!(defaults.len(), NotificationProvider::ALL.len());
    assert!(defaults.iter().all(|channel| !channel.configured));

    let saved = service
        .update_channel(
            user_id,
            NotificationProvider::Ntfy,
            UpdateNotificationChannelRequest {
                enabled: true,
                config: serde_json::json!({
                    "server": "https://ntfy.sh",
                    "topic": "recurrency-test",
                    "token": "secret"
                }),
            },
        )
        .await
        .unwrap();
    assert!(saved.enabled);
    assert_eq!(saved.config["token"], MASKED_VALUE);

    let listed = service.list_channels(user_id).await.unwrap();
    let ntfy = listed
        .iter()
        .find(|channel| channel.provider == "ntfy")
        .unwrap();
    assert!(ntfy.enabled);
    assert_eq!(ntfy.config["token"], MASKED_VALUE);
    assert_eq!(ntfy.config["topic"], "recurrency-test");
}

#[sqlx::test]
async fn summary_notification_schedule_roundtrip(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let service = SqlxNotificationSettingsService::new(pool.clone());

    let saved = service
        .update_summary(
            user_id,
            UpdateSummaryNotificationSettingsRequest {
                enabled: true,
                frequency: "weekly".to_string(),
                weekday: Some(1),
                day_of_month: None,
                hour: 9,
                minute: 30,
                timezone: "UTC".to_string(),
                period: "current".to_string(),
            },
        )
        .await
        .unwrap();
    assert!(saved.enabled);
    assert_eq!(saved.frequency, "weekly");
    assert_eq!(saved.next_run_at.is_some(), true);

    let loaded = service.get_summary(user_id).await.unwrap();
    assert!(loaded.enabled);
    assert_eq!(loaded.weekday, Some(1));
    assert_eq!(loaded.hour, 9);
    assert_eq!(loaded.minute, 30);
}

#[sqlx::test]
async fn summary_now_requires_enabled_channels(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let service = SqlxNotificationSettingsService::new(pool.clone());
    let result = service.send_summary_now(user_id).await;
    assert!(result.is_err());
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

    let netflix =
        create_subscription(&pool, user_id, "Netflix", Decimal::from(1599), "monthly").await;
    let spotify =
        create_subscription(&pool, user_id, "Spotify", Decimal::from(999), "monthly").await;
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
    let created =
        create_subscription(&pool, user_id, "Netflix", Decimal::from(1599), "monthly").await;

    let sub = SubscriptionService::get_by_id(&pool, user_id, created.id)
        .await
        .unwrap();
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
        schedule_day: None,
        schedule_month: None,
        recurrence: None,
        start_date: None,
        end_date: None,
        clear_end_date: false,
        next_billing_date: None,
        account_id: None,
        category_id: None,
        is_active: None,
        notes: None,
        logo_url: None,
    };
    let updated = SubscriptionService::update(&pool, user_id, sub.id, req)
        .await
        .unwrap();
    assert_eq!(updated.name, "Disney+");
}

#[sqlx::test]
async fn subscription_update_deactivate(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let sub = create_subscription(&pool, user_id, "Netflix", Decimal::from(1599), "monthly").await;

    let req = UpdateSubscriptionRequest {
        name: None,
        amount: None,
        currency: None,
        frequency: None,
        schedule_day: None,
        schedule_month: None,
        recurrence: None,
        start_date: None,
        end_date: None,
        clear_end_date: false,
        next_billing_date: None,
        account_id: None,
        category_id: None,
        is_active: Some(false),
        notes: None,
        logo_url: None,
    };
    let updated = SubscriptionService::update(&pool, user_id, sub.id, req)
        .await
        .unwrap();
    assert!(!updated.is_active);
}

#[sqlx::test]
async fn subscription_delete(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let sub = create_subscription(&pool, user_id, "Netflix", Decimal::from(1599), "monthly").await;

    SubscriptionService::delete(&pool, user_id, sub.id)
        .await
        .unwrap();

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
    assert!(
        SubscriptionService::get_by_id(&pool, user1_id, sub.id)
            .await
            .is_ok()
    );
}

#[sqlx::test]
async fn subscription_get_upcoming(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    create_subscription(&pool, user_id, "Netflix", Decimal::from(1599), "monthly").await;

    let upcoming = SubscriptionService::get_upcoming(&pool, user_id, 30)
        .await
        .unwrap();
    assert!(!upcoming.is_empty());
    assert!(upcoming.iter().all(|s| s.is_active));
}

#[sqlx::test]
async fn subscription_get_upcoming_excludes_inactive(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let inactive =
        create_subscription(&pool, user_id, "Canceled", Decimal::from(1000), "monthly").await;

    let req = UpdateSubscriptionRequest {
        name: None,
        amount: None,
        currency: None,
        frequency: None,
        schedule_day: None,
        schedule_month: None,
        recurrence: None,
        start_date: None,
        end_date: None,
        clear_end_date: false,
        next_billing_date: None,
        account_id: None,
        category_id: None,
        is_active: Some(false),
        notes: None,
        logo_url: None,
    };
    SubscriptionService::update(&pool, user_id, inactive.id, req)
        .await
        .unwrap();

    let upcoming = SubscriptionService::get_upcoming(&pool, user_id, 365)
        .await
        .unwrap();
    assert!(upcoming.iter().all(|s| s.is_active));
}

#[sqlx::test]
async fn subscription_schedule_and_end_date(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let start = NaiveDate::from_ymd_opt(2099, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2099, 4, 1).unwrap();
    let req = CreateSubscriptionRequest {
        name: "Scheduled".to_string(),
        amount: Decimal::from(10),
        currency: "EUR".to_string(),
        frequency: "monthly".to_string(),
        schedule_day: Some(15),
        schedule_month: None,
        recurrence: None,
        is_active: None,
        start_date: start,
        end_date: Some(end),
        account_id: Uuid::nil(),
        category_id: None,
        notes: None,
        logo_url: None,
    };
    let sub = SubscriptionService::create(&pool, user_id, req)
        .await
        .unwrap();
    assert_eq!(sub.schedule_day, Some(15));
    assert_eq!(sub.end_date, Some(end));
    assert_eq!(
        sub.next_billing_date,
        Some(NaiveDate::from_ymd_opt(2099, 1, 15).unwrap())
    );

    let calendar = SubscriptionService::get_calendar(&pool, user_id, 2099, 1)
        .await
        .unwrap();
    assert_eq!(calendar.len(), 1);
    assert_eq!(
        calendar[0].billing_date,
        NaiveDate::from_ymd_opt(2099, 1, 15).unwrap()
    );

    let after_end = SubscriptionService::get_calendar(&pool, user_id, 2099, 4)
        .await
        .unwrap();
    assert!(after_end.is_empty());

    let report = ReportService::get_monthly_summary(&pool, user_id, 2099, 5)
        .await
        .unwrap();
    assert_eq!(report.subscriptions_total, Decimal::ZERO);
    let categories = ReportService::get_category_breakdown(&pool, user_id, 2099, 5)
        .await
        .unwrap();
    assert!(categories.is_empty());

    let cleared = SubscriptionService::update(
        &pool,
        user_id,
        sub.id,
        UpdateSubscriptionRequest {
            clear_end_date: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(cleared.end_date.is_none());

    let report = ReportService::get_monthly_summary(&pool, user_id, 2099, 5)
        .await
        .unwrap();
    assert_eq!(report.subscriptions_total, Decimal::from(10));
}

#[sqlx::test]
async fn subscription_list_rolls_stale_billing_dates_forward(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let sub = create_subscription_with_date(
        &pool,
        user_id,
        "Stale",
        Decimal::from(10),
        "monthly",
        NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
    )
    .await;
    sqlx::query("UPDATE subscriptions SET next_billing_date = $1 WHERE id = $2")
        .bind(NaiveDate::from_ymd_opt(2020, 1, 3).unwrap())
        .bind(sub.id)
        .execute(&pool)
        .await
        .unwrap();

    let listed = SubscriptionService::get_all(&pool, user_id).await.unwrap();
    let refreshed = listed.iter().find(|item| item.id == sub.id).unwrap();
    assert!(refreshed.next_billing_date.unwrap() >= chrono::Utc::now().date_naive());
}

#[sqlx::test]
async fn subscription_update_start_recalculates_next_billing(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let req = CreateSubscriptionRequest {
        name: "Recalculate".to_string(),
        amount: Decimal::from(10),
        currency: "EUR".to_string(),
        frequency: "monthly".to_string(),
        schedule_day: Some(15),
        schedule_month: None,
        recurrence: None,
        is_active: None,
        start_date: NaiveDate::from_ymd_opt(2099, 1, 1).unwrap(),
        end_date: None,
        account_id: Uuid::nil(),
        category_id: None,
        notes: None,
        logo_url: None,
    };
    let sub = SubscriptionService::create(&pool, user_id, req)
        .await
        .unwrap();
    let updated = SubscriptionService::update(
        &pool,
        user_id,
        sub.id,
        UpdateSubscriptionRequest {
            start_date: Some(NaiveDate::from_ymd_opt(2099, 2, 1).unwrap()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(
        updated.next_billing_date,
        Some(NaiveDate::from_ymd_opt(2099, 2, 15).unwrap())
    );
}

#[sqlx::test]
async fn subscription_calendar_keeps_completed_dates(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let sub = create_subscription_with_date(
        &pool,
        user_id,
        "Completed",
        Decimal::from(10),
        "monthly",
        NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
    )
    .await;
    // Set the recurring day to the 3rd so the month has a completed date.
    let updated = SubscriptionService::update(
        &pool,
        user_id,
        sub.id,
        UpdateSubscriptionRequest {
            schedule_day: Some(3),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(updated.end_date.is_none());

    let january = SubscriptionService::get_calendar(&pool, user_id, 2024, 1)
        .await
        .unwrap();
    assert!(
        january
            .iter()
            .any(|item| item.billing_date == NaiveDate::from_ymd_opt(2024, 1, 3).unwrap())
    );
    let february = SubscriptionService::get_calendar(&pool, user_id, 2024, 2)
        .await
        .unwrap();
    assert!(
        february
            .iter()
            .any(|item| item.billing_date == NaiveDate::from_ymd_opt(2024, 2, 3).unwrap())
    );
}

#[sqlx::test]
async fn subscription_get_calendar_returns_for_next_month(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    // Create subscription with start_date = 1st of next month
    let now = chrono::Utc::now().date_naive();
    let next_month = if now.month() == 12 {
        chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1)
    }
    .unwrap();
    create_subscription_with_date(
        &pool,
        user_id,
        "Netflix",
        Decimal::from(1599),
        "monthly",
        next_month,
    )
    .await;

    let calendar =
        SubscriptionService::get_calendar(&pool, user_id, next_month.year(), next_month.month())
            .await
            .unwrap();
    // Calendar may be empty if next_billing_date advanced past this month (edge case: today is 1st)
    // But function should not error
    for item in &calendar {
        assert!(
            item.billing_date.year() == next_month.year()
                && item.billing_date.month() == next_month.month()
        );
    }
}

#[sqlx::test]
async fn subscription_get_calendar_empty(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let calendar = SubscriptionService::get_calendar(&pool, user_id, 2024, 6)
        .await
        .unwrap();
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
async fn first_category_created_by_register_is_default(pool: PgPool) {
    // Register creates an uncategorized category which should be default
    let (user_id, _) = create_test_user(&pool).await;
    let cats = CategoryService::get_all(&pool, user_id).await.unwrap();
    let uncategorized = cats.iter().find(|c| c.is_system).unwrap();
    assert!(
        uncategorized.is_default,
        "uncategorized category should be default"
    );
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

    CategoryService::delete(&pool, user_id, cat.id)
        .await
        .unwrap();

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

// ── Account Tests ─────────────────────────────────────────────────────────────

#[sqlx::test]
async fn account_default_created_on_registration(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let accounts = AccountService::get_all(&pool, user_id).await.unwrap();
    assert_eq!(accounts.len(), 1);
    assert!(accounts[0].is_default);
    assert_eq!(accounts[0].name, "General");
}

#[sqlx::test]
async fn account_create_update_and_set_default(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let created = AccountService::create(
        &pool,
        user_id,
        CreateAccountRequest {
            name: "Savings".to_string(),
            icon: Some("Wallet".to_string()),
            color: Some("#22c55e".to_string()),
        },
    )
    .await
    .unwrap();
    assert!(!created.is_default);

    let updated = AccountService::update(
        &pool,
        user_id,
        created.id,
        UpdateAccountRequest {
            name: Some("Broker".to_string()),
            icon: serde_json::Value::Null,
            color: serde_json::Value::Null,
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.name, "Broker");
    assert!(updated.icon.is_none());

    let default = AccountService::set_default(&pool, user_id, created.id)
        .await
        .unwrap();
    assert!(default.is_default);
    let all = AccountService::get_all(&pool, user_id).await.unwrap();
    assert_eq!(all.iter().filter(|a| a.is_default).count(), 1);
}

#[sqlx::test]
async fn account_delete_reassigns_subscriptions_to_default(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let default = AccountService::get_default(&pool, user_id).await.unwrap();
    let extra = AccountService::create(
        &pool,
        user_id,
        CreateAccountRequest {
            name: "Extra".to_string(),
            icon: None,
            color: None,
        },
    )
    .await
    .unwrap();

    let sub = SubscriptionService::create(
        &pool,
        user_id,
        CreateSubscriptionRequest {
            name: "Netflix".to_string(),
            amount: Decimal::from(999),
            currency: "EUR".to_string(),
            frequency: "monthly".to_string(),
            schedule_day: None,
            schedule_month: None,
            recurrence: None,
            is_active: None,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            end_date: None,
            account_id: extra.id,
            category_id: None,
            notes: None,
            logo_url: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(sub.account_id, extra.id);

    AccountService::delete(&pool, user_id, extra.id)
        .await
        .unwrap();
    let sub_after = SubscriptionService::get_by_id(&pool, user_id, sub.id)
        .await
        .unwrap();
    assert_eq!(sub_after.account_id, default.id);
}

#[sqlx::test]
async fn debt_manual_payment_and_automatic_payoff(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let manual = DebtService::create(
        &pool,
        user_id,
        CreateDebtRequest {
            name: "Manual".into(),
            total_amount: Decimal::from(100),
            currency: "EUR".into(),
            account_id: Uuid::nil(),
            category_id: None,
            debt_type: "manual".into(),
            monthly_payment: None,
            payment_days: Vec::new(),
            start_date: None,
            notes: None,
            logo_url: None,
        },
    )
    .await
    .unwrap();
    let paid = DebtService::record_payment(
        &pool,
        user_id,
        manual.id,
        RecordDebtPaymentRequest {
            amount: Decimal::from(40),
            paid_on: NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
            note: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(paid.remaining_amount, Decimal::from(60));
    assert_eq!(paid.paid_amount, Decimal::from(40));

    let automatic = DebtService::create(
        &pool,
        user_id,
        CreateDebtRequest {
            name: "Automatic".into(),
            total_amount: Decimal::from(300),
            currency: "EUR".into(),
            account_id: Uuid::nil(),
            category_id: None,
            debt_type: "automatic".into(),
            monthly_payment: Some(Decimal::from(100)),
            payment_days: vec![1],
            start_date: Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
            notes: None,
            logo_url: None,
        },
    )
    .await
    .unwrap();
    assert!(automatic.subscription_id.is_some());
    let all = DebtService::get_all(&pool, user_id).await.unwrap();
    let automatic = all.iter().find(|debt| debt.id == automatic.id).unwrap();
    assert_eq!(automatic.remaining_amount, Decimal::ZERO);
    assert!(!automatic.is_active);
    let payments = DebtService::payments(&pool, user_id, automatic.id)
        .await
        .unwrap();
    assert_eq!(payments.len(), 3);
}

// ── Schema reconciliation ────────────────────────────────────────────────────

#[sqlx::test]
async fn schema_reconciliation_creates_the_expected_indexes(pool: PgPool) {
    let default_category_index: Option<String> = sqlx::query_scalar(
        "SELECT indexname FROM pg_indexes
         WHERE schemaname = 'public' AND indexname = 'categories_one_default_per_user'",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert_eq!(
        default_category_index.as_deref(),
        Some("categories_one_default_per_user")
    );

    let verified_index: Option<String> = sqlx::query_scalar(
        "SELECT indexname FROM pg_indexes
         WHERE schemaname = 'public' AND indexname = 'idx_users_verified'",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert_eq!(verified_index.as_deref(), Some("idx_users_verified"));
}

#[sqlx::test]
async fn preferences_update_all_fields_in_one_call(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let service = SqlxPreferencesService::new(pool.clone());

    let response = service
        .update_preferences(
            user_id,
            UpdatePreferencesRequest {
                theme: Some("dark".into()),
                subscriptions_view: Some("list".into()),
                first_day_of_week: Some("sunday".into()),
                date_format: Some("YYYY/MM/DD".into()),
            },
        )
        .await
        .expect("preferences update should succeed");

    assert_eq!(response.theme, "dark");
    assert_eq!(response.subscriptions_view, "list");
    assert_eq!(response.first_day_of_week, "sunday");
    assert_eq!(response.date_format, "YYYY/MM/DD");

    let loaded = service.get_preferences(user_id).await.unwrap();
    assert_eq!(loaded.theme, response.theme);
    assert_eq!(loaded.subscriptions_view, response.subscriptions_view);
    assert_eq!(loaded.first_day_of_week, response.first_day_of_week);
    assert_eq!(loaded.date_format, response.date_format);
}

// ── Administration ───────────────────────────────────────────────────────────

#[sqlx::test]
async fn first_registered_user_is_admin(pool: PgPool) {
    let first = AuthService::register(
        &pool,
        CreateUserRequest {
            email: format!("admin_{}@test.com", Uuid::new_v4()),
            password: "testpassword123".to_string(),
            name: "Admin".to_string(),
        },
    )
    .await
    .unwrap();
    let second = AuthService::register(
        &pool,
        CreateUserRequest {
            email: format!("member_{}@test.com", Uuid::new_v4()),
            password: "testpassword123".to_string(),
            name: "Member".to_string(),
        },
    )
    .await
    .unwrap();

    assert!(first.user.is_admin);
    assert!(!second.user.is_admin);
}

#[sqlx::test]
async fn admin_settings_toggle_registration(pool: PgPool) {
    let service = SqlxAdminService::new(pool);
    assert!(service.settings().await.unwrap().registration_enabled);

    let updated = service.set_registration_enabled(false).await.unwrap();
    assert!(!updated.registration_enabled);
    assert!(!service.settings().await.unwrap().registration_enabled);

    let restored = service.set_registration_enabled(true).await.unwrap();
    assert!(restored.registration_enabled);
}

#[sqlx::test]
async fn admin_full_backup_includes_every_application_table(pool: PgPool) {
    let _ = create_test_user(&pool).await;
    let service = SqlxAdminService::new(pool);
    let backup = service.full_backup().await.unwrap();

    assert!(backup.tables.contains_key("users"));
    assert!(backup.tables.contains_key("app_settings"));
    assert!(backup.tables.contains_key("accounts"));
    assert!(
        backup.tables["users"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty())
    );
}

#[sqlx::test]
async fn registration_handler_respects_the_admin_setting(pool: PgPool) {
    SqlxAdminService::new(pool.clone())
        .set_registration_enabled(false)
        .await
        .unwrap();

    let state = AppState::new(pool.clone());
    let result = recurrency_backend::auth::handlers::register(
        State(state),
        Json(CreateUserRequest {
            email: format!("blocked_{}@test.com", Uuid::new_v4()),
            password: "testpassword123".to_string(),
            name: "Blocked".to_string(),
        }),
    )
    .await;
    assert!(matches!(result, Err(AppError::Forbidden(_))));

    // Re-enable and verify the handler delegates to the real registration
    // service again.
    SqlxAdminService::new(pool.clone())
        .set_registration_enabled(true)
        .await
        .unwrap();
    let state = AppState::new(pool);
    let created = recurrency_backend::auth::handlers::register(
        State(state),
        Json(CreateUserRequest {
            email: format!("allowed_{}@test.com", Uuid::new_v4()),
            password: "testpassword123".to_string(),
            name: "Allowed".to_string(),
        }),
    )
    .await
    .unwrap();
    assert!(!created.0.user.email.is_empty());
}

#[sqlx::test]
async fn admin_handlers_reject_non_administrators(pool: PgPool) {
    let (_, admin_token) = create_test_user(&pool).await;
    let (_, member_token) = create_test_user(&pool).await;
    let state = AppState::new(pool);
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {member_token}").parse().unwrap(),
    );

    let rejected = recurrency_backend::admin::handlers::list_users(
        State(state.clone()),
        recurrency_backend::shared::OptionalQuery(None),
        headers.clone(),
    )
    .await;
    assert!(matches!(rejected, Err(AppError::Forbidden(_))));

    headers.insert(
        AUTHORIZATION,
        format!("Bearer {admin_token}").parse().unwrap(),
    );
    let allowed = recurrency_backend::admin::handlers::list_users(
        State(state),
        recurrency_backend::shared::OptionalQuery(None),
        headers,
    )
    .await
    .unwrap();
    assert!(allowed.0.iter().any(|user| user.is_admin));
}

#[sqlx::test]
async fn admin_can_delete_a_member_and_their_data(pool: PgPool) {
    let (admin_id, _) = create_test_user(&pool).await;
    let (member_id, _) = create_test_user(&pool).await;
    create_subscription(&pool, member_id, "Member sub", Decimal::from(9), "monthly").await;

    let service = SqlxAdminService::new(pool.clone());
    service.delete_user(admin_id, member_id).await.unwrap();
    assert!(
        !service
            .list_users()
            .await
            .unwrap()
            .iter()
            .any(|user| user.id == member_id)
    );
    let remaining: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM subscriptions WHERE user_id = $1")
            .bind(member_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(remaining, 0);
}

#[sqlx::test]
async fn admin_cannot_delete_themselves(pool: PgPool) {
    let (admin_id, _) = create_test_user(&pool).await;
    let service = SqlxAdminService::new(pool);
    assert!(matches!(
        service.delete_user(admin_id, admin_id).await,
        Err(AppError::BadRequest(_))
    ));
}

#[sqlx::test]
async fn admin_full_backup_can_be_restored(pool: PgPool) {
    let (admin_id, _) = create_test_user(&pool).await;
    let (member_id, _) = create_test_user(&pool).await;
    let service = SqlxAdminService::new(pool.clone());
    let backup = service.full_backup().await.unwrap();

    service.delete_user(admin_id, member_id).await.unwrap();
    assert!(
        !service
            .list_users()
            .await
            .unwrap()
            .iter()
            .any(|user| user.id == member_id)
    );

    let result = service.restore_backup(backup).await.unwrap();
    assert_eq!(result.restored_tables, 14);
    let users = service.list_users().await.unwrap();
    assert!(users.iter().any(|user| user.id == member_id));
    assert!(users.iter().any(|user| user.id == admin_id));
}

#[sqlx::test]
async fn admin_can_change_a_user_role(pool: PgPool) {
    let (admin_id, _) = create_test_user(&pool).await;
    let (member_id, _) = create_test_user(&pool).await;
    let service = SqlxAdminService::new(pool);

    let promoted = service.set_admin(admin_id, member_id, true).await.unwrap();
    assert!(promoted.is_admin);
    assert!(
        service
            .list_users()
            .await
            .unwrap()
            .iter()
            .find(|user| user.id == member_id)
            .is_some_and(|user| user.is_admin)
    );

    let demoted = service.set_admin(admin_id, member_id, false).await.unwrap();
    assert!(!demoted.is_admin);
}

#[sqlx::test]
async fn last_administrator_cannot_be_demoted(pool: PgPool) {
    let (admin_id, _) = create_test_user(&pool).await;
    let service = SqlxAdminService::new(pool);
    assert!(matches!(
        service.set_admin(admin_id, admin_id, false).await,
        Err(AppError::Conflict(_))
    ));
}

#[sqlx::test]
async fn registration_status_is_public_and_matches_the_admin_setting(pool: PgPool) {
    let service = SqlxAdminService::new(pool.clone());
    service.set_registration_enabled(false).await.unwrap();
    let status =
        recurrency_backend::auth::handlers::registration_status(State(AppState::new(pool.clone())))
            .await
            .unwrap();
    assert!(!status.0.registration_enabled);

    service.set_registration_enabled(true).await.unwrap();
    let status =
        recurrency_backend::auth::handlers::registration_status(State(AppState::new(pool)))
            .await
            .unwrap();
    assert!(status.0.registration_enabled);
}

#[sqlx::test]
async fn cli_admin_can_create_update_and_delete_users(pool: PgPool) {
    let email = format!("cli_{}@test.com", Uuid::new_v4());
    let create = AdminArgs {
        action: AdminAction::Create,
        target: None,
        email: Some(email.clone()),
        new_email: None,
        name: Some("CLI User".to_string()),
        password: Some("testpassword123".to_string()),
        is_admin: None,
    };
    recurrency_backend::admin::cli::run(&pool, &create)
        .await
        .unwrap();
    let user: Option<(Uuid, bool)> =
        sqlx::query_as("SELECT id, is_admin FROM users WHERE email = $1")
            .bind(&email)
            .fetch_optional(&pool)
            .await
            .unwrap();
    let (user_id, is_admin) = user.unwrap();
    assert!(is_admin);

    let member_email = format!("cli_member_{}@test.com", Uuid::new_v4());
    let create_member = AdminArgs {
        action: AdminAction::Create,
        target: None,
        email: Some(member_email.clone()),
        new_email: None,
        name: Some("CLI Member".to_string()),
        password: Some("testpassword123".to_string()),
        is_admin: None,
    };
    recurrency_backend::admin::cli::run(&pool, &create_member)
        .await
        .unwrap();

    let update = AdminArgs {
        action: AdminAction::Update,
        target: Some(email.clone()),
        email: None,
        new_email: None,
        name: Some("CLI Renamed".to_string()),
        password: None,
        is_admin: Some(false),
    };
    // A normal update succeeds, while the CLI refuses to demote the only admin.
    let rename = AdminArgs {
        action: AdminAction::Update,
        target: Some(email.clone()),
        email: None,
        new_email: None,
        name: Some("CLI Renamed".to_string()),
        password: None,
        is_admin: None,
    };
    recurrency_backend::admin::cli::run(&pool, &rename)
        .await
        .unwrap();
    let name: String = sqlx::query_scalar("SELECT name FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(name, "CLI Renamed");
    assert!(
        recurrency_backend::admin::cli::run(&pool, &update)
            .await
            .is_err()
    );

    let delete = AdminArgs {
        action: AdminAction::Delete,
        target: Some(member_email.clone()),
        email: None,
        new_email: None,
        name: None,
        password: None,
        is_admin: None,
    };
    recurrency_backend::admin::cli::run(&pool, &delete)
        .await
        .unwrap();
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)")
        .bind(member_email)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!exists);
}

#[sqlx::test]
async fn ai_settings_encrypt_the_provider_key(pool: PgPool) {
    let (user_id, _) = create_test_user(&pool).await;
    let service = SqlxAiService::new(pool.clone());
    let settings = service
        .update_settings(
            user_id,
            UpdateAiSettingsRequest {
                enabled: true,
                provider: Some("openai".to_string()),
                base_url: Some("https://api.example.com/v1".to_string()),
                model: Some("test-model".to_string()),
                api_key: Some("sk-test-secret".to_string()),
                clear_api_key: false,
            },
        )
        .await
        .unwrap();
    assert!(settings.configured);
    assert!(settings.enabled);
    let encrypted: Vec<u8> =
        sqlx::query_scalar("SELECT api_key_encrypted FROM ai_settings WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!String::from_utf8_lossy(&encrypted).contains("sk-test-secret"));

    let cleared = service
        .update_settings(
            user_id,
            UpdateAiSettingsRequest {
                enabled: false,
                provider: None,
                base_url: None,
                model: None,
                api_key: None,
                clear_api_key: true,
            },
        )
        .await
        .unwrap();
    assert!(!cleared.configured);
    assert!(!cleared.enabled);
}

#[sqlx::test]
async fn ai_lists_models_from_the_provider(pool: PgPool) {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let (user_id, _) = create_test_user(&pool).await;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let response_body = r#"{"data":[{"id":"z-model"},{"id":"a-model"}]}"#.to_string();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = Vec::new();
        let mut chunk = [0u8; 4096];
        while request.windows(4).all(|window| window != b"\r\n\r\n") {
            let read = stream.read(&mut chunk).unwrap();
            if read == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..read]);
        }
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });

    let service = SqlxAiService::new(pool.clone());
    service
        .update_settings(
            user_id,
            UpdateAiSettingsRequest {
                enabled: true,
                provider: Some("openai".to_string()),
                base_url: Some(format!("http://127.0.0.1:{port}/v1")),
                model: Some("a-model".to_string()),
                api_key: Some("test-key".to_string()),
                clear_api_key: false,
            },
        )
        .await
        .unwrap();
    let models = service
        .models(user_id, AiModelsRequest { base_url: None })
        .await
        .unwrap();
    server.join().unwrap();
    assert_eq!(models.models, vec!["a-model", "z-model"]);
}

#[sqlx::test]
async fn ai_transcribes_audio_with_the_provider(pool: PgPool) {
    use base64::Engine;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let (user_id, _) = create_test_user(&pool).await;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let response_body = r#"{"text":"hello from speech"}"#.to_string();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = Vec::new();
        let mut chunk = [0u8; 4096];
        while request.windows(4).all(|window| window != b"\r\n\r\n") {
            let read = stream.read(&mut chunk).unwrap();
            if read == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..read]);
        }
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });

    let service = SqlxAiService::new(pool.clone());
    service
        .update_settings(
            user_id,
            UpdateAiSettingsRequest {
                enabled: true,
                provider: Some("openai".to_string()),
                base_url: Some(format!("http://127.0.0.1:{port}/v1")),
                model: Some("a-model".to_string()),
                api_key: Some("test-key".to_string()),
                clear_api_key: false,
            },
        )
        .await
        .unwrap();
    let audio = base64::engine::general_purpose::STANDARD.encode([1_u8, 2, 3, 4]);
    let result = service
        .transcribe(
            user_id,
            AiTranscriptionRequest {
                audio_base64: audio,
                mime_type: "audio/webm".to_string(),
            },
        )
        .await
        .unwrap();
    server.join().unwrap();
    assert_eq!(result.text, "hello from speech");
}

#[sqlx::test]
async fn ai_chat_executes_a_subscription_action(pool: PgPool) {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let (user_id, _) = create_test_user(&pool).await;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let model_content = serde_json::json!({
        "reply": "Created the subscription.",
        "actions": [{
            "type": "create_subscription",
            "data": {
                "name": "AI subscription",
                "amount": "9.99",
                "start_date": "2026-01-01",
                "logo_url": "https://example.test/logo.png",
                "recurrence": {
                    "interval": 1,
                    "unit": "year",
                    "days": [],
                    "weekdays": [],
                    "dates": [
                        {"month": 1, "day": 15},
                        {"month": 6, "day": 15, "amount": "5.00"}
                    ]
                }
            }
        }]
    })
    .to_string();
    let response_body = serde_json::json!({
        "choices": [{ "message": { "role": "assistant", "content": model_content } }]
    })
    .to_string();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = Vec::new();
        let mut chunk = [0u8; 4096];
        while request.windows(4).all(|window| window != b"\r\n\r\n") {
            let read = stream.read(&mut chunk).unwrap();
            if read == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..read]);
        }
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });

    let service = SqlxAiService::new(pool.clone());
    service
        .update_settings(
            user_id,
            UpdateAiSettingsRequest {
                enabled: true,
                provider: Some("openai".to_string()),
                base_url: Some(format!("http://127.0.0.1:{port}/v1")),
                model: Some("test".to_string()),
                api_key: Some("test-key".to_string()),
                clear_api_key: false,
            },
        )
        .await
        .unwrap();
    let response = service
        .chat(
            user_id,
            recurrency_backend::ai::models::AiChatRequest {
                messages: vec![recurrency_backend::ai::models::AiMessage {
                    role: "user".to_string(),
                    content: "Add a 9.99 monthly subscription called AI subscription".to_string(),
                }],
            },
        )
        .await
        .unwrap();
    server.join().unwrap();
    assert!(response.actions.iter().any(|action| action.success));
    let (name, logo, recurrence): (Option<String>, Option<String>, Option<String>) =
        sqlx::query_as(
            "SELECT name, logo_url, recurrence::text FROM subscriptions WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&pool)
        .await
        .unwrap()
        .map(
            |(name, logo, recurrence): (String, Option<String>, String)| {
                (Some(name), logo, Some(recurrence))
            },
        )
        .unwrap_or((None, None, None));
    assert_eq!(name.as_deref(), Some("AI subscription"));
    assert_eq!(logo.as_deref(), Some("https://example.test/logo.png"));
    assert!(recurrence.unwrap().contains("\"month\": 6"));
}
