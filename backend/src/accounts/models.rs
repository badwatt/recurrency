use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A budgeting account. Every subscription and transaction belongs to one.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    #[serde(default)]
    pub icon: serde_json::Value,
    #[serde(default)]
    pub color: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct SetDefaultAccountRequest {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccountResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Account> for AccountResponse {
    fn from(account: Account) -> Self {
        Self {
            id: account.id,
            user_id: account.user_id,
            name: account.name,
            icon: account.icon,
            color: account.color,
            is_default: account.is_default,
            created_at: account.created_at,
            updated_at: account.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountWithCounts {
    pub account: AccountResponse,
    pub subscription_count: i64,
    pub transaction_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_account() -> Account {
        Account {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Main account".into(),
            icon: Some("Wallet".into()),
            color: Some("#6366f1".into()),
            is_default: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn account_response_preserves_fields() {
        let account = sample_account();
        let id = account.id;
        let response: AccountResponse = account.into();
        assert_eq!(response.id, id);
        assert_eq!(response.name, "Main account");
        assert_eq!(response.icon.as_deref(), Some("Wallet"));
        assert_eq!(response.color.as_deref(), Some("#6366f1"));
        assert!(response.is_default);
    }

    #[test]
    fn create_request_accepts_optional_fields() {
        let req: CreateAccountRequest =
            serde_json::from_str(r##"{"name":"Savings","icon":"Wallet","color":"#fff"}"##).unwrap();
        assert_eq!(req.name, "Savings");
        assert_eq!(req.icon.as_deref(), Some("Wallet"));
        assert_eq!(req.color.as_deref(), Some("#fff"));
    }

    #[test]
    fn create_request_minimal() {
        let req: CreateAccountRequest = serde_json::from_str(r#"{"name":"Cash"}"#).unwrap();
        assert_eq!(req.name, "Cash");
        assert!(req.icon.is_none());
        assert!(req.color.is_none());
    }

    #[test]
    fn update_request_defaults_to_json_null() {
        let req: UpdateAccountRequest = serde_json::from_str("{}").unwrap();
        assert!(req.name.is_none());
        assert!(req.icon.is_null());
        assert!(req.color.is_null());
    }

    #[test]
    fn update_request_partial_fields() {
        let req: UpdateAccountRequest =
            serde_json::from_str(r##"{"name":"Renamed","color":"#123456"}"##).unwrap();
        assert_eq!(req.name.as_deref(), Some("Renamed"));
        assert_eq!(req.color.as_str(), Some("#123456"));
    }

    #[test]
    fn set_default_request_parses_uuid() {
        let id = Uuid::new_v4();
        let req: SetDefaultAccountRequest =
            serde_json::from_str(&format!(r#"{{"id":"{id}"}}"#)).unwrap();
        assert_eq!(req.id, id);
    }

    #[test]
    fn set_default_request_rejects_invalid_uuid() {
        let result: Result<SetDefaultAccountRequest, _> =
            serde_json::from_str(r#"{"id":"not-a-uuid"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn account_with_counts_serializes() {
        let with_counts = AccountWithCounts {
            account: sample_account().into(),
            subscription_count: 3,
            transaction_count: 7,
        };
        let json = serde_json::to_value(&with_counts).unwrap();
        assert_eq!(json["subscription_count"], 3);
        assert_eq!(json["transaction_count"], 7);
        assert_eq!(json["account"]["name"], "Main account");
    }

    #[test]
    fn account_response_roundtrip() {
        let response: AccountResponse = sample_account().into();
        let json = serde_json::to_string(&response).unwrap();
        let back: AccountResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, back);
    }
}
