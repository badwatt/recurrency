//! Navigation helpers that work identically in the browser and WebView.

/// Return the viewport to the top after a menu navigation. Route changes do
/// not reset scroll on their own, especially in the Android WebView.
pub fn scroll_to_top() {
    let _ = dioxus::document::eval("window.scrollTo(0, 0);");
}
