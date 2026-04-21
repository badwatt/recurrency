/// Tests for index.html — CSS variable setup and theme initialization.

#[test]
fn index_html_body_bgmap_matches_css_variables() {
    // index.html must define CSS variables for body background BEFORE tailwind.css loads.
    // Bug: <style> was AFTER <link> → tailwind.css body{background-color:hsl(var(--background))}
    // parsed with undefined vars → body defaulted to white on refresh.
    // Fix: inline <style> with :root/.dark/.mocha CSS vars BEFORE any stylesheet.
    // Note: <link> for tailwind.css removed — CSS loaded via asset!() in main.rs
    // (works on all platforms including Android).
    let html = include_str!("../../../assets/index.html");

    // <style> with CSS vars must be present and BEFORE any <link> stylesheet
    let style_pos = html.find("<style>").unwrap_or(usize::MAX);
    let link_pos = html.find("<link rel=\"stylesheet\"").unwrap_or(usize::MAX);
    // If no <link rel=stylesheet> exists (removed), just check <style> exists
    if link_pos != usize::MAX {
        assert!(
            style_pos < link_pos,
            "<style> with CSS vars must appear BEFORE <link> stylesheet."
        );
    } else {
        assert!(
            style_pos != usize::MAX,
            "<style> with CSS vars must exist in index.html."
        );
    }
    for selector in [":root", ".dark", ".mocha"] {
        assert!(
            html.contains(selector),
            "CSS variables for {} must be defined inline",
            selector
        );
    }
}

#[test]
fn index_html_sets_body_background_on_theme() {
    // Body background is set via tailwind.css: body{background-color:hsl(var(--background))}.
    // CSS variables must be defined before tailwind.css parses this rule.
    // Bug: <style> after <link> → vars undefined → body white on refresh.
    // Fix: inline <style> with vars before <link> → body gets correct themed bg.
    let html = include_str!("../../../assets/index.html");
    // Verify CSS variables are defined in <style> (which must come before <link>)
    assert!(
        html.contains("--background"),
        "inline <style> must define --background CSS variable"
    );
    // The theme script adds .dark/.mocha class to <html> before body renders
    assert!(
        html.contains("document.documentElement.classList"),
        "index.html must add theme class to <html> element"
    );
}

#[test]
fn index_html_applies_dark_class_to_html() {
    let html = include_str!("../../../assets/index.html");
    assert!(
        html.contains("document.documentElement.classList.add('dark')"),
        "index.html must add 'dark' class to <html> element"
    );
}

#[test]
fn index_html_applies_mocha_class_to_html() {
    let html = include_str!("../../../assets/index.html");
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
    let html = include_str!("../../../assets/index.html");
    // Script in body sets meta.content with dark/mocha/light hex values
    let body_section = html.split("<body>").nth(1).unwrap_or("");
    assert!(
        body_section.contains("meta.content"),
        "Body script must set meta[name=theme-color] content"
    );
    // Verify the hex values are correct
    assert!(
        body_section.contains("#020817"),
        "Dark mode meta theme-color must be #020817 (matches CSS --background)"
    );
    assert!(
        body_section.contains("#1e1e2e"),
        "Mocha mode meta theme-color must be #1e1e2e (matches CSS --background)"
    );
    assert!(
        body_section.contains("#ffffff"),
        "Light mode meta theme-color must be #ffffff (matches CSS --background)"
    );
}

#[test]
fn index_html_has_loading_spinner_before_wasm() {
    // Before WASM loads, there's a gap where only index.html is visible.
    // Must show a themed loading spinner so users see a themed page, not blank.
    let html = include_str!("../../../assets/index.html");
    let body_section = html.split("\x3cbody>").nth(1).unwrap_or("").to_lowercase();
    // Must have a spinner element (id="pre-wasm-loader" or class containing "loading")
    let has_spinner = body_section.contains("pre-wasm-loader")
        || body_section.contains("loading")
        || body_section.contains("spinner");
    assert!(
        has_spinner,
        "index.html must have a loading spinner in <body> that shows before WASM loads"
    );
    // Spinner must use CSS variables for theming (not hardcoded colors)
    let html_lower = html.to_lowercase();
    let has_css_vars = html_lower.contains("--primary")
        || html_lower.contains("--foreground")
        || html_lower.contains("--background");
    assert!(
        has_css_vars,
        "Loading spinner must use CSS variables (--primary, --foreground) for theme adaptation"
    );
}

#[test]
fn index_html_loader_is_backdrop_blur_overlay() {
    // Pre-WASM loader must be semi-transparent + backdrop-blur.
    // User intent: loader overlays the app (visible through blur), not opaque wall.
    let html = include_str!("../../../assets/index.html");
    let style_section = html
        .split("<style>")
        .nth(1)
        .unwrap_or("")
        .split("</style>")
        .next()
        .unwrap_or("");
    // Must have backdrop-filter: blur
    assert!(
        style_section.contains("backdrop-filter"),
        "#pre-wasm-loader must have backdrop-filter:blur"
    );
    // Must be SEMI-transparent (rgba), NOT opaque solid
    assert!(
        style_section.contains("rgba("),
        "#pre-wasm-loader must use rgba() for semi-transparent background"
    );
    // Must NOT be solid opaque white/black
    assert!(
        !style_section.contains("background-color: rgb(255,255,255)")
            && !style_section.contains("rgb(2,8,23)"),
        "Loader must NOT be opaque solid color"
    );
}

#[test]
fn index_html_has_inline_theme_variables() {
    // CSS variables for dark/mocha are defined in tailwind.css which loads async.
    // On refresh, before tailwind.css parses, var(--background) etc. are empty
    // for .dark/.mocha themes → spinner shows wrong colors (light theme fallback).
    // Bug: spinner colors wrong on refresh in dark/mocha mode.
    // Fix: inline fallback CSS variables for each theme in index.html <style>.
    let html = include_str!("../../../assets/index.html");
    let style_section = html
        .split("<style>")
        .nth(1)
        .unwrap_or("")
        .split("</style>")
        .next()
        .unwrap_or("");
    // Must have .dark and .mocha selectors defining the spinner's CSS variables
    assert!(
        style_section.contains(".dark"),
        "Inline <style> must define .dark selector with CSS variable overrides"
    );
    assert!(
        style_section.contains(".mocha"),
        "Inline <style> must define .mocha selector with CSS variable overrides"
    );
    // Each must define --background, --foreground, --primary, --muted
    for theme in [".dark", ".mocha"] {
        let block = style_section.split(theme).nth(1).unwrap_or("");
        let block_short: String = block.chars().take(500).collect();
        for var in ["--background", "--foreground", "--primary", "--muted"] {
            assert!(
                block_short.contains(var),
                "{} in inline <style> must define {} for spinner theming",
                theme,
                var
            );
        }
    }
}
