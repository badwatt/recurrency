/// Tests for index.html — CSS variable setup and theme initialization.

#[test]
fn index_html_body_bgmap_matches_css_variables() {
    // Theme CSS variables live in tailwind.src.css, compiled into tailwind.css,
    // loaded via asset!() with with_static_head(true) into <head> before body.
    // index.html still needs the inline script that adds .dark/.mocha class.
    let html = include_str!("../../../index.html");
    assert!(
        html.contains("document.documentElement.className") || html.contains("document.documentElement.classList"),
        "index.html must contain theme classList script"
    );
    for selector in [":root", ".dark", ".mocha"] {
        assert!(
            html.contains(selector) || {
                let css = include_str!("../../../assets/styles/tailwind.src.css");
                css.contains(selector)
            },
            "CSS variables for {} must be defined in tailwind.src.css or index.html",
            selector
        );
    }
}

#[test]
fn index_html_sets_body_background_on_theme() {
    // Body background is set in tailwind.css via body { @apply bg-background text-foreground }.
    // CSS variables are in tailwind.src.css; asset!() with with_static_head(true)
    // injects <link> in generated <head> before body renders → no white flash.
    let css = include_str!("../../../assets/styles/tailwind.src.css");
    assert!(
        css.contains("--background"),
        "tailwind.src.css must define --background CSS variable"
    );
    // The theme script adds .dark/.mocha class to <html> before body renders
    let html = include_str!("../../../index.html");
    assert!(
        html.contains("document.documentElement.className") || html.contains("document.documentElement.classList"),
        "index.html must add theme class to <html> element"
    );
}

#[test]
fn index_html_applies_dark_class_to_html() {
    let html = include_str!("../../../index.html");
    assert!(
        html.contains("'dark'") || html.contains("className"),
        "index.html must add 'dark' class to <html> element"
    );
}

#[test]
fn index_html_applies_mocha_class_to_html() {
    let html = include_str!("../../../index.html");
    assert!(
        html.contains("'mocha'") || html.contains("className"),
        "index.html must add 'mocha' class to <html> element"
    );
}

#[test]
fn index_html_meta_theme_color_matches_css_variables() {
    // meta[name=theme-color] must match the inline CSS variable --background hex values.
    // Bug: dark theme-color was #030712 but CSS --background hsl(222.2,84%,4.9%)=#020817.
    // Fix: body-end script sets meta.content to correct hex values per theme.
    let html = include_str!("../../../index.html");
    // Theme script runs in <head> before the loader paints.
    let head = html.split("\x3cbody").next().unwrap_or("");
    assert!(
        head.contains("meta.content"),
        "Body script must set meta[name=theme-color] content"
    );
    // Verify the hex values are correct
    assert!(
        head.contains("#0d0d11"),
        "Dark mode meta theme-color must be #0d0d11 (matches CSS --background)"
    );
    assert!(
        head.contains("#1e1e2e"),
        "Mocha mode meta theme-color must be #1e1e2e (matches CSS --background)"
    );
    assert!(
        head.contains("#FAF9F2"),
        "Light mode meta theme-color must be #FAF9F2 (matches CSS --background)"
    );
}

#[test]
fn index_html_forwards_theme_to_android_system_bars() {
    let html = include_str!("../../../index.html");
    assert!(
        html.contains("RecurrencyAndroid"),
        "index.html must use the Android system-bars bridge"
    );
    assert!(
        html.contains("setSystemBars"),
        "index.html must call setSystemBars with the theme color"
    );
    assert!(
        html.contains("cls === 'light'"),
        "system-bar icon contrast must follow the light theme"
    );
}

#[test]
fn tailwind_src_has_theme_and_loader_css() {
    let css = include_str!("../../../assets/styles/tailwind.src.css");
    for selector in [":root", ".dark", ".mocha"] {
        assert!(css.contains(selector), "tailwind.src.css must define {}", selector);
    }
    assert!(css.contains(".recurrency-loader"), "must style the canonical loader");
    assert!(css.contains(".recurrency-spinner"), "must style the canonical spinner");
    assert!(css.contains("@keyframes recurrency-spin"), "must define the canonical spin keyframe");
    assert!(!css.contains("#pre-wasm-loader"), "legacy pre-wasm loader must be gone");
    assert!(!css.contains("--loader-bg"), "legacy loader variables must be gone");
}

#[test]
fn index_html_has_loading_spinner_before_wasm() {
    let html = include_str!("../../../index.html");
    let body_section = html.split("\x3cbody").nth(1).unwrap_or("");
    assert!(body_section.contains("recurrency-loader"));
    assert!(body_section.contains("recurrency-spinner"));
    assert!(body_section.contains("recurrency-brand"));
}

