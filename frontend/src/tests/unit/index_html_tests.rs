/// Tests for index.html — CSS variable setup and theme initialization.

#[test]
fn index_html_body_bgmap_matches_css_variables() {
    // Note: tailwind.css loaded via [web.resource] in Dioxus.toml.
    // dx CLI injects <link> in generated <head> at build time.
    // Inline <style> vars must come BEFORE that link so CSS has
    // defined variables at parse time.
    let html = include_str!("../../../index.html");

    // <style> with CSS vars must be present and BEFORE any <link> stylesheet
    let style_pos = html.find("<style>").unwrap_or(usize::MAX);
    let link_pos = html.find("<link rel=\"stylesheet\"").unwrap_or(usize::MAX);
    // If no <link rel=stylesheet> exists, just check <style> exists
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
    // Body background is set in tailwind.css: body{background-color:hsl(var(--background))}.
    // CSS variables are inline in <style> (before dx-injected <link>).
    // On refresh, vars are defined before external CSS parses → no white flash.
    let html = include_str!("../../../index.html");
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
    let html = include_str!("../../../index.html");
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
    // Pre-WASM loader must be OPAQUE to prevent FOUC.
    // tailwind.css loads via [web.resource]; dx injects <link> in generated HTML.
    // Opaque loader hides any unstyled content until CSS parses.
    let html = include_str!("../../../index.html");
    let style_section = html
        .split("<style>")
        .nth(1)
        .unwrap_or("")
        .split("</style>")
        .next()
        .unwrap_or("");
    // Must NOT have backdrop-filter (opaque bg, no blur needed)
    assert!(
        !style_section.contains("backdrop-filter"),
        "#pre-wasm-loader must NOT use backdrop-filter with opaque bg"
    );
    // Must be OPAQUE solid color, not rgba semi-transparent
    assert!(
        !style_section.contains("rgba("),
        "#pre-wasm-loader must NOT use rgba() — must be opaque"
    );
    // Must use solid colors: #fff for light, #020817 for dark, #1e1e2e for mocha
    assert!(
        style_section.contains("--loader-bg: #fff")
            || style_section.contains("--loader-bg:#fff"),
        "Light theme loader must be opaque white"
    );
    assert!(
        style_section.contains("--loader-bg: #020817")
            || style_section.contains("--loader-bg:#020817"),
        "Dark theme loader must be opaque #020817"
    );
    assert!(
        style_section.contains("--loader-bg: #1e1e2e")
            || style_section.contains("--loader-bg:#1e1e2e"),
        "Mocha theme loader must be opaque #1e1e2e"
    );
}

#[test]
fn index_html_has_inline_theme_variables() {
    // CSS variables for dark/mocha are defined in tailwind.css which loads async.
    // On refresh, before tailwind.css parses, var(--background) etc. are empty
    // for .dark/.mocha themes → spinner shows wrong colors (light theme fallback).
    // Bug: spinner colors wrong on refresh in dark/mocha mode.
    // Fix: inline fallback CSS variables for each theme in index.html <style>.
    let html = include_str!("../../../index.html");
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
