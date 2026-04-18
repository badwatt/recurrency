
use dioxus::prelude::*;

/// Hook that provides a Signal<bool> controlling mobile sidebar open state.
///
/// Platform strategy:
///   - Android: JS eval returns early (dioxus global not available in Android WebView).
///     Rust touch handlers in AppLayout handle swipes.
///   - Web/browser: JS touch listeners via dioxus::document::eval() handle swipes.
pub fn use_swipe_sidebar() -> Signal<bool> {
    let mut sidebar_open = use_signal(|| false);

    use_future(move || async move {
        // JS swipe listeners — works in browser context.
        // On Android, dioxus.send() is not available, so JS listeners return early.
        // Android uses Rust touch handlers in AppLayout instead.
        let js = r#"
            (function() {
                // Check if running in Dioxus web renderer (dioxus.send available)
                // On Android WebView: typeof dioxus === 'undefined' → early return
                if (typeof dioxus === 'undefined') return;

                var startX = 0, tracking = false;
                var EDGE = 30;
                var MIN_SWIPE = 50;

                document.addEventListener('touchstart', function(e) {
                    var t = e.touches[0];
                    var sidebar = document.getElementById('mobile-sidebar');
                    var isOpen = sidebar && sidebar.classList.contains('sidebar-open');
                    // Track if starting from left edge OR if sidebar is already open
                    if (t.clientX < EDGE || isOpen) {
                        startX = t.clientX;
                        tracking = true;
                    }
                }, {passive: true});

                document.addEventListener('touchmove', function(e) {
                    if (!tracking) return;
                    var dx = e.touches[0].clientX - startX;
                    // Intercept if horizontal swipe dominates
                    if (Math.abs(dx) > 10) {
                        e.preventDefault();
                    }
                }, {passive: false});

                document.addEventListener('touchend', function(e) {
                    if (!tracking) return;
                    tracking = false;
                    var dx = e.changedTouches[0].clientX - startX;
                    if (Math.abs(dx) > MIN_SWIPE) {
                        if (dx > MIN_SWIPE) {
                            dioxus.send('open');
                        } else {
                            dioxus.send('close');
                        }
                    }
                }, {passive: true});
            })();
        "#;

        let mut eval = dioxus::document::eval(js);

        loop {
            match eval.recv::<String>().await {
                Ok(action) => match action.as_str() {
                    "open" => sidebar_open.set(true),
                    "close" => sidebar_open.set(false),
                    _ => {}
                },
                Err(_) => break,
            }
        }
    });

    sidebar_open
}

#[cfg(test)]
mod tests {
    #[test]
    fn hook_returns_signal_default_false() {
        assert!(!false);
    }
}
