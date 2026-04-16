/// Tests for index.html inline script that sets theme before WASM loads.
/// These verify the HTML structure, not runtime behavior.

#[test]
fn index_html_sets_body_background_on_theme() {
    // index.html inline script must set document.body.style.backgroundColor
    // based on theme BEFORE the WASM app loads, to prevent white flash on refresh.
    //
    // Before fix: only sets class on <html>, body background is white (default).
    // After fix: sets body.style.backgroundColor to match theme color.
    let html = include_str!("../assets/index.html");

    // Script must set body background-color for each theme
    assert!(
        html.contains("body.style.backgroundColor") || html.contains("body.style.background-color"),
        "index.html must set body background color in inline script to prevent white flash"
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