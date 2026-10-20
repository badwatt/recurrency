//! Local (offline) accounts.
//!
//! In offline mode there is no server, but the app still requires a login so a
//! session can exist and be logged out. Accounts are stored on-device and the
//! password is salted + stretched with SHA-256. This is local-only access
//! control (the data never leaves the device); it is not a replacement for the
//! server's Argon2-based auth.

use crate::models::auth::UserResponse;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

const ACCOUNTS_KEY: &str = "recurrency_local_accounts";
const HASH_ROUNDS: usize = 8192;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalAccount {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub currency: String,
    pub salt: String,
    pub password_hash: String,
}

impl LocalAccount {
    fn to_user(&self) -> UserResponse {
        UserResponse {
            id: self.id,
            email: self.email.clone(),
            name: Some(self.name.clone()),
            currency: self.currency.clone(),
            verified: true,
        }
    }
}

fn hash_password(salt: &str, password: &str) -> String {
    let mut digest = Sha256::digest(format!("{salt}:{password}").as_bytes());
    for _ in 0..HASH_ROUNDS {
        digest = Sha256::digest(digest);
    }
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn validate(email: &str, password: &str, name: &str) -> Result<(), String> {
    if !normalize_email(email).contains('@') {
        return Err("Enter a valid email".to_string());
    }
    if password.len() < 8 {
        return Err("Password must be at least 8 characters".to_string());
    }
    if name.trim().is_empty() {
        return Err("Name is required".to_string());
    }
    Ok(())
}

// ── Persistence ─────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn accounts_file() -> Option<std::path::PathBuf> {
    crate::state::auth::get_app_data_dir()
        .map(|d| d.join("recurrency").join("local_accounts.json"))
}

fn load_accounts() -> Vec<LocalAccount> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(json)) = storage.get_item(ACCOUNTS_KEY) {
                    return serde_json::from_str(&json).unwrap_or_default();
                }
            }
        }
        Vec::new()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        accounts_file()
            .and_then(|f| std::fs::read_to_string(f).ok())
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }
}

fn save_accounts(accounts: &[LocalAccount]) -> Result<(), String> {
    let json = serde_json::to_string(accounts).map_err(|e| e.to_string())?;
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                storage
                    .set_item(ACCOUNTS_KEY, &json)
                    .map_err(|e| format!("localStorage error: {e:?}"))?;
            }
        }
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let file = accounts_file().ok_or_else(|| "No data directory".to_string())?;
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(file, json).map_err(|e| e.to_string())
    }
}

/// Whether at least one local account exists on this device.
pub fn has_accounts() -> bool {
    !load_accounts().is_empty()
}

/// Create a local account and return its user. Fails if the email is taken.
pub fn register(email: &str, password: &str, name: &str) -> Result<UserResponse, String> {
    validate(email, password, name)?;
    let email = normalize_email(email);
    let mut accounts = load_accounts();
    if accounts.iter().any(|a| a.email == email) {
        return Err("An account with this email already exists".to_string());
    }
    let salt = Uuid::new_v4().to_string();
    let account = LocalAccount {
        id: Uuid::new_v4(),
        email,
        name: name.trim().to_string(),
        currency: "EUR".to_string(),
        password_hash: hash_password(&salt, password),
        salt,
    };
    let user = account.to_user();
    accounts.push(account);
    save_accounts(&accounts)?;
    Ok(user)
}

/// Authenticate against the on-device account store.
pub fn login(email: &str, password: &str) -> Result<UserResponse, String> {
    let email = normalize_email(email);
    let accounts = load_accounts();
    let account = accounts
        .iter()
        .find(|a| a.email == email)
        .ok_or_else(|| "Invalid email or password".to_string())?;
    if hash_password(&account.salt, password) != account.password_hash {
        return Err("Invalid email or password".to_string());
    }
    Ok(account.to_user())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic_and_salt_dependent() {
        let a = hash_password("salt1", "password123");
        let b = hash_password("salt1", "password123");
        let c = hash_password("salt2", "password123");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn hash_differs_for_wrong_password() {
        assert_ne!(
            hash_password("salt", "password123"),
            hash_password("salt", "password124")
        );
    }

    #[test]
    fn email_normalized() {
        assert_eq!(normalize_email("  User@Example.COM "), "user@example.com");
    }

    #[test]
    fn validation_rejects_bad_input() {
        assert!(validate("no-at-sign", "password123", "Name").is_err());
        assert!(validate("a@b.com", "short", "Name").is_err());
        assert!(validate("a@b.com", "password123", "   ").is_err());
        assert!(validate("a@b.com", "password123", "Name").is_ok());
    }

    #[test]
    fn account_to_user_maps_fields() {
        let acc = LocalAccount {
            id: Uuid::new_v4(),
            email: "a@b.com".into(),
            name: "Alice".into(),
            currency: "EUR".into(),
            salt: "s".into(),
            password_hash: "h".into(),
        };
        let user = acc.to_user();
        assert_eq!(user.email, "a@b.com");
        assert_eq!(user.name.as_deref(), Some("Alice"));
        assert!(user.verified);
    }
}
