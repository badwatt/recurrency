use argon2::{Argon2, PasswordHasher};

use crate::shared::error::{AppError, AppResult};

pub fn hash_password(password: &str) -> AppResult<String> {
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes())
        .map_err(|e: argon2::password_hash::Error| AppError::Internal(e.to_string()))?
        .to_string();
    Ok(password_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::password_hash::PasswordVerifier;

    #[test]
    fn hash_and_verify_roundtrip() {
        let hash = hash_password("test123").unwrap();
        let parsed = argon2::PasswordHash::new(&hash).unwrap();
        Argon2::default()
            .verify_password("test123".as_bytes(), &parsed)
            .unwrap();
    }

    #[test]
    fn different_passwords_produce_different_hashes() {
        let h1 = hash_password("password1").unwrap();
        let h2 = hash_password("password2").unwrap();
        assert_ne!(h1, h2);
    }

    #[test]
    fn same_password_produces_different_hashes() {
        // Salts are random, so same password → different hash
        let h1 = hash_password("same").unwrap();
        let h2 = hash_password("same").unwrap();
        assert_ne!(h1, h2);
    }

    #[test]
    fn wrong_password_fails_verification() {
        let hash = hash_password("correct").unwrap();
        let parsed = argon2::PasswordHash::new(&hash).unwrap();
        let result = Argon2::default().verify_password("wrong".as_bytes(), &parsed);
        assert!(result.is_err());
    }

    #[test]
    fn hash_length_reasonable() {
        let hash = hash_password("test_pass_val").unwrap();
        // Argon2 hashes are typically ~97 characters
        assert!(hash.len() > 50);
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn hash_contains_argon2_identifier() {
        let hash = hash_password("test").unwrap();
        assert!(hash.contains("$argon2id$"));
    }
}
