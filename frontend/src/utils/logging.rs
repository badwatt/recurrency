//! Logging utility for Recurrency frontend.
//! Writes logs to logs/frontend.log (creates dir if missing).

#[cfg(not(target_arch = "wasm32"))]
use std::fs::OpenOptions;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;

#[cfg(not(target_arch = "wasm32"))]
static LOG_FILE: Mutex<Option<std::path::PathBuf>> = Mutex::new(None);

#[cfg(not(target_arch = "wasm32"))]
fn is_dev() -> bool {
    std::env::var("APP_ENV").unwrap_or_default() != "production"
}

#[cfg(not(target_arch = "wasm32"))]
fn get_log_path() -> std::path::PathBuf {
    let mut guard = LOG_FILE.lock().unwrap();
    if guard.is_none() {
        std::fs::create_dir_all("logs").ok();
        let path = std::path::PathBuf::from("logs/frontend.log");
        *guard = Some(path.clone());
    }
    guard.clone().unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
fn write_log(level: &str, msg: &str) {
    if !is_dev() {
        return;
    }
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(get_log_path())
    {
        use std::io::Write;
        writeln!(file, "[{}] {}", level, msg).ok();
    }
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! log_internal {
    ($level:expr, $($arg:tt)*) => {{
        let msg = format!($($arg)*);
        write_log($level, &msg);
    }};
}

#[cfg(target_arch = "wasm32")]
macro_rules! log_internal {
    ($level:expr, $($arg:tt)*) => {{}};
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => { log_internal!("INFO", $($arg)*); };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { log_internal!("ERROR", $($arg)*); };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => { log_internal!("DEBUG", $($arg)*); };
}
