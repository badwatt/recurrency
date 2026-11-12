use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::currency::service::{RatesResponse, convert};
#[cfg(test)]
use crate::shared::date_utils::days_in_month;
use crate::subscriptions::models::Recurrence;

#[derive(Debug, Clone)]
pub struct SummaryAccount {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct SummarySubscription {
    pub account_id: Uuid,
    pub amount: Decimal,
    pub currency: String,
    pub frequency: String,
    pub schedule_day: Option<i16>,
    pub schedule_month: Option<i16>,
    pub recurrence: Recurrence,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub next_billing_date: Option<NaiveDate>,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct SummaryTransaction {
    pub account_id: Uuid,
    pub amount: Decimal,
    pub currency: String,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountSummary {
    pub name: String,
    pub budget: Decimal,
    pub pending: Decimal,
}

pub fn monthly_equivalent(amount: Decimal, frequency: &str) -> Decimal {
    match frequency {
        "daily" => amount * Decimal::from(30),
        "weekly" => amount * Decimal::from_str_exact("4.33").unwrap_or(Decimal::ZERO),
        "monthly" => amount,
        "quarterly" => amount / Decimal::from(3),
        "yearly" => amount / Decimal::from(12),
        _ => amount,
    }
}

#[cfg(test)]
fn add_months_with_day(date: NaiveDate, months: i32, day: i16) -> NaiveDate {
    let total = date.year() * 12 + date.month() as i32 - 1 + months;
    let year = total.div_euclid(12);
    let month = (total.rem_euclid(12) + 1) as u32;
    let day = (day.max(1) as u32).min(days_in_month(year, month));
    NaiveDate::from_ymd_opt(year, month, day).unwrap_or(date)
}

#[cfg(test)]
fn advance_scheduled(
    date: NaiveDate,
    frequency: &str,
    schedule_day: Option<i16>,
    schedule_month: Option<i16>,
) -> NaiveDate {
    match frequency {
        "daily" => date + chrono::Duration::days(1),
        "weekly" => date + chrono::Duration::weeks(1),
        "monthly" => add_months_with_day(date, 1, schedule_day.unwrap_or(date.day() as i16)),
        "quarterly" => add_months_with_day(date, 3, schedule_day.unwrap_or(date.day() as i16)),
        "yearly" => {
            let year = date.year() + 1;
            let month = schedule_month.unwrap_or(date.month() as i16) as u32;
            let day = schedule_day.unwrap_or(date.day() as i16);
            let day = (day.max(1) as u32).min(days_in_month(year, month));
            NaiveDate::from_ymd_opt(year, month, day).unwrap_or(date)
        }
        _ => date + chrono::Duration::days(30),
    }
}

#[cfg(test)]
fn billing_dates_in_month(
    next_billing_date: Option<NaiveDate>,
    frequency: &str,
    schedule_day: Option<i16>,
    schedule_month: Option<i16>,
    end_date: Option<NaiveDate>,
    year: i32,
    month: u32,
) -> Vec<NaiveDate> {
    let month_start = NaiveDate::from_ymd_opt(year, month, 1).expect("valid month start");
    let month_end = NaiveDate::from_ymd_opt(year, month, days_in_month(year, month))
        .expect("valid month end");
    let Some(mut date) = next_billing_date else {
        return Vec::new();
    };
    let mut guard = 0;
    while date < month_start && guard < 1000 {
        date = advance_scheduled(date, frequency, schedule_day, schedule_month);
        guard += 1;
    }
    let mut dates = Vec::new();
    while date <= month_end && guard < 2000 {
        if end_date.is_some_and(|end| date >= end) {
            break;
        }
        dates.push(date);
        date = advance_scheduled(date, frequency, schedule_day, schedule_month);
        guard += 1;
    }
    dates
}

pub fn build_account_summaries(
    accounts: &[SummaryAccount],
    subscriptions: &[SummarySubscription],
    transactions: &[SummaryTransaction],
    today: NaiveDate,
    rates: &RatesResponse,
) -> Vec<AccountSummary> {
    let mut plan: HashMap<Uuid, AccountSummary> = accounts
        .iter()
        .map(|account| {
            (
                account.id,
                AccountSummary {
                    name: account.name.clone(),
                    budget: Decimal::ZERO,
                    pending: Decimal::ZERO,
                },
            )
        })
        .collect();

    for subscription in subscriptions.iter().filter(|item| item.is_active) {
        let occurrences = crate::subscriptions::recurrence::occurrences_in_month(
            subscription.start_date,
            &subscription.recurrence,
            subscription.end_date,
            today.year(),
            today.month(),
            subscription.amount,
        );
        if occurrences.is_empty() {
            continue;
        }
        let entry = plan.entry(subscription.account_id).or_insert_with(|| AccountSummary {
            name: "No account".to_string(),
            budget: Decimal::ZERO,
            pending: Decimal::ZERO,
        });
        for (date, amount) in occurrences {
            let amount = convert(amount, &subscription.currency, &rates.base, rates);
            entry.budget += amount;
            if date >= today {
                entry.pending += amount;
            }
        }
    }

    for transaction in transactions.iter().filter(|item| {
        item.date.year() == today.year() && item.date.month() == today.month()
    }) {
        let entry = plan.entry(transaction.account_id).or_insert_with(|| AccountSummary {
            name: "No account".to_string(),
            budget: Decimal::ZERO,
            pending: Decimal::ZERO,
        });
        let amount = convert(transaction.amount, &transaction.currency, &rates.base, rates);
        entry.budget += amount;
        if transaction.date > today {
            entry.pending += amount;
        }
    }

    let mut result: Vec<AccountSummary> = plan.into_values().collect();
    result.sort_by(|a, b| b.budget.cmp(&a.budget).then_with(|| a.name.cmp(&b.name)));
    result
}

pub fn total_budget(accounts: &[AccountSummary]) -> Decimal {
    accounts.iter().map(|account| account.budget).sum()
}

pub fn total_pending(accounts: &[AccountSummary]) -> Decimal {
    accounts.iter().map(|account| account.pending).sum()
}

pub fn build_message(
    accounts: &[SummaryAccount],
    subscriptions: &[SummarySubscription],
    transactions: &[SummaryTransaction],
    today: NaiveDate,
    currency: &str,
    rates: &RatesResponse,
) -> String {
    let plan = build_account_summaries(accounts, subscriptions, transactions, today, rates);
    let budget = total_budget(&plan);
    let pending = total_pending(&plan);
    let month = today.format("%B %Y");
    let mut body = format!(
        "Recurrency — {month}\nTransfer this month: {currency} {budget:.2}\nPending from today: {currency} {pending:.2}"
    );
    let visible: Vec<&AccountSummary> = plan
        .iter()
        .filter(|account| account.budget > Decimal::ZERO || account.pending > Decimal::ZERO)
        .collect();
    if !visible.is_empty() {
        body.push('\n');
        for account in visible {
            body.push_str(&format!(
                "\n{}: {currency} {:.2} budget · {currency} {:.2} pending",
                account.name, account.budget, account.pending
            ));
        }
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rates() -> RatesResponse {
        RatesResponse {
            base: "EUR".to_string(),
            provider: "test".to_string(),
            fetched_at: chrono::Utc::now(),
            rates: HashMap::from([("EUR".to_string(), Decimal::ONE)]),
        }
    }

    fn account(id: Uuid, name: &str) -> SummaryAccount {
        SummaryAccount {
            id,
            name: name.to_string(),
        }
    }

    #[test]
    fn monthly_equivalent_weekly() {
        assert_eq!(
            monthly_equivalent(Decimal::from(10), "weekly"),
            Decimal::from_str_exact("43.30").unwrap()
        );
    }

    #[test]
    fn monthly_equivalent_yearly() {
        assert_eq!(
            monthly_equivalent(Decimal::from(120), "yearly"),
            Decimal::from(10)
        );
    }

    #[test]
    fn summary_splits_budget_and_pending() {
        let account_id = Uuid::new_v4();
        let today = NaiveDate::from_ymd_opt(2026, 11, 10).unwrap();
        let accounts = vec![account(account_id, "Main")];
        let subscriptions = vec![
            SummarySubscription {
                account_id,
                amount: Decimal::from(15),
                currency: "EUR".to_string(),
                frequency: "monthly".to_string(),
                schedule_day: None,
                schedule_month: None,
                recurrence: crate::subscriptions::recurrence::recurrence_from_legacy(
                    "monthly",
                    NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                    Some(15),
                    None,
                ),
                start_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                end_date: None,
                next_billing_date: Some(NaiveDate::from_ymd_opt(2026, 11, 15).unwrap()),
                is_active: true,
            },
            SummarySubscription {
                account_id,
                amount: Decimal::from(10),
                currency: "EUR".to_string(),
                frequency: "monthly".to_string(),
                schedule_day: None,
                schedule_month: None,
                recurrence: crate::subscriptions::recurrence::recurrence_from_legacy(
                    "monthly",
                    NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
                    Some(5),
                    None,
                ),
                start_date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
                end_date: None,
                next_billing_date: Some(NaiveDate::from_ymd_opt(2026, 11, 5).unwrap()),
                is_active: true,
            },
        ];
        let transactions = vec![
            SummaryTransaction {
                account_id,
                amount: Decimal::from(7),
                currency: "EUR".to_string(),
                date: NaiveDate::from_ymd_opt(2026, 11, 10).unwrap(),
            },
            SummaryTransaction {
                account_id,
                amount: Decimal::from(3),
                currency: "EUR".to_string(),
                date: NaiveDate::from_ymd_opt(2026, 11, 20).unwrap(),
            },
        ];
        let summaries = build_account_summaries(&accounts, &subscriptions, &transactions, today, &test_rates());
        assert_eq!(summaries[0].budget, Decimal::from(35));
        assert_eq!(summaries[0].pending, Decimal::from(18));
        assert_eq!(total_budget(&summaries), Decimal::from(35));
        assert_eq!(total_pending(&summaries), Decimal::from(18));
    }

    #[test]
    fn message_contains_month_and_totals() {
        let account_id = Uuid::new_v4();
        let today = NaiveDate::from_ymd_opt(2026, 11, 10).unwrap();
        let message = build_message(
            &[account(account_id, "Main")],
            &[SummarySubscription {
                account_id,
                amount: Decimal::from(15),
                currency: "EUR".to_string(),
                frequency: "monthly".to_string(),
                schedule_day: None,
                schedule_month: None,
                recurrence: crate::subscriptions::recurrence::recurrence_from_legacy(
                    "monthly",
                    NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                    Some(15),
                    None,
                ),
                start_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                end_date: None,
                next_billing_date: Some(NaiveDate::from_ymd_opt(2026, 11, 15).unwrap()),
                is_active: true,
            }],
            &[],
            today,
            "EUR",
            &test_rates(),
        );
        assert!(message.contains("November 2026"));
        assert!(message.contains("Transfer this month: EUR 15.00"));
        assert!(message.contains("Pending from today: EUR 15.00"));
        assert!(message.contains("Main: EUR 15.00 budget"));
    }

    #[test]
    fn inactive_subscription_is_ignored() {
        let account_id = Uuid::new_v4();
        let today = NaiveDate::from_ymd_opt(2026, 11, 10).unwrap();
        let summaries = build_account_summaries(
            &[account(account_id, "Main")],
            &[SummarySubscription {
                account_id,
                amount: Decimal::from(15),
                currency: "EUR".to_string(),
                frequency: "monthly".to_string(),
                schedule_day: None,
                schedule_month: None,
                recurrence: crate::subscriptions::recurrence::recurrence_from_legacy(
                    "monthly",
                    NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                    Some(15),
                    None,
                ),
                start_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                end_date: None,
                next_billing_date: Some(NaiveDate::from_ymd_opt(2026, 11, 15).unwrap()),
                is_active: false,
            }],
            &[],
            today,
            &test_rates(),
        );
        assert_eq!(summaries[0].budget, Decimal::ZERO);
        assert_eq!(summaries[0].pending, Decimal::ZERO);
    }
}
