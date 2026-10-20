use crate::state::preferences::PREFERENCES;

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

/// JS snippet that forwards the active theme to Android's system bars.
/// Guarded so web/desktop builds (no bridge) are unaffected.
pub(crate) fn system_bars_js(color: &str, is_light: bool) -> String {
    format!(
        "try{{if(window.RecurrencyAndroid&&window.RecurrencyAndroid.setSystemBars){{window.RecurrencyAndroid.setSystemBars('{color}',{is_light});}}}}catch(e){{}}"
    )
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
    let bars = system_bars_js(color, cls == "light");

    let js = format!(
        r#"
        (function() {{
            document.documentElement.classList.remove('dark', 'mocha', 'light');
            document.documentElement.classList.add({class_args});
            var meta = document.querySelector('meta[name="theme-color"]');
            if (meta) meta.content = '{color}';
            {bars}
        }})()
        "#,
        class_args = class_args,
        color = color,
        bars = bars,
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

    #[test]
    fn system_bars_js_forwards_color_and_contrast() {
        let light = system_bars_js("#FAF9F2", true);
        assert!(light.contains("RecurrencyAndroid.setSystemBars"));
        assert!(light.contains("'#FAF9F2'"));
        assert!(light.contains(",true"));

        let mocha = system_bars_js("#1e1e2e", false);
        assert!(mocha.contains("'#1e1e2e'"));
        assert!(mocha.contains(",false"));
    }
}