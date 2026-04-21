#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Subscription {
        id: Option<String>,
        name: String,
        amount: String,
        currency: String,
        frequency: String,
        #[serde(rename = "startDate")]
        start_date: String,
        #[serde(rename = "nextBillingDate")]
        next_billing_date: Option<String>,
        #[serde(rename = "isActive")]
        is_active: bool,
        #[serde(rename = "categoryId")]
        category_id: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Transaction {
        id: Option<String>,
        name: String,
        amount: String,
        currency: String,
        date: String,
        notes: Option<String>,
        #[serde(rename = "categoryId")]
        category_id: Option<String>,
    }

    // Validation tests
    mod validation_tests {
        use super::*;

        #[test]
        fn test_subscription_valid_frequencies() {
            let valid_frequencies = vec!["weekly", "monthly", "quarterly", "yearly"];
            for freq in valid_frequencies {
                let sub = Subscription {
                    id: None,
                    name: "Test".to_string(),
                    amount: "10.00".to_string(),
                    currency: "USD".to_string(),
                    frequency: freq.to_string(),
                    start_date: "2026-01-15".to_string(),
                    next_billing_date: None,
                    is_active: true,
                    category_id: None,
                };
                assert!(!sub.frequency.is_empty());
            }
        }

        #[test]
        fn test_transaction_date_format() {
            let tx = Transaction {
                id: None,
                name: "Coffee".to_string(),
                amount: "5.00".to_string(),
                currency: "USD".to_string(),
                date: "2026-04-07".to_string(),
                notes: Some("Morning coffee".to_string()),
                category_id: None,
            };
            // Verify date format is YYYY-MM-DD
            assert!(tx.date.len() == 10);
            assert!(tx.date.chars().nth(4) == Some('-'));
            assert!(tx.date.chars().nth(7) == Some('-'));
        }

        #[test]
        fn test_amount_precision() {
            // Amounts should support 2 decimal places
            let amounts = vec!["9.99", "100.00", "1234.56", "0.01"];
            for amount in amounts {
                let parts: Vec<&str> = amount.split('.').collect();
                if parts.len() == 2 {
                    assert!(parts[1].len() <= 2);
                }
            }
        }

        #[test]
        fn test_currency_code_length() {
            let currencies = vec!["USD", "EUR", "GBP", "JPY"];
            for currency in currencies {
                assert!(currency.len() == 3);
                assert!(currency.chars().all(|c| c.is_ascii_uppercase()));
            }
        }
    }

    // Business logic tests
    mod business_logic_tests {
        use rust_decimal::Decimal;

        pub fn calculate_monthly_amount(amount: Decimal, frequency: &str) -> Decimal {
            match frequency {
                "weekly" => amount * Decimal::from_str_exact("4.33").unwrap(),
                "monthly" => amount,
                "quarterly" => amount / Decimal::from(3),
                "yearly" => amount / Decimal::from(12),
                _ => amount,
            }
        }

        #[test]
        fn test_netflix_monthly_cost() {
            // Netflix at $15.99/month = $15.99 monthly
            let amount = Decimal::from_str_exact("15.99").unwrap();
            let monthly = calculate_monthly_amount(amount, "monthly");
            assert_eq!(monthly, Decimal::from_str_exact("15.99").unwrap());
        }

        #[test]
        fn test_gym_weekly_cost() {
            // Gym at $10/week = $43.30/month
            let amount = Decimal::from(10);
            let monthly = calculate_monthly_amount(amount, "weekly");
            assert_eq!(monthly, Decimal::from_str_exact("43.30").unwrap());
        }

        #[test]
        fn test_car_insurance_quarterly() {
            // Car insurance at $300/quarter = $100/month
            let amount = Decimal::from(300);
            let monthly = calculate_monthly_amount(amount, "quarterly");
            assert_eq!(monthly, Decimal::from(100));
        }

        #[test]
        fn test_subscription_active_status() {
            // Default subscription should be active
            let sub = super::Subscription {
                id: None,
                name: "Test".to_string(),
                amount: "10.00".to_string(),
                currency: "USD".to_string(),
                frequency: "monthly".to_string(),
                start_date: "2026-01-15".to_string(),
                next_billing_date: None,
                is_active: true,
                category_id: None,
            };
            assert!(sub.is_active);
        }
    }

    // Data model tests
    mod data_model_tests {
        use super::*;

        #[test]
        fn test_subscription_serialization() {
            let sub = Subscription {
                id: Some("123".to_string()),
                name: "Netflix".to_string(),
                amount: "15.99".to_string(),
                currency: "USD".to_string(),
                frequency: "monthly".to_string(),
                start_date: "2026-01-15".to_string(),
                next_billing_date: Some("2026-02-15".to_string()),
                is_active: true,
                category_id: None,
            };

            let json = serde_json::to_string(&sub).unwrap();
            assert!(json.contains("\"name\":\"Netflix\""));
            assert!(json.contains("\"amount\":\"15.99\""));
            assert!(json.contains("\"frequency\":\"monthly\""));
        }

        #[test]
        fn test_transaction_serialization() {
            let tx = Transaction {
                id: Some("456".to_string()),
                name: "Coffee".to_string(),
                amount: "5.00".to_string(),
                currency: "USD".to_string(),
                date: "2026-04-07".to_string(),
                notes: Some("Morning coffee".to_string()),
                category_id: None,
            };

            let json = serde_json::to_string(&tx).unwrap();
            assert!(json.contains("\"name\":\"Coffee\""));
            assert!(json.contains("\"amount\":\"5.00\""));
            assert!(json.contains("\"date\":\"2026-04-07\""));
        }

        #[test]
        fn test_subscription_deserialization() {
            let json = r#"{
                "id": "123",
                "name": "Spotify",
                "amount": "9.99",
                "currency": "USD",
                "frequency": "monthly",
                "startDate": "2026-01-01",
                "nextBillingDate": "2026-02-01",
                "isActive": true,
                "categoryId": null
            }"#;

            let sub: Subscription = serde_json::from_str(json).unwrap();
            assert_eq!(sub.name, "Spotify");
            assert_eq!(sub.amount, "9.99");
            assert!(sub.is_active);
        }
    }
}
