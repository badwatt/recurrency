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
    let base = base.trim().to_uppercase();
    if base.is_empty() {
        return Err(AppError::BadRequest("Base currency is required".to_string()));
    }
    let cached = {
        let guard = CACHE.lock().map_err(|_| AppError::Internal("Rate cache poisoned".into()))?;
        guard
            .as_ref()
            .filter(|cached| cached.fetched_at.elapsed() < CACHE_TTL)
            .cloned()
    };
    let cached = match cached {
        Some(cached) => cached,
        None => {
            let fresh = match fetch_eur_rates().await {
                Ok(fresh) => fresh,
                Err(error) => {
                    tracing::warn!("Exchange-rate provider unavailable: {error}");
                    CachedRates {
                        fetched_at: Instant::now(),
                        provider: "unavailable".to_string(),
                        eur_rates: HashMap::from([("EUR".to_string(), Decimal::ONE)]),
                    }
                }
            };
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

async fn fetch_eur_rates() -> AppResult<CachedRates> {
    if let Ok(key) = std::env::var("FIXER_API_KEY")
        && !key.trim().is_empty()
        && let Ok(rates) = fetch_fixer(key.trim()).await
    {
        return Ok(CachedRates {
            fetched_at: Instant::now(),
            provider: "fixer".to_string(),
            eur_rates: rates,
        });
    }
    let rates = fetch_frankfurter().await?;
    Ok(CachedRates {
        fetched_at: Instant::now(),
        provider: "frankfurter".to_string(),
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
