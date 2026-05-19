/// Tests for index.html — CSS variable setup and theme initialization.

#[test]
fn index_html_body_bgmap_matches_css_variables() {
    // Theme CSS variables live in tailwind.src.css, compiled into tailwind.css,
    // loaded via asset!() with with_static_head(true) into <head> before body.
    // index.html still needs the inline script that adds .dark/.mocha class.
    let html = include_str!("../../../index.html");
    assert!(
        html.contains("document.documentElement.classList"),
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
        html.contains("document.documentElement.classList"),
        "index.html must add theme class to <html> element"
    );
}

#[test]
fn index_html_applies_dark_class_to_html() {
    let html = include_str!("../../../index.html");
    assert!(
        html.contains("document.documentElement.classList.add('dark')"),
        "index.html must add 'dark' class to <html> element"
    );
}

#[test]
fn index_html_applies_mocha_class_to_html() {
    let html = include_str!("../../../index.html");
    assert!(
        html.contains("document.documentElement.classList.add('mocha')"),
        "index.html must add 'mocha' class to <html> element"
    );
}

#[test]
fn index_html_meta_theme_color_matches_css_variables() {
    // meta[name=theme-color] must match the inline CSS variable --background hex values.
    // Bug: dark theme-color was #030712 but CSS --background hsl(222.2,84%,4.9%)=#020817.
    // Fix: body-end script sets meta.content to correct hex values per theme.
    let html = include_str!("../../../index.html");
    // Script in body sets meta.content with dark/mocha/light hex values
    let body_section = html.split("\x3cbody").nth(1).unwrap_or("");
    assert!(
        body_section.contains("meta.content"),
        "Body script must set meta[name=theme-color] content"
    );
    // Verify the hex values are correct
    assert!(
        body_section.contains("#0d0d11"),
        "Dark mode meta theme-color must be #0d0d11 (matches CSS --background)"
    );
    assert!(
        body_section.contains("#1e1e2e"),
        "Mocha mode meta theme-color must be #1e1e2e (matches CSS --background)"
    );
    assert!(
        body_section.contains("#FAF9F2"),
        "Light mode meta theme-color must be #FAF9F2 (matches CSS --background)"
    );
}

#[test]
fn tailwind_src_has_theme_and_loader_css() {
    let css = include_str!("../../../assets/styles/tailwind.src.css");
    // Theme selectors
    for selector in [":root", ".dark", ".mocha"] {
        assert!(css.contains(selector), "tailwind.src.css must define {}", selector);
    }
    // Loader CSS variables
    assert!(css.contains("--loader-bg"), "tailwind.src.css must define --loader-bg");
    assert!(css.contains("--loader-fg"), "tailwind.src.css must define --loader-fg");
    // Loader element styles
    assert!(css.contains("#pre-wasm-loader"), "tailwind.src.css must style #pre-wasm-loader");
    assert!(css.contains(".rings"), "tailwind.src.css must define .rings");
    assert!(css.contains(".r1"), "tailwind.src.css must define .r1");
    assert!(css.contains(".r2"), "tailwind.src.css must define .r2");
    // Keyframes
    assert!(css.contains("@keyframes spin"), "tailwind.src.css must define @keyframes spin");
    assert!(css.contains("@keyframes pulse"), "tailwind.src.css must define @keyframes pulse");
    // Opaque loader colors
    assert!(
        css.contains("--loader-bg: #FAF9F2") || css.contains("--loader-bg:#FAF9F2"),
        "light --loader-bg must be opaque #FAF9F2"
    );
    assert!(
        css.contains("--loader-bg: #0d0d11") || css.contains("--loader-bg:#0d0d11"),
        "dark --loader-bg must be opaque #0d0d11"
    );
    assert!(
        css.contains("--loader-bg: #1e1e2e") || css.contains("--loader-bg:#1e1e2e"),
        "mocha --loader-bg must be opaque #1e1e2e"
    );
    // No backdrop-filter or rgba in loader styles
    assert!(
        !css.contains("backdrop-filter"),
        "tailwind.src.css must NOT use backdrop-filter"
    );
}

#[test]
fn index_html_has_loading_spinner_before_wasm() {
    // Before WASM loads, there's a gap where only index.html is visible.
    // Must show a themed loading spinner so users see a themed page, not blank.
    let html = include_str!("../../../index.html");
    let body_section = html.split("\x3cbody").nth(1).unwrap_or("");
    // Must have a spinner element (id="pre-wasm-loader" or class containing "loading")
    let has_spinner = body_section.contains("pre-wasm-loader")
        || body_section.contains("loading")
        || body_section.contains("spinner");
    assert!(
        has_spinner,
        "index.html must have a loading spinner in <body> that shows before WASM loads"
    );
    // Spinner CSS variables live in tailwind.src.css (loaded via asset!()).
    let css = include_str!("../../../assets/styles/tailwind.src.css").to_lowercase();
    let has_css_vars = css.contains("--primary")
        || css.contains("--foreground")
        || css.contains("--background");
    assert!(
        has_css_vars,
        "tailwind.src.css must define spinner CSS variables (--primary, --foreground)"
    );
}

#[test]
fn index_html_loader_styles_moved_to_tailwind_src() {
    // Pre-WASM loader CSS moved from inline index.html <style> to tailwind.src.css.
    // Verify tailwind.src.css defines opaque loader bg and no backdrop-filter.
    let css = include_str!("../../../assets/styles/tailwind.src.css");
    let style_section = css;
    assert!(
        !style_section.contains("backdrop-filter"),
        "#pre-wasm-loader must NOT use backdrop-filter with opaque bg"
    );
    assert!(
        !style_section.contains("rgba("),
        "#pre-wasm-loader must NOT use rgba() — must be opaque"
    );
    assert!(
        style_section.contains("--loader-bg: #FAF9F2")
            || style_section.contains("--loader-bg:#FAF9F2"),
        "Light theme loader must be opaque #FAF9F2"
    );
    assert!(
        style_section.contains("--loader-bg: #0d0d11")
            || style_section.contains("--loader-bg:#0d0d11"),
        "Dark theme loader must be opaque #0d0d11"
    );
    assert!(
        style_section.contains("--loader-bg: #1e1e2e")
            || style_section.contains("--loader-bg:#1e1e2e"),
        "Mocha theme loader must be opaque #1e1e2e"
    );
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
fn index_html_body_hidden_via_css_visibility() {
    // Body must be hidden via inline CSS visibility, not display:none class,
    // so Dioxus use_effect can reveal it by setting style.visibility='visible'.
    let html = include_str!("../../../index.html");
    let style_section = html.split("\x3cstyle\x3e").nth(1).unwrap_or("").split("\x3c/style\x3e").next().unwrap_or("");
    assert!(
        style_section.contains("body { visibility: hidden; }"),
        "index.html inline style must hide body with visibility:hidden"
    );
}
