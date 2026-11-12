use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::shared::error::{AppError, AppResult};

const FRANKFURTER_URL: &str = "https://api.frankfurter.dev/v2/rates?base=EUR";
const FIXER_URL: &str = "https://data.fixer.io/api/latest";
const CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60);
pub const PROVIDER_FRANKFURTER: &str = "frankfurter";
pub const PROVIDER_FIXER: &str = "fixer";

#[derive(Debug, Clone, Serialize)]
pub struct RatesResponse {
    pub base: String,
    pub provider: String,
    pub fetched_at: DateTime<Utc>,
    pub rates: HashMap<String, Decimal>,
}

#[derive(Debug, Clone)]
struct CachedRates {
    fetched_at: Instant,
    provider: String,
    cache_key: String,
    eur_rates: HashMap<String, Decimal>,
}

static CACHE: LazyLock<Mutex<Option<CachedRates>>> = LazyLock::new(|| Mutex::new(None));

#[derive(Debug, Deserialize)]
struct FrankfurterRate {
    quote: String,
    rate: Decimal,
}

#[derive(Debug, Deserialize)]
struct FixerResponse {
    success: bool,
    rates: Option<HashMap<String, Decimal>>,
}

pub async fn get_rates(base: &str) -> AppResult<RatesResponse> {
    let env_key = std::env::var("FIXER_API_KEY")
        .ok()
        .filter(|key| !key.trim().is_empty());
    get_rates_with(base, "auto", env_key.as_deref()).await
}

/// Fetch rates using the provider and key configured by the user, falling
/// back to the environment defaults when no user settings exist.
pub async fn get_rates_for_user(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    base: &str,
) -> AppResult<RatesResponse> {
    if load_record(pool, user_id).await?.is_none() {
        return get_rates(base).await;
    }
    let (provider, key) = load_currency_settings(pool, user_id).await?;
    get_rates_with(base, &provider, key.as_deref()).await
}

pub async fn get_rates_with(
    base: &str,
    requested_provider: &str,
    fixer_key: Option<&str>,
) -> AppResult<RatesResponse> {
    let base = base.trim().to_uppercase();
    if base.is_empty() {
        return Err(AppError::BadRequest("Base currency is required".to_string()));
    }
    let env_key = std::env::var("FIXER_API_KEY")
        .ok()
        .filter(|key| !key.trim().is_empty());
    let provider = if requested_provider == PROVIDER_FIXER
        || (requested_provider == "auto" && fixer_key.or(env_key.as_deref()).is_some())
    {
        PROVIDER_FIXER
    } else {
        PROVIDER_FRANKFURTER
    };
    let key = if provider == PROVIDER_FIXER {
        fixer_key
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .map(str::to_string)
            .or(env_key)
    } else {
        None
    };
    let cache_key = format!("{provider}:{}", key.as_deref().unwrap_or(""));
    let cached = {
        let guard = CACHE.lock().map_err(|_| AppError::Internal("Rate cache poisoned".into()))?;
        guard
            .as_ref()
            .filter(|cached| {
                cached.cache_key == cache_key && cached.fetched_at.elapsed() < CACHE_TTL
            })
            .cloned()
    };
    let cached = match cached {
        Some(cached) => cached,
        None => {
            let mut fresh = match fetch_eur_rates(provider, key.as_deref()).await {
                Ok(fresh) => fresh,
                Err(error) => {
                    tracing::warn!("Exchange-rate provider unavailable: {error}");
                    CachedRates {
                        fetched_at: Instant::now(),
                        provider: "unavailable".to_string(),
                        cache_key: cache_key.clone(),
                        eur_rates: HashMap::from([("EUR".to_string(), Decimal::ONE)]),
                    }
                }
            };
            fresh.cache_key = cache_key;
            let mut guard = CACHE
                .lock()
                .map_err(|_| AppError::Internal("Rate cache poisoned".into()))?;
            *guard = Some(fresh.clone());
            fresh
        }
    };

    let base_rate = cached
        .eur_rates
        .get(&base)
        .copied()
        .unwrap_or(Decimal::ONE);
    let mut rates = HashMap::new();
    rates.insert(base.clone(), Decimal::ONE);
    for (currency, rate) in &cached.eur_rates {
        if *currency == base {
            continue;
        }
        if *rate > Decimal::ZERO && base_rate > Decimal::ZERO {
            rates.insert(currency.clone(), *rate / base_rate);
        }
    }

    Ok(RatesResponse {
        base,
        provider: cached.provider,
        fetched_at: Utc::now(),
        rates,
    })
}

