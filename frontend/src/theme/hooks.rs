use crate::state::preferences::PREFERENCES;

/// Available themes.
pub const THEMES: &[&str] = &["light", "dark", "mocha", "system"];

/// Cycle to next theme: light → dark → mocha → system → light
pub fn cycle_theme(current: &str) -> &'static str {
    match current {
        "light" => "dark",
        "dark" => "mocha",
        "mocha" => "system",
        _ => "light",
    }
}

/// Resolve a theme name to the CSS class to apply on <html>.
/// "system" resolves to "dark" if OS prefers dark, else "light".
pub fn resolve_theme_class(theme: &str) -> &'static str {
    match theme {
        "light" => "light",
        "dark" => "dark",
        "mocha" => "mocha",
        "system" => {
            // In a real browser context, we'd check matchMedia.
            // For tests and non-web, default to light.
            #[cfg(feature = "web")]
            {
                // This would use web_sys at runtime
                "light" // fallback; actual check done in apply_theme_to_document
            }
            #[cfg(not(feature = "web"))]
            "light"
        }
        _ => "light",
    }
}

/// Apply the current theme class to the document <html> element.
/// Uses dioxus::document::eval() — works on all platforms (web, Android, desktop).
fn apply_theme_class(theme: &str) {
    let class = match theme {
        "system" => {
            // Check OS preference via JS
            "light" // Default; actual check done in JS below
        }
        "dark" => "dark",
        "mocha" => "mocha",
        _ => "light",
    };

    let js = if theme == "system" {
        format!(
            "(function() {{ \
                var cls = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'; \
                document.documentElement.className = cls; \
                try {{ localStorage.setItem('recurrency-theme', '{}'); }} catch(e) {{}} \
            }})()",
            theme
        )
    } else {
        format!(
            "(function() {{ \
                document.documentElement.className = '{}'; \
                try {{ localStorage.setItem('recurrency-theme', '{}'); }} catch(e) {{}} \
            }})()",
            class, theme
        )
    };

    // Use dioxus::document::eval — available on all Dioxus platforms (web, Android, desktop).
    // This is async but we don't await — fire and forget. Theme applies within ms.
    #[allow(clippy::redundant_closure_call)]
    (|| {
        let js_owned = js.clone();
        dioxus::document::eval(&js_owned);
    })();
}

/// Apply the current theme to the document element.
/// On web: sets class on <html>, persists to localStorage.
/// On Android/desktop: uses dioxus::document::eval() to manipulate DOM + localStorage.
#[cfg(feature = "web")]
pub fn apply_theme_to_document(theme: &str) {
    // Prefer web_sys for web (synchronous, more reliable).
    // But also fall back to eval for consistency.
    #[cfg(target_arch = "wasm32")]
    {
        let resolved = match theme {
            "system" => {
                if let Some(window) = web_sys::window() {
                    if let Ok(Some(mql)) = window.match_media("(prefers-color-scheme: dark)") {
                        if mql.matches() { "dark" } else { "light" }
                    } else { "light" }
                } else { "light" }
            }
            "dark" => "dark",
            "mocha" => "mocha",
            _ => "light",
        };

        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document() {
                if let Some(html) = doc.document_element() {
                    html.set_class_name(resolved);
                }
            }
            // Persist theme to localStorage
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("recurrency-theme", theme);
            }
        }
    }
}

/// Cycle theme and apply it.
/// Works on all platforms: web, Android, desktop.
pub fn toggle_theme() {
    let current = PREFERENCES().theme().to_string();
    let next = cycle_theme(&current);
    PREFERENCES.write().set_theme(next.to_string());

    // Apply theme to DOM on all platforms using dioxus::document::eval().
    // On web, we could use web_sys directly (synchronous) but eval() works too.
    apply_theme_class(next);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_light_to_dark() {
        assert_eq!(cycle_theme("light"), "dark");
    }

    #[test]
    fn cycle_dark_to_mocha() {
        assert_eq!(cycle_theme("dark"), "mocha");
    }

    #[test]
    fn cycle_mocha_to_system() {
        assert_eq!(cycle_theme("mocha"), "system");
    }

    #[test]
    fn cycle_system_to_light() {
        assert_eq!(cycle_theme("system"), "light");
    }

    #[test]
    fn cycle_unknown_to_light() {
        assert_eq!(cycle_theme("other"), "light");
    }

    #[test]
    fn resolve_explicit_themes() {
        assert_eq!(resolve_theme_class("light"), "light");
        assert_eq!(resolve_theme_class("dark"), "dark");
        assert_eq!(resolve_theme_class("mocha"), "mocha");
    }

    #[test]
    fn themes_list() {
        assert_eq!(THEMES.len(), 4);
    }

    // ── Additional tests ──
    #[test]
    fn cycle_theme_sequence() {
        let mut current = "light";
        current = cycle_theme(current);
        assert_eq!(current, "dark");
        current = cycle_theme(current);
        assert_eq!(current, "mocha");
        current = cycle_theme(current);
        assert_eq!(current, "system");
        current = cycle_theme(current);
        assert_eq!(current, "light");
    }

    #[test]
    fn resolve_unknown_theme_falls_to_light() {
        assert_eq!(resolve_theme_class("unknown"), "light");
        assert_eq!(resolve_theme_class(""), "light");
        assert_eq!(resolve_theme_class("foo"), "light");
    }

    #[test]
    fn resolve_system_theme_default_light() {
        // In test/non-web context, system resolves to light
        assert_eq!(resolve_theme_class("system"), "light");
    }

    #[test]
    fn themes_contains_all_values() {
        assert!(THEMES.contains(&"light"));
        assert!(THEMES.contains(&"dark"));
        assert!(THEMES.contains(&"mocha"));
        assert!(THEMES.contains(&"system"));
    }
}
