use aes_gcm::aead::{Aead, AeadCore, Generate, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use serde_json::Value;
use sha2::{Digest, Sha256};

const NONCE_LEN: usize = 12;

type NotificationNonce = Nonce<<Aes256Gcm as AeadCore>::NonceSize>;

/// Derive the notification-config encryption key from the application secret.
/// Keeping this separate from the JWT key material would be preferable, but
/// avoids introducing another mandatory deployment secret.
pub fn derive_key(secret: &str) -> [u8; 32] {
    let digest = Sha256::digest(format!("recurrency:notifications:{secret}").as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest);
    key
}

pub fn encrypt_with_key(key: &[u8; 32], value: &Value) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| "Invalid notification encryption key".to_string())?;
    let nonce = NotificationNonce::generate();
    let plaintext = serde_json::to_vec(value).map_err(|_| "Invalid notification config".to_string())?;
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_ref())
        .map_err(|_| "Could not encrypt notification config".to_string())?;
    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

pub fn decrypt_with_key(key: &[u8; 32], data: &[u8]) -> Result<Value, String> {
    if data.len() <= NONCE_LEN {
        return Err("Invalid notification config".to_string());
    }
    let (nonce, ciphertext) = data.split_at(NONCE_LEN);
    let nonce: [u8; NONCE_LEN] = nonce
        .try_into()
        .map_err(|_| "Invalid notification config".to_string())?;
    let nonce = NotificationNonce::from(nonce);
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| "Invalid notification encryption key".to_string())?;
    let plaintext = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|_| "Could not decrypt notification config".to_string())?;
    serde_json::from_slice(&plaintext).map_err(|_| "Invalid notification config".to_string())
}

pub fn encrypt_with_secret(secret: &str, value: &Value) -> Result<Vec<u8>, String> {
    encrypt_with_key(&derive_key(secret), value)
}

pub fn decrypt_with_secret(secret: &str, data: &[u8]) -> Result<Value, String> {
    decrypt_with_key(&derive_key(secret), data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const KEY: [u8; 32] = [7u8; 32];

    #[test]
    fn roundtrip() {
        let value = json!({"token": "secret", "topic": "alerts"});
        let encrypted = encrypt_with_key(&KEY, &value).unwrap();
        assert_ne!(encrypted, serde_json::to_vec(&value).unwrap());
        let decrypted = decrypt_with_key(&KEY, &encrypted).unwrap();
        assert_eq!(decrypted, value);
    }

    #[test]
    fn different_key_fails() {
        let encrypted = encrypt_with_key(&KEY, &json!({"token": "secret"})).unwrap();
        assert!(decrypt_with_key(&[8u8; 32], &encrypted).is_err());
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let mut encrypted = encrypt_with_key(&KEY, &json!({"token": "secret"})).unwrap();
        let last = encrypted.len() - 1;
        encrypted[last] ^= 1;
        assert!(decrypt_with_key(&KEY, &encrypted).is_err());
    }

    #[test]
    fn short_input_fails() {
        assert!(decrypt_with_key(&KEY, b"short").is_err());
    }

    #[test]
    fn secret_roundtrip() {
        let encrypted = encrypt_with_secret("test-secret", &json!({"url": "https://example.com"})).unwrap();
        assert_eq!(
            decrypt_with_secret("test-secret", &encrypted).unwrap(),
            json!({"url": "https://example.com"})
        );
        assert!(decrypt_with_secret("other-secret", &encrypted).is_err());
    }
}