#[test]
fn index_html_loader_styles_moved_to_tailwind_src() {
    let css = include_str!("../../../assets/styles/tailwind.src.css");
    assert!(!css.contains("backdrop-filter"));
    assert!(css.contains(".recurrency-loader"));
    assert!(css.contains(".recurrency-spinner"));
    assert!(css.contains("html.light .recurrency-loader"));
    assert!(css.contains("html.dark .recurrency-loader"));
    assert!(css.contains("html.mocha .recurrency-loader"));
}

#[test]
fn index_html_theme_variables_in_tailwind_src() {
    // CSS variables for dark/mocha moved from inline index.html <style> to tailwind.src.css.
    let css = include_str!("../../../assets/styles/tailwind.src.css");
    // Must have .dark and .mocha selectors defining the spinner's CSS variables
    assert!(
        css.contains(".dark"),
        "tailwind.src.css must define .dark selector with CSS variable overrides"
    );
    assert!(
        css.contains(".mocha"),
        "tailwind.src.css must define .mocha selector with CSS variable overrides"
    );
    // Each must define --background, --foreground, --primary, --muted
    for theme in [".dark", ".mocha"] {
        let block = css.split(theme).nth(1).unwrap_or("");
        let block_short: String = block.chars().take(500).collect();
        for var in ["--background", "--foreground", "--primary", "--muted"] {
            assert!(
                block_short.contains(var),
                "{} in tailwind.src.css must define {} for spinner theming",
                theme,
                var
            );
        }
    }
}

#[test]
fn index_html_body_must_not_have_hidden_class() {
    // Tailwind .hidden sets display: none, which would make the entire page
    // invisible forever since no JS removes the class after the 0d8bb26 fix.
    let html = include_str!("../../../index.html");
    let body_tag = html.split("\x3cbody").nth(1).unwrap_or("").split('\x3e').next().unwrap_or("");
    assert!(
        !body_tag.contains("hidden"),
        "<body> must NOT have class='hidden' — Tailwind display:none hides page permanently"
    );
}

#[test]
fn index_html_hides_main_until_wasm_renders() {
    let html = include_str!("../../../index.html");
    let style_section = html.split("\x3cstyle\x3e").nth(1).unwrap_or("").split("\x3c/style\x3e").next().unwrap_or("");
    assert!(style_section.contains("#main { visibility: hidden; }"));
    assert!(!style_section.contains("body { visibility: hidden; }"));
}

#[test]
fn index_html_pre_wasm_loader_is_sibling_of_main() {
    let html = include_str!("../../../index.html");
    let body_section = html.split("\x3cbody").nth(1).unwrap_or("");
    let loader = body_section.find("id=\"recurrency-loader\"").unwrap_or(usize::MAX);
    let main = body_section.find("id=\"main\"").unwrap_or(usize::MAX);
    assert!(loader < main, "loader must exist before the app root");
    assert!(body_section.contains("recurrency-spinner"));
}

#[test]
fn index_html_critical_theming_prevents_android_fouc() {
    let html = include_str!("../../../index.html");
    let style_section = html.split("\x3cstyle\x3e").nth(1).unwrap_or("").split("\x3c/style\x3e").next().unwrap_or("");
    assert!(style_section.contains("html.light { background-color: #FAF9F2;"));
    assert!(style_section.contains("html.dark { background-color: #0d0d11;"));
    assert!(style_section.contains("html.mocha { background-color: #1e1e2e;"));
    assert!(style_section.contains(".recurrency-loader"));
    assert!(style_section.contains("recurrency-spin"));
}

#[test]
fn loading_favicon_is_the_same_png_as_the_loaded_clock() {
    let html = include_str!("../../../index.html");
    assert!(
        html.contains("data:image/png;base64,"),
        "the pre-WASM favicon must use the clock PNG, not a different SVG mark"
    );
    assert!(
        !html.contains("image/svg+xml"),
        "there must not be a second, different loading favicon"
    );
}

#[test]
fn web_title_is_not_duplicated() {
    let html = include_str!("../../../index.html");
    assert!(html.contains("<title></title>"), "Dioxus appends the app title; the template must be empty");
    assert!(!html.contains("<title>Recurrency</title>"));
    let config = include_str!("../../../Dioxus.toml");
    assert!(config.contains("title = \"Recurrency\""));
}

#[test]
fn desktop_window_title_is_recurrency() {
    let source = include_str!("../../lib.rs");
    assert!(source.contains("WindowBuilder::new().with_title(\"Recurrency\")"));
}

#[test]
fn all_platforms_use_the_same_loader_markup() {
    let app = include_str!("../../layout/app_layout.rs");
    assert!(app.contains("recurrency-spinner"));
    let android = include_str!("../../../../tools/src/android.rs");
    assert!(android.contains("recurrency-spinner"));
    let desktop = include_str!("../../../../vendor/dioxus-desktop/src/assets/prod.index.html");
    assert!(desktop.contains("recurrency-spinner"));
}
