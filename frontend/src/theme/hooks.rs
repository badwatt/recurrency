use crate::state::preferences::PREFERENCES;

/// Available themes (cycle order: light → dark → mocha → light).
pub const THEMES: &[&str] = &["light", "dark", "mocha"];

/// Default theme.
pub const DEFAULT_THEME: &str = "mocha";

/// Background colors per theme class. Used for native WebView + meta theme-color.
pub fn theme_bg_color(cls: &str) -> &'static str {
    match cls {
        "light" => "#FAF9F2",
        "dark" => "#0d0d11",
        _ => "#1e1e2e", // mocha
    }
}

/// Cycle to next theme.
pub fn cycle_theme(current: &str) -> &'static str {
    match current {
        "light" => "dark",
        "dark" => "mocha",
        "mocha" => "light",
        _ => "light",
    }
}

/// Resolve theme name to CSS class.
pub fn resolve_theme_class(theme: &str) -> &'static str {
    match theme {
        "light" => "light",
        "dark" => "dark",
        "mocha" => "mocha",
        _ => "mocha",
    }
}

/// Apply theme class to <html>. Sets class + meta theme-color.
/// Mocha: class="mocha dark" (dark for Tailwind, mocha for CSS vars).
/// Dark:  class="dark"       (dark for Tailwind + CSS vars).
/// Light: class="light"      (no dark, light CSS vars override :root).
pub(crate) fn apply_theme_class(theme: &str) {
    let cls = resolve_theme_class(theme);
    let color = theme_bg_color(cls);

    // Build class list as separate JS arguments: 'mocha', 'dark' or just 'light'
    let class_args = if cls == "light" {
        format!("'{}'", cls)
    } else {
        format!("'{}', 'dark'", cls)
    };

    let js = format!(
        r#"
        (function() {{
            document.documentElement.classList.remove('dark', 'mocha', 'light');
            document.documentElement.classList.add({class_args});
            var meta = document.querySelector('meta[name="theme-color"]');
            if (meta) meta.content = '{color}';
        }})()
        "#,
        class_args = class_args,
        color = color,
    );
    let _ = dioxus::document::eval(&js);
}

/// Cycle theme and apply it.
pub fn toggle_theme() {
    let current = PREFERENCES().theme().to_string();
    let next = cycle_theme(&current);
    PREFERENCES.write().set_theme(next.to_string());
    apply_theme_class(next);
}

/// Get icon name for theme toggle.
pub fn theme_icon_name(theme: &str) -> &'static str {
    match theme {
        "dark" => "moon",
        "mocha" => "coffee",
        _ => "sun",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_order() {
        assert_eq!(cycle_theme("light"), "dark");
        assert_eq!(cycle_theme("dark"), "mocha");
        assert_eq!(cycle_theme("mocha"), "light");
    }

    #[test]
    fn resolve_classes() {
        assert_eq!(resolve_theme_class("light"), "light");
        assert_eq!(resolve_theme_class("dark"), "dark");
        assert_eq!(resolve_theme_class("mocha"), "mocha");
    }

    #[test]
    fn default_is_mocha() {
        assert_eq!(DEFAULT_THEME, "mocha");
    }

    #[test]
    fn bg_colors() {
        assert_eq!(theme_bg_color("light"), "#FAF9F2");
        assert_eq!(theme_bg_color("dark"), "#0d0d11");
        assert_eq!(theme_bg_color("mocha"), "#1e1e2e");
    }

    #[test]
    fn class_args_format() {
        let light_args = format!("'{}'", "light");
        assert_eq!(light_args, "'light'");

        let mocha_args = format!("'{}', '{}'", "mocha", "dark");
        assert_eq!(mocha_args, "'mocha', 'dark'");

        let dark_args = format!("'{}', '{}'", "dark", "dark");
        assert_eq!(dark_args, "'dark', 'dark'");
    }
}