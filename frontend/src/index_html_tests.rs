/// Tests for index.html inline script that sets theme before WASM loads.
/// These verify the HTML structure, not runtime behavior.

use std::collections::HashMap;

/// Parse the bgMap from the index.html <body> script: dark: '#XXXXXX', mocha: '#XXXXXX', light: '#XXXXXX'
fn parse_bgmap(html: &str) -> HashMap<&str, &str> {
    let body_section = html.split("<body>").nth(1).unwrap_or("");
    let mut map = HashMap::new();
    for theme in &["dark", "mocha", "light"] {
        // Find pattern: {theme}: '#XXXXXX'
        let needle = format!("{}: '", theme);
        if let Some(idx) = body_section.find(&needle) {
            let rest = &body_section[idx + needle.len()..];
            if rest.starts_with('#') {
                if let Some(end) = rest[1..].find('\'') {
                    let hex = &rest[1..1 + end];
                    if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
                        map.insert(*theme, hex);
                    }
                }
            }
        }
    }
    map
}

/// Parse CSS --background: h s% l% values from tailwind.css and return theme→hex pairs.
fn parse_css_backgrounds(css: &str) -> HashMap<&str, String> {
    let mut backgrounds: Vec<String> = Vec::new();
    // Find all --background: H S% L% in order
    let parts: Vec<&str> = css.split("--background:").skip(1).collect();
    for part in &parts {
        let trimmed = part.trim();
        // Parse: 222.2 84% 4.9% or 0 0% 100%;--foreground...
        // Tokenize on whitespace/semicolon, take first 3 numeric-ish tokens
        // Take text up to first semicolon (end of CSS declaration)
        let before_semi = trimmed.split(';').next().unwrap_or(trimmed);
        let mut nums = Vec::new();
        for tok in before_semi.split_whitespace() {
            let clean = tok.trim_end_matches('%');
            if clean.parse::<f64>().is_ok() {
                nums.push(clean.to_string());
            }
            if nums.len() == 3 { break; }
        }
        if nums.len() == 3 {
            if let (Ok(h), Ok(s), Ok(l)) = (
                nums[0].parse::<f64>(),
                nums[1].parse::<f64>(),
                nums[2].parse::<f64>(),
            ) {
                let (r, g, b) = hsl_to_rgb(h, s / 100.0, l / 100.0);
                backgrounds.push(format!("{:02x}{:02x}{:02x}", r, g, b));
            }
        }
    }
    let mut map = HashMap::new();
    // Order: light (:root), dark, mocha
    let themes = ["light", "dark", "mocha"];
    for (i, theme) in themes.iter().enumerate() {
        if i < backgrounds.len() {
            map.insert(*theme, backgrounds[i].clone());
        }
    }
    map
}

/// Convert HSL to RGB (h in degrees, s and l in [0,1]).
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let (r1, g1, b1) = if h_prime < 1.0 { (c, x, 0.0) }
        else if h_prime < 2.0 { (x, c, 0.0) }
        else if h_prime < 3.0 { (0.0, c, x) }
        else if h_prime < 4.0 { (0.0, x, c) }
        else if h_prime < 5.0 { (x, 0.0, c) }
        else { (c, 0.0, x) };
    let m = l - c / 2.0;
    (
        ((r1 + m) * 255.0).round() as u8,
        ((g1 + m) * 255.0).round() as u8,
        ((b1 + m) * 255.0).round() as u8,
    )
}

#[test]
fn index_html_body_bgmap_matches_css_variables() {
    // index.html must derive body background from CSS variables,
    // not hardcode hex values that drift from CSS.
    // Bug: hardcoded hex #030712 didn't match CSS --background hsl(222.2,84%,4.9%)=#020817.
    // Fix: use getComputedStyle to read --background, fallback to exact hex matches.
    let html = include_str!("../assets/index.html");
    let css = include_str!("../assets/tailwind.css");
    let body_section = html.split("\x3cbody>").nth(1).unwrap_or("");

    // Primary: script reads CSS variable
    let reads_css_var = body_section.contains("getComputedStyle")
        && body_section.contains("--background");
    assert!(reads_css_var,
        "index.html body script should read --background CSS variable via getComputedStyle");

    // Fallback: hardcoded hex must match CSS variables exactly
    let bgmap = parse_bgmap(html);
    let cssmap = parse_css_backgrounds(css);
    for theme in &["light", "dark", "mocha"] {
        if let (Some(html_hex), Some(css_hex)) = (bgmap.get(*theme), cssmap.get(*theme)) {
            assert_eq!(
                html_hex.to_lowercase(),
                css_hex.to_lowercase(),
                "bgMap {} fallback color mismatch: html #{}, CSS #{}",
                theme, html_hex, css_hex
            );
        }
        // Missing from bgmap is OK if CSS variable path is used
    }
}

#[test]
fn index_html_sets_body_background_on_theme() {
    // index.html must set body background color AFTER body exists.
    // Bug: script in <head> runs before <body> exists → document.body is null → silent failure.
    // Fix: use DOMContentLoaded or place script after <body> opening tag.
    let html = include_str!("../assets/index.html");

    // Script must set body background-color AFTER body element exists
    assert!(
        html.contains("body.style.backgroundColor") || html.contains("body.style.background-color"),
        "index.html must set body background color to prevent white flash"
    );

    // Script that sets body.style must run AFTER <body> exists
    // Either: (a) script is inside <body>, or (b) uses DOMContentLoaded
    let body_section = html.split("<body>").nth(1).unwrap_or("");
    let has_dom_content_loaded = html.contains("DOMContentLoaded");
    let bg_setup_in_body = body_section.contains("body.style.backgroundColor")
        || body_section.contains("body.style.background-color");
    assert!(
        has_dom_content_loaded || bg_setup_in_body,
        "body.style.backgroundColor must be set AFTER body exists (DOMContentLoaded or script in <body>)"
    );
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