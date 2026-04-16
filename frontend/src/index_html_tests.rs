/// Tests for index.html inline script that sets theme before WASM loads.
/// These verify the HTML structure, not runtime behavior.

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