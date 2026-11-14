//! Privacy mode: hides monetary amounts behind a blur.

use dioxus::prelude::*;

const STORAGE_KEY: &str = "recurrency-hide-amounts";
const FILE: &str = "hide_amounts.txt";

fn load() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        return web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item(STORAGE_KEY).ok().flatten())
            .as_deref()
            .is_some_and(is_enabled);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Some(dir) = crate::state::auth::get_app_data_dir() else {
            return false;
        };
        std::fs::read_to_string(dir.join("recurrency").join(FILE))
            .ok()
            .as_deref()
            .is_some_and(is_enabled)
    }
}

fn is_enabled(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

pub fn persist(enabled: bool) {
    let value = if enabled { "1" } else { "0" };
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(storage) =
            web_sys::window().and_then(|window| window.local_storage().ok().flatten())
        {
            let _ = storage.set_item(STORAGE_KEY, value);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(dir) = crate::state::auth::get_app_data_dir() {
            let app_dir = dir.join("recurrency");
            if std::fs::create_dir_all(&app_dir).is_ok() {
                let _ = std::fs::write(app_dir.join(FILE), value);
            }
        }
    }
}

pub static PRIVACY: GlobalSignal<bool> = Signal::global(load);

pub fn toggle() {
    let next = !*PRIVACY.read();
    *PRIVACY.write() = next;
    persist(next);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stored_values() {
        assert!(is_enabled("1"));
        assert!(is_enabled("true"));
        assert!(is_enabled("YES"));
        assert!(!is_enabled("0"));
        assert!(!is_enabled(""));
    }
}