async fn fetch_eur_rates(provider: &str, key: Option<&str>) -> AppResult<CachedRates> {
    if provider == PROVIDER_FIXER
        && let Some(key) = key
        && let Ok(rates) = fetch_fixer(key).await
    {
        return Ok(CachedRates {
            fetched_at: Instant::now(),
            provider: PROVIDER_FIXER.to_string(),
            cache_key: format!("{PROVIDER_FIXER}:{key}"),
            eur_rates: rates,
        });
    }
    let rates = fetch_frankfurter().await?;
    Ok(CachedRates {
        fetched_at: Instant::now(),
        provider: PROVIDER_FRANKFURTER.to_string(),
        cache_key: format!("{PROVIDER_FRANKFURTER}:"),
        eur_rates: rates,
    })
}

async fn fetch_frankfurter() -> AppResult<HashMap<String, Decimal>> {
    let rows = reqwest::Client::new()
        .get(FRANKFURTER_URL)
        .header("User-Agent", "recurrency")
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(|error| AppError::Internal(format!("Rate provider unavailable: {error}")))?
        .error_for_status()
        .map_err(|error| AppError::Internal(format!("Rate provider error: {error}")))?
        .json::<Vec<FrankfurterRate>>()
        .await
        .map_err(|error| AppError::Internal(format!("Invalid rate response: {error}")))?;

    let mut rates = HashMap::new();
    for row in rows {
        if row.rate > Decimal::ZERO {
            rates.insert(row.quote.to_uppercase(), row.rate);
        }
    }
    rates.insert("EUR".to_string(), Decimal::ONE);
    if rates.len() <= 1 {
        return Err(AppError::Internal("Rate provider returned no rates".into()));
    }
    Ok(rates)
}

async fn fetch_fixer(key: &str) -> AppResult<HashMap<String, Decimal>> {
    let response = reqwest::Client::new()
        .get(format!("{FIXER_URL}?access_key={key}&base=EUR"))
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(|error| AppError::Internal(format!("Fixer unavailable: {error}")))?
        .json::<FixerResponse>()
        .await
        .map_err(|error| AppError::Internal(format!("Invalid Fixer response: {error}")))?;
    if !response.success {
        return Err(AppError::Internal("Fixer rejected the API key".into()));
    }
    let mut rates = response
        .rates
        .ok_or_else(|| AppError::Internal("Fixer returned no rates".into()))?;
    rates.insert("EUR".to_string(), Decimal::ONE);
    Ok(rates)
}

