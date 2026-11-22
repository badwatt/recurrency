//! Pure dashboard calculations, kept away from the native view.

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{Account, Category, Rates, Subscription, Transaction};
use crate::recurrence::occurrences_in_month;

#[derive(Clone, Debug, PartialEq)]
pub struct AccountBudget {
    pub account_id: Uuid,
    pub name: String,
    pub color: String,
    pub subscriptions: Decimal,
    pub manual: Decimal,
    pub budget: Decimal,
    pub pending: Decimal,
    pub subscription_count: usize,
    pub manual_count: usize,
    pub pending_count: usize,
}

impl AccountBudget {
    pub fn has_movements(&self) -> bool {
        self.budget > Decimal::ZERO
            || self.pending > Decimal::ZERO
            || self.subscription_count > 0
            || self.manual_count > 0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CategoryBudget {
    pub category_id: Uuid,
    pub name: String,
    pub color: String,
    pub amount: Decimal,
    pub pending: Decimal,
    pub charge_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpcomingPayment {
    pub id: Uuid,
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub account_id: Uuid,
    pub account_name: String,
    pub account_color: String,
    pub date: NaiveDate,
    pub is_transaction: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DashboardData {
    pub accounts: Vec<AccountBudget>,
    pub categories: Vec<CategoryBudget>,
    pub upcoming: Vec<UpcomingPayment>,
    pub rates: Rates,
    pub active_subscriptions: usize,
}

pub fn build_account_budgets(
    accounts: &[Account],
    subscriptions: &[Subscription],
    transactions: &[Transaction],
    today: NaiveDate,
    rates: &Rates,
) -> Vec<AccountBudget> {
    let mut plan: HashMap<Uuid, AccountBudget> = accounts
        .iter()
        .map(|account| {
            (
                account.id,
                AccountBudget {
                    account_id: account.id,
                    name: account.name.clone(),
                    color: account.color.clone().unwrap_or_else(|| "#6b7280".to_string()),
                    subscriptions: Decimal::ZERO,
                    manual: Decimal::ZERO,
                    budget: Decimal::ZERO,
                    pending: Decimal::ZERO,
                    subscription_count: 0,
                    manual_count: 0,
                    pending_count: 0,
                },
            )
        })
        .collect();

    for subscription in subscriptions.iter().filter(|subscription| subscription.is_active) {
        let occurrences = occurrences_in_month(
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
        let entry = plan.entry(subscription.account_id).or_insert_with(|| AccountBudget {
            account_id: subscription.account_id,
            name: "No account".to_string(),
            color: "#6b7280".to_string(),
            subscriptions: Decimal::ZERO,
            manual: Decimal::ZERO,
            budget: Decimal::ZERO,
            pending: Decimal::ZERO,
            subscription_count: 0,
            manual_count: 0,
            pending_count: 0,
        });
        entry.subscription_count += 1;
        for (date, amount) in occurrences {
            let amount = rates.convert(amount, &subscription.currency);
            entry.subscriptions += amount;
            entry.budget += amount;
            if date >= today {
                entry.pending += amount;
                entry.pending_count += 1;
            }
        }
    }

    for transaction in transactions
        .iter()
        .filter(|transaction| transaction.date.year() == today.year() && transaction.date.month() == today.month())
    {
        let entry = plan.entry(transaction.account_id).or_insert_with(|| AccountBudget {
            account_id: transaction.account_id,
            name: "No account".to_string(),
            color: "#6b7280".to_string(),
            subscriptions: Decimal::ZERO,
            manual: Decimal::ZERO,
            budget: Decimal::ZERO,
            pending: Decimal::ZERO,
            subscription_count: 0,
            manual_count: 0,
            pending_count: 0,
        });
        let amount = rates.convert(transaction.amount, &transaction.currency);
        entry.manual += amount;
        entry.budget += amount;
        entry.manual_count += 1;
        if transaction.date > today {
            entry.pending += amount;
            entry.pending_count += 1;
        }
    }

    let mut result: Vec<AccountBudget> = plan
        .into_values()
        .filter(AccountBudget::has_movements)
        .collect();
    result.sort_by(|a, b| {
        b.budget
            .cmp(&a.budget)
            .then_with(|| b.pending.cmp(&a.pending))
            .then_with(|| a.name.cmp(&b.name))
    });
    result
}

pub fn build_category_budgets(
    categories: &[Category],
    subscriptions: &[Subscription],
    transactions: &[Transaction],
    today: NaiveDate,
    rates: &Rates,
) -> Vec<CategoryBudget> {
    let mut plan: HashMap<Uuid, CategoryBudget> = HashMap::new();
    let mut add = |category_id: Uuid, amount: Decimal, pending: bool| {
        let category = categories.iter().find(|category| category.id == category_id);
        let entry = plan.entry(category_id).or_insert_with(|| CategoryBudget {
            category_id,
            name: category.map_or_else(|| "Uncategorized".to_string(), |category| category.name.clone()),
            color: category
                .and_then(|category| category.color.clone())
                .unwrap_or_else(|| "#6b7280".to_string()),
            amount: Decimal::ZERO,
            pending: Decimal::ZERO,
            charge_count: 0,
        });
        entry.amount += amount;
        entry.charge_count += 1;
        if pending {
            entry.pending += amount;
        }
    };

    for subscription in subscriptions.iter().filter(|subscription| subscription.is_active) {
        let occurrences = occurrences_in_month(
            subscription.start_date,
            &subscription.recurrence,
            subscription.end_date,
            today.year(),
            today.month(),
            subscription.amount,
        );
        for (date, amount) in occurrences {
            add(
                subscription.category_id,
                rates.convert(amount, &subscription.currency),
                date >= today,
            );
        }
    }
    for transaction in transactions
        .iter()
        .filter(|transaction| transaction.date.year() == today.year() && transaction.date.month() == today.month())
    {
        add(
            transaction.category_id,
            rates.convert(transaction.amount, &transaction.currency),
            transaction.date > today,
        );
    }

    let mut result: Vec<CategoryBudget> = plan.into_values().collect();
    result.sort_by(|a, b| b.amount.cmp(&a.amount).then_with(|| a.name.cmp(&b.name)));
    result
}

pub fn upcoming_payments(
    subscriptions: &[Subscription],
    transactions: &[Transaction],
    accounts: &[Account],
    today: NaiveDate,
) -> Vec<UpcomingPayment> {
    let mut items = Vec::new();
    for subscription in subscriptions.iter().filter(|subscription| subscription.is_active) {
        if let Some(date) = subscription.next_billing_date {
            items.push(UpcomingPayment {
                id: subscription.id,
                name: subscription.name.clone(),
                amount: subscription.amount,
                currency: subscription.currency.clone(),
                account_id: subscription.account_id,
                account_name: account_name(accounts, subscription.account_id),
                account_color: account_color(accounts, subscription.account_id),
                date,
                is_transaction: false,
            });
        }
    }
    for transaction in transactions.iter().filter(|transaction| transaction.date >= today) {
        items.push(UpcomingPayment {
            id: transaction.id,
            name: transaction.name.clone(),
            amount: transaction.amount,
            currency: transaction.currency.clone(),
            account_id: transaction.account_id,
            account_name: account_name(accounts, transaction.account_id),
            account_color: account_color(accounts, transaction.account_id),
            date: transaction.date,
            is_transaction: true,
        });
    }
    items.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.name.cmp(&b.name)));
    items
}

fn account_name(accounts: &[Account], account_id: Uuid) -> String {
    accounts
        .iter()
        .find(|account| account.id == account_id)
        .map_or_else(|| "No account".to_string(), |account| account.name.clone())
}

fn account_color(accounts: &[Account], account_id: Uuid) -> String {
    accounts
        .iter()
        .find(|account| account.id == account_id)
        .and_then(|account| account.color.clone())
        .unwrap_or_else(|| "#6b7280".to_string())
}

pub fn total_budget(plan: &[AccountBudget]) -> Decimal {
    plan.iter().map(|entry| entry.budget).sum()
}

pub fn total_pending(plan: &[AccountBudget]) -> Decimal {
    plan.iter().map(|entry| entry.pending).sum()
}

pub fn pending_ratio(budget: Decimal, pending: Decimal) -> f64 {
    if budget <= Decimal::ZERO {
        return 0.0;
    }
    let ratio = pending / budget * Decimal::from(100);
    ratio
        .min(Decimal::from(100))
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Recurrence, AnnualDate};
    use chrono::NaiveDate;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid test date")
    }

    fn account(id: Uuid, name: &str) -> Account {
        Account {
            id,
            name: name.to_string(),
            icon: None,
            color: None,
            is_default: false,
        }
    }

    fn subscription(account_id: Uuid, amount: Decimal, next: NaiveDate) -> Subscription {
        Subscription {
            id: Uuid::new_v4(),
            account_id,
            category_id: Uuid::new_v4(),
            name: "Streaming".to_string(),
            amount,
            currency: "EUR".to_string(),
            recurrence: Recurrence {
                interval: 1,
                unit: "month".to_string(),
                days: vec![next.day() as u8],
                weekdays: Vec::new(),
                dates: Vec::<AnnualDate>::new(),
            },
            start_date: date(2026, 1, 1),
            next_billing_date: Some(next),
            end_date: None,
            is_active: true,
            notes: None,
            logo_url: None,
        }
    }

    #[test]
    fn budgets_split_paid_and_pending_charges() {
        let account_id = Uuid::new_v4();
        let accounts = vec![account(account_id, "Main")];
        let subscriptions = vec![subscription(account_id, Decimal::from(10), date(2026, 11, 25))];
        let budgets = build_account_budgets(
            &accounts,
            &subscriptions,
            &[],
            date(2026, 11, 22),
            &Rates::default(),
        );
        assert_eq!(budgets[0].budget, Decimal::from(10));
        assert_eq!(budgets[0].pending, Decimal::from(10));
    }

    #[test]
    fn empty_accounts_are_hidden_from_the_plan() {
        let budgets = build_account_budgets(
            &[account(Uuid::new_v4(), "Empty")],
            &[],
            &[],
            date(2026, 11, 22),
            &Rates::default(),
        );
        assert!(budgets.is_empty());
    }

    #[test]
    fn manual_payments_on_today_are_not_pending() {
        let account_id = Uuid::new_v4();
        let transaction = Transaction {
            id: Uuid::new_v4(),
            account_id,
            category_id: Uuid::new_v4(),
            name: "Coffee".to_string(),
            amount: Decimal::from(3),
            currency: "EUR".to_string(),
            date: date(2026, 11, 22),
            notes: None,
            logo_url: None,
        };
        let budgets = build_account_budgets(
            &[account(account_id, "Main")],
            &[],
            &[transaction],
            date(2026, 11, 22),
            &Rates::default(),
        );
        assert_eq!(budgets[0].budget, Decimal::from(3));
        assert_eq!(budgets[0].pending, Decimal::ZERO);
    }

    #[test]
    fn upcoming_merges_subscriptions_and_future_transactions() {
        let account_id = Uuid::new_v4();
        let subscription = subscription(account_id, Decimal::from(10), date(2026, 11, 25));
        let transaction = Transaction {
            id: Uuid::new_v4(),
            account_id,
            category_id: Uuid::new_v4(),
            name: "Rent".to_string(),
            amount: Decimal::from(900),
            currency: "EUR".to_string(),
            date: date(2026, 11, 23),
            notes: None,
            logo_url: None,
        };
        let upcoming = upcoming_payments(
            &[subscription],
            &[transaction],
            &[account(account_id, "Main")],
            date(2026, 11, 22),
        );
        assert_eq!(upcoming.len(), 2);
        assert_eq!(upcoming[0].name, "Rent");
    }

    #[test]
    fn pending_ratio_is_capped() {
        assert_eq!(pending_ratio(Decimal::from(10), Decimal::from(20)), 100.0);
        assert_eq!(pending_ratio(Decimal::ZERO, Decimal::from(10)), 0.0);
    }

    #[test]
    fn rates_convert_into_base_currency() {
        let rates = Rates {
            base: "EUR".to_string(),
            rates: HashMap::from([("USD".to_string(), Decimal::new(11, 1))]),
        };
        assert_eq!(rates.convert(Decimal::from(11), "USD"), Decimal::from(10));
    }
}
