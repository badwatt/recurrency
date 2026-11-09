use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A budgeting account. Every subscription and transaction belongs to one.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateAccountRequest {
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
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
    fn account_roundtrip() {
        let account = sample_account();
        let json = serde_json::to_string(&account).unwrap();
        let back: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(account, back);
    }

    #[test]
    fn create_request_optional_fields() {
        let req = CreateAccountRequest {
            name: "Savings".into(),
            icon: None,
            color: None,
        };
        assert_eq!(req.name, "Savings");
        assert!(req.icon.is_none());
        assert!(req.color.is_none());
    }

    #[test]
    fn update_request_default_is_empty() {
        let req = UpdateAccountRequest::default();
        assert!(req.name.is_none());
        assert!(req.icon.is_none());
        assert!(req.color.is_none());
    }

    #[test]
    fn account_default_flag() {
        let account = sample_account();
        assert!(account.is_default);
        assert_eq!(account.name, "Main account");
    }
}
