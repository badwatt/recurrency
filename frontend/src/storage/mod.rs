pub mod folder_sync;
pub mod local;
pub mod local_auth;
pub mod provider;

pub use provider::Storage;
#[cfg(test)]
pub(crate) static STORAGE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