#[derive(Debug, Clone, Serialize)]
pub struct CurrencySettingsResponse {
    pub provider: String,
    pub fixer_configured: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCurrencySettingsRequest {
    pub provider: Option<String>,
    #[serde(default)]
    pub fixer_api_key: Option<String>,
    #[serde(default)]
    pub clear_fixer_api_key: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct CurrencySettingsRecord {
    provider: String,
    fixer_api_key: Option<Vec<u8>>,
}

pub fn validate_provider(value: &str) -> bool {
    matches!(value, PROVIDER_FRANKFURTER | PROVIDER_FIXER)
}

fn encrypt_fixer_key(key: &str) -> AppResult<Vec<u8>> {
    let secret = crate::notifications::service::notification_secret()?;
    crate::notifications::crypto::encrypt_with_secret(&secret, &serde_json::json!(key.trim()))
        .map_err(AppError::Internal)
}

fn decrypt_fixer_key(encrypted: &[u8]) -> AppResult<String> {
    let secret = crate::notifications::service::notification_secret()?;
    let value = crate::notifications::crypto::decrypt_with_secret(&secret, encrypted)
        .map_err(AppError::Internal)?;
    value
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| AppError::Internal("Invalid Fixer API key".into()))
}

async fn load_record(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
) -> AppResult<Option<CurrencySettingsRecord>> {
    sqlx::query_as::<_, CurrencySettingsRecord>(
        "SELECT provider, fixer_api_key FROM currency_settings WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(Into::into)
}

async fn load_currency_settings(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
) -> AppResult<(String, Option<String>)> {
    let Some(record) = load_record(pool, user_id).await? else {
        return Ok((PROVIDER_FRANKFURTER.to_string(), None));
    };
    let key = match record.fixer_api_key {
        Some(bytes) => Some(decrypt_fixer_key(&bytes)?),
        None => None,
    };
    Ok((record.provider, key))
}

pub async fn get_currency_settings(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
) -> AppResult<CurrencySettingsResponse> {
    let (provider, key) = load_currency_settings(pool, user_id).await?;
    let env_key = std::env::var("FIXER_API_KEY").unwrap_or_default();
    Ok(CurrencySettingsResponse {
        provider,
        fixer_configured: key.is_some() || !env_key.trim().is_empty(),
    })
}

pub async fn update_currency_settings(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    req: UpdateCurrencySettingsRequest,
) -> AppResult<CurrencySettingsResponse> {
    let existing = load_record(pool, user_id).await?;
    let provider = req
        .provider
        .map(|value| value.trim().to_lowercase())
        .or_else(|| existing.as_ref().map(|row| row.provider.clone()))
        .unwrap_or_else(|| PROVIDER_FRANKFURTER.to_string());
    if !validate_provider(&provider) {
        return Err(AppError::BadRequest("Unknown exchange-rate provider".into()));
    }
    let key = if req.clear_fixer_api_key {
        None
    } else if let Some(key) = req.fixer_api_key {
        if key.trim().is_empty() {
            existing.as_ref().and_then(|row| row.fixer_api_key.clone())
        } else {
            Some(encrypt_fixer_key(&key)?)
        }
    } else {
        existing.as_ref().and_then(|row| row.fixer_api_key.clone())
    };
    if provider == PROVIDER_FIXER
        && key.is_none()
        && std::env::var("FIXER_API_KEY")
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
    {
        return Err(AppError::BadRequest(
            "A Fixer API key is required for this provider".into(),
        ));
    }
    sqlx::query(
        r#"
        INSERT INTO currency_settings (user_id, provider, fixer_api_key)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id) DO UPDATE SET
            provider = EXCLUDED.provider,
            fixer_api_key = EXCLUDED.fixer_api_key,
            updated_at = NOW()
        "#,
    )
    .bind(user_id)
    .bind(&provider)
    .bind(&key)
    .execute(pool)
    .await?;
    get_currency_settings(pool, user_id).await
}

/// Convert an amount between currencies using a response already based on `to`.
pub fn convert(amount: Decimal, from: &str, to: &str, rates: &RatesResponse) -> Decimal {
    if from.eq_ignore_ascii_case(to) {
        return amount;
    }
    let from = from.to_uppercase();
    let Some(rate) = rates.rates.get(&from) else {
        return amount;
    };
    if *rate <= Decimal::ZERO {
        return amount;
    }
    amount / *rate
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rates(base: &str, values: &[(&str, Decimal)]) -> RatesResponse {
        RatesResponse {
            base: base.to_string(),
            provider: "test".into(),
            fetched_at: Utc::now(),
            rates: values
                .iter()
                .map(|(code, value)| (code.to_string(), *value))
                .collect(),
        }
    }

    #[test]
    fn provider_validation() {
        assert!(validate_provider(PROVIDER_FRANKFURTER));
        assert!(validate_provider(PROVIDER_FIXER));
        assert!(!validate_provider("auto"));
    }

    #[test]
    fn same_currency_is_identity() {
        let rates = rates("EUR", &[]);
        assert_eq!(convert(Decimal::from(10), "EUR", "EUR", &rates), Decimal::from(10));
    }

    #[test]
    fn converts_using_base_rate() {
        let rates = rates("EUR", &[("USD", Decimal::new(11, 1))]);
        assert_eq!(convert(Decimal::from(11), "USD", "EUR", &rates), Decimal::from(10));
    }

    #[test]
    fn missing_rate_falls_back_to_original() {
        let rates = rates("EUR", &[]);
        assert_eq!(convert(Decimal::from(10), "GBP", "EUR", &rates), Decimal::from(10));
    }
}
