//! Desktop window preferences.
//!
//! Window decorations are chosen before the webview exists, so this setting is
//! read at launch and applied on the next start.

const FRAMELESS_FILE: &str = "frameless.txt";
/// Menu id used by the Linux native Window menu.
pub const MENU_ID: &str = "recurrency-frameless";

/// Linux desktop builds can hide window decorations for tiling WMs.
pub const fn available() -> bool {
    cfg!(all(target_os = "linux", not(target_arch = "wasm32")))
}

fn parse_flag(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

pub fn stored() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Some(dir) = crate::state::auth::get_app_data_dir() else {
            return false;
        };
        std::fs::read_to_string(dir.join("recurrency").join(FRAMELESS_FILE))
            .map(|value| parse_flag(&value))
            .unwrap_or(false)
    }
    #[cfg(target_arch = "wasm32")]
    {
        false
    }
}

pub fn set_stored(enabled: bool) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Some(dir) = crate::state::auth::get_app_data_dir() else {
            return;
        };
        let app_dir = dir.join("recurrency");
        if std::fs::create_dir_all(&app_dir).is_ok() {
            let _ = std::fs::write(
                app_dir.join(FRAMELESS_FILE),
                if enabled { "1" } else { "0" },
            );
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = enabled;
    }
}

/// Whether the next launch should start frameless.
pub fn enabled() -> bool {
    if !available() {
        return false;
    }
    stored()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_parsing_is_boolean_like() {
        assert!(parse_flag("1"));
        assert!(parse_flag("true"));
        assert!(parse_flag("YES"));
        assert!(parse_flag(" on "));
        assert!(!parse_flag("0"));
        assert!(!parse_flag(""));
    }

    #[test]
    fn availability_matches_linux_desktop() {
        assert_eq!(
            available(),
            cfg!(all(target_os = "linux", not(target_arch = "wasm32")))
        );
    }
}
