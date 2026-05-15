//! Test environment variable helpers.
//!
//! In Rust 2024 edition, `env::set_var` and `env::remove_var` are unsafe.
//! This module provides safe wrappers for use in tests only.

/// Set an environment variable. Wrapper around `unsafe env::set_var` for edition 2024.
///
/// # Panics
/// Never panics. Internally calls `unsafe { std::env::set_var(key, value) }`.
#[cfg(test)]
pub fn set_env(key: &str, value: &str) {
    // SAFETY: Only used in single-threaded test contexts.
    unsafe { std::env::set_var(key, value); }
}

/// Remove an environment variable. Wrapper around `unsafe env::remove_var` for edition 2024.
///
/// # Panics
/// Never panics. Internally calls `unsafe { std::env::remove_var(key) }`.
#[cfg(test)]
pub fn remove_env(key: &str) {
    // SAFETY: Only used in single-threaded test contexts.
    unsafe { std::env::remove_var(key); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_remove_env() {
        set_env("TEST_HELPER_KEY", "test_value");
        assert_eq!(std::env::var("TEST_HELPER_KEY").unwrap(), "test_value");
        remove_env("TEST_HELPER_KEY");
        assert!(std::env::var("TEST_HELPER_KEY").is_err());
    }
}