use crate::theme::catalog::{THEMES, theme_by_id};

/// Default theme.
pub const DEFAULT_THEME: &str = "mocha";

/// Background colour for the native WebView and meta theme-color.
pub fn theme_bg_color(theme: &str) -> &'static str {
    theme_by_id(theme)
        .map(|definition| definition.background)
        .unwrap_or_else(|| THEMES[0].background)
}

/// Foreground colour used by the pre-WASM loader and native chrome.
pub fn theme_fg_color(theme: &str) -> &'static str {
    theme_by_id(theme)
        .map(|definition| definition.foreground)
        .unwrap_or_else(|| THEMES[0].foreground)
}

/// Decode the theme background into the RGB triple used by native windowing.
pub fn theme_rgb(theme: &str) -> (u8, u8, u8) {
    let hex = theme_bg_color(theme).trim_start_matches('#');
    let parse = |range| {
        hex.get(range)
            .and_then(|value| u8::from_str_radix(value, 16).ok())
            .unwrap_or(30)
    };
    (parse(0..2), parse(2..4), parse(4..6))
}

/// Resolve a theme name to the CSS class that carries its variables.
pub fn resolve_theme_class(theme: &str) -> &'static str {
    theme_by_id(theme)
        .map(|definition| definition.id)
        .unwrap_or(DEFAULT_THEME)
}

/// JS snippet that forwards the active theme to Android's system bars.
/// Guarded so web/desktop builds (no bridge) are unaffected.
pub(crate) fn system_bars_js(color: &str, is_light: bool) -> String {
    format!(
        "try{{if(window.RecurrencyAndroid&&window.RecurrencyAndroid.setSystemBars){{window.RecurrencyAndroid.setSystemBars('{color}',{is_light});}}}}catch(e){{}}"
    )
}

/// Apply a theme to <html>: CSS classes, native chrome colours and meta color.
pub(crate) fn apply_theme_class(theme: &str) {
    let definition = theme_by_id(theme).unwrap_or(&THEMES[0]);
    let color = definition.background;
    let foreground = definition.foreground;

    let remove_args = THEMES
        .iter()
        .map(|theme| format!("'{}'", theme.id))
        .collect::<Vec<_>>()
        .join(", ");

    let class_args = if definition.is_dark {
        if definition.id == "dark" {
            "'dark'".to_string()
        } else {
            format!("'{}', 'dark'", definition.id)
        }
    } else if definition.id == "light" {
        "'light'".to_string()
    } else {
        format!("'{}', 'light'", definition.id)
    };

    let bars = system_bars_js(color, !definition.is_dark);

    let js = format!(
        r#"
        (function() {{
            var all = [{remove_args}];
            document.documentElement.classList.remove.apply(document.documentElement.classList, all);
            document.documentElement.classList.add({class_args});
            document.documentElement.style.backgroundColor = '{color}';
            document.documentElement.style.color = '{foreground}';
            var meta = document.querySelector('meta[name="theme-color"]');
            if (meta) meta.content = '{color}';
            {bars}
        }})()
        "#,
        remove_args = remove_args,
        class_args = class_args,
        color = color,
        foreground = foreground,
        bars = bars,
    );
    let _ = dioxus::document::eval(&js);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_mocha() {
        assert_eq!(DEFAULT_THEME, "mocha");
    }

    #[test]
    fn resolve_legacy_and_new_classes() {
        assert_eq!(resolve_theme_class("light"), "light");
        assert_eq!(resolve_theme_class("dark"), "dark");
        assert_eq!(resolve_theme_class("mocha"), "mocha");
        assert_eq!(resolve_theme_class("mocha-bloom"), "mocha-bloom");
        assert_eq!(resolve_theme_class("tokyo-night"), "tokyo-night");
        assert_eq!(resolve_theme_class("solarized-light"), "solarized-light");
        assert_eq!(resolve_theme_class("unknown"), "mocha");
    }

    #[test]
    fn bg_colors_match_the_catalogue() {
        assert_eq!(theme_bg_color("light"), "#FAF9F2");
        assert_eq!(theme_bg_color("dark"), "#0D0D11");
        assert_eq!(theme_bg_color("mocha"), "#1E1E2E");
        assert_eq!(theme_bg_color("mocha-bloom"), "#221A2E");
        assert_eq!(theme_bg_color("dracula"), "#282A36");
        assert_eq!(theme_bg_color("nord"), "#2E3440");
    }

    #[test]
    fn fg_colors_match_the_catalogue() {
        assert_eq!(theme_fg_color("latte"), "#4C4F69");
        assert_eq!(theme_fg_color("solarized-light"), "#586E75");
        assert_eq!(theme_fg_color("unknown"), "#CDD6F4");
    }

    #[test]
    fn every_catalogue_theme_has_a_background() {
        for theme in THEMES {
            assert_eq!(theme_bg_color(theme.id), theme.background);
            assert_eq!(theme_fg_color(theme.id), theme.foreground);
            assert_eq!(resolve_theme_class(theme.id), theme.id);
        }
    }

    #[test]
    fn theme_rgb_matches_the_background() {
        assert_eq!(theme_rgb("mocha"), (30, 30, 46));
        assert_eq!(theme_rgb("latte"), (239, 241, 245));
        assert_eq!(theme_rgb("unknown"), (30, 30, 46));
    }

    #[test]
    fn system_bars_js_forwards_color_and_contrast() {
        let light = system_bars_js("#FAF9F2", true);
        assert!(light.contains("RecurrencyAndroid.setSystemBars"));
        assert!(light.contains("'#FAF9F2'"));
        assert!(light.contains(",true"));

        let mocha = system_bars_js("#1E1E2E", false);
        assert!(mocha.contains("'#1E1E2E'"));
        assert!(mocha.contains(",false"));
    }
}
