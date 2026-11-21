//! Environment access with test-scoped overrides.
//!
//! Rust 2024 makes `env::set_var` and `env::remove_var` unsafe because they
//! mutate process-global state. Tests use a thread-local override instead:
//! no process-global mutation and no `unsafe`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::env;

thread_local! {
    static OVERRIDES: RefCell<HashMap<String, Option<String>>> = RefCell::new(HashMap::new());
}

/// Read a variable, preferring a thread-local test override when present.
pub fn var(key: &str) -> Result<String, env::VarError> {
    let override_value = OVERRIDES.with(|overrides| overrides.borrow().get(key).cloned());
    match override_value {
        Some(Some(value)) => Ok(value),
        Some(None) => Err(env::VarError::NotPresent),
        None => env::var(key),
    }
}

/// Override a variable for the current test thread.
#[cfg(test)]
pub fn set_env(key: &str, value: &str) {
    OVERRIDES.with(|overrides| {
        overrides
            .borrow_mut()
            .insert(key.to_string(), Some(value.to_string()));
    });
}

/// Remove a variable for the current test thread.
#[cfg(test)]
pub fn remove_env(key: &str) {
    OVERRIDES.with(|overrides| {
        overrides.borrow_mut().insert(key.to_string(), None);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_remove_env() {
        set_env("TEST_HELPER_KEY", "test_value");
        assert_eq!(
            var("TEST_HELPER_KEY").expect("override must exist"),
            "test_value"
        );
        remove_env("TEST_HELPER_KEY");
        assert!(var("TEST_HELPER_KEY").is_err());
    }
}
