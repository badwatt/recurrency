/// Tests for index.html — CSS variable setup and theme initialization.

#[test]
fn index_html_body_bgmap_matches_css_variables() {
    // index.html must define CSS variables for body background BEFORE tailwind.css loads.
    // Bug: <style> was AFTER <link> → tailwind.css body{background-color:hsl(var(--background))}
    // parsed with undefined vars → body defaulted to white on refresh.
    // Fix: inline <style> with :root/.dark/.mocha CSS vars BEFORE <link>.
    let html = include_str!("../assets/index.html");

    // <style> must come BEFORE <link> so vars are defined when tailwind.css parses
    let link_pos = html.find("<link rel=\"stylesheet\"").unwrap_or(usize::MAX);
    let style_pos = html.find("<style>").unwrap_or(usize::MAX);
    assert!(style_pos < link_pos, "<style> with CSS vars must appear BEFORE <link> to tailwind.css so body has themed bg.");
    for selector in [":root", ".dark", ".mocha"] {
        assert!(html.contains(selector),
            "CSS variables for {} must be defined inline", selector);
    }
}

#[test]
fn index_html_sets_body_background_on_theme() {
    // Body background is set via tailwind.css: body{background-color:hsl(var(--background))}.
    // CSS variables must be defined before tailwind.css parses this rule.
    // Bug: <style> after <link> → vars undefined → body white on refresh.
    // Fix: inline <style> with vars before <link> → body gets correct themed bg.
    let html = include_str!("../assets/index.html");
    // Verify CSS variables are defined in <style> (which must come before <link>)
    assert!(html.contains("--background"),
        "inline <style> must define --background CSS variable");
    // The theme script adds .dark/.mocha class to <html> before body renders
    assert!(html.contains("document.documentElement.classList"),
        "index.html must add theme class to <html> element");
}

#[test]
fn index_html_applies_dark_class_to_html() {
    let html = include_str!("../assets/index.html");
    assert!(
        html.contains("document.documentElement.classList.add('dark')"),
        "index.html must add 'dark' class to <html> element"
    );
}

#[test]
fn index_html_applies_mocha_class_to_html() {
    let html = include_str!("../assets/index.html");
    assert!(
        html.contains("document.documentElement.classList.add('mocha')"),
        "index.html must add 'mocha' class to <html> element"
    );
}

#[test]
fn index_html_meta_theme_color_matches_css_variables() {
    // meta[name=theme-color] must match the inline CSS variable --background hex values.
    // Bug: dark theme-color was #030712 but CSS --background hsl(222.2,84%,4.9%)=#020817.
    // Fix: inline <style> defines vars correctly, meta.theme-color uses correct hex values.
    let html = include_str!("../assets/index.html");
    // Find meta.content assignments in the <head> script
    let head_section = html.split("</head>").nth(0).unwrap_or("");
    let meta_line = head_section.lines()
        .find(|l| l.contains("meta.content"))
        .unwrap_or("");
    // Extract all #XXXXXX from meta.content line, order: dark, mocha, light
    let all_hex: Vec<String> = meta_line.split('#')
        .skip(1)
        .filter_map(|s| {
            let hex: String = s.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
            if hex.len() == 6 { Some(hex) } else { None }
        })
        .collect();
    assert!(all_hex.len() >= 3,
        "meta.content line must have 3 hex values (dark, mocha, light), found: {}",
        all_hex.len());
    // dark: #020817, mocha: #1e1e2e, light: #ffffff
    let expected = ["020817", "1e1e2e", "ffffff"];
    let themes = ["dark", "mocha", "light"];
    for (i, theme) in themes.iter().enumerate() {
        assert_eq!(all_hex[i].to_lowercase(), expected[i],
            "meta theme-color for {} mismatch: expected #{}, got #{}",
            theme, expected[i], all_hex[i]);
    }
}

#[test]
fn index_html_has_loading_spinner_before_wasm() {
    // Before WASM loads, there's a gap where only index.html is visible.
    // Must show a themed loading spinner so users see a themed page, not blank.
    let html = include_str!("../assets/index.html");
    let body_section = html.split("\x3cbody>").nth(1).unwrap_or("").to_lowercase();
    // Must have a spinner element (id="pre-wasm-loader" or class containing "loading")
    let has_spinner = body_section.contains("pre-wasm-loader") || body_section.contains("loading") || body_section.contains("spinner");
    assert!(has_spinner,
        "index.html must have a loading spinner in <body> that shows before WASM loads");
    // Spinner must use CSS variables for theming (not hardcoded colors)
    let html_lower = html.to_lowercase();
    let has_css_vars = html_lower.contains("--primary") || html_lower.contains("--foreground") || html_lower.contains("--background");
    assert!(has_css_vars,
        "Loading spinner must use CSS variables (--primary, --foreground) for theme adaptation");
}

#[test]
fn index_html_loader_is_backdrop_blur_overlay() {
    // Loading spinner must be a semi-transparent overlay with backdrop-blur.
    // Must use EXPLICIT rgba values (not hsl(var/alpha) which may be ambiguous
    // when --background contains spaces like "222.2 84% 4.9%").
    // Bug: bg-background/85 generates bg-color:hsl(var(--background)) with no alpha.
    let html = include_str!("../assets/index.html");
    let style_section = html.split("<style>").nth(1).unwrap_or("").split("</style>").next().unwrap_or("");
    let lower = style_section.to_lowercase();
    // Must have backdrop-filter: blur
    assert!(lower.contains("backdrop-filter") || lower.contains("backdrop-blur"),
        "#pre-wasm-loader style must include backdrop-filter:blur");
    // Must use EXPLICIT rgb()/rgba() values for background —
    // rgb() and hsl() with CSS variable + alpha may be ambiguous.
    let has_explicit_rgb = lower.contains("rgb(");
    assert!(has_explicit_rgb,
        "#pre-wasm-loader must use explicit rgb() for themed background");
}

#[test]
fn index_html_has_inline_theme_variables() {
    // CSS variables for dark/mocha are defined in tailwind.css which loads async.
    // On refresh, before tailwind.css parses, var(--background) etc. are empty
    // for .dark/.mocha themes → spinner shows wrong colors (light theme fallback).
    // Bug: spinner colors wrong on refresh in dark/mocha mode.
    // Fix: inline fallback CSS variables for each theme in index.html <style>.
    let html = include_str!("../assets/index.html");
    let style_section = html.split("<style>").nth(1).unwrap_or("").split("</style>").next().unwrap_or("");
    // Must have .dark and .mocha selectors defining the spinner's CSS variables
    assert!(style_section.contains(".dark"),
        "Inline <style> must define .dark selector with CSS variable overrides");
    assert!(style_section.contains(".mocha"),
        "Inline <style> must define .mocha selector with CSS variable overrides");
    // Each must define --background, --foreground, --primary, --muted
    for theme in [".dark", ".mocha"] {
        let block = style_section.split(theme).nth(1).unwrap_or("");
        let block_short: String = block.chars().take(500).collect();
        for var in ["--background", "--foreground", "--primary", "--muted"] {
            assert!(block_short.contains(var),
                "{} in inline <style> must define {} for spinner theming",
                theme, var);
        }
    }
}