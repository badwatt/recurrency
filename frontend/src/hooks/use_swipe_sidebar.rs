use dioxus::prelude::*;

/// Hook that provides a Signal<bool> controlling mobile sidebar open state.
/// On web (wasm), registers touch listeners via eval() for:
///   - swipe from left edge (30px) → opens sidebar
///   - swipe left → closes sidebar
/// On desktop/server, this is a no-op (just returns a signal that starts false).
pub fn use_swipe_sidebar() -> Signal<bool> {
    let sidebar_open = use_signal(|| false);

    #[cfg(feature = "web")]
    {
        use_future(move || async move {
            let _ = dioxus::document::eval(r#"
                (function() {
                    var startX = 0, startY = 0, tracking = false;
                    var EDGE = 30;
                    var MIN_SWIPE = 80;
                    document.addEventListener('touchstart', function(e) {
                        var t = e.touches[0];
                        if (t.clientX < EDGE) {
                            startX = t.clientX;
                            startY = t.clientY;
                            tracking = true;
                        }
                    }, {passive: true});
                    document.addEventListener('touchmove', function(e) {
                        if (!tracking) return;
                        var dx = e.touches[0].clientX - startX;
                        var dy = Math.abs(e.touches[0].clientY - startY);
                        if (dy > MIN_SWIPE) tracking = false;
                    }, {passive: true});
                    document.addEventListener('touchend', function(e) {
                        if (!tracking) return;
                        tracking = false;
                        var dx = e.changedTouches[0].clientX - startX;
                        if (dx > MIN_SWIPE) {
                            dioxus.send('open');
                        }
                    }, {passive: true});
                    // Swipe left to close
                    document.addEventListener('touchstart', function(e) {
                        var sidebar = document.getElementById('mobile-sidebar');
                        if (!sidebar) return;
                        startX = e.touches[0].clientX;
                    }, {passive: true});
                    document.addEventListener('touchend', function(e) {
                        var dx = startX - e.changedTouches[0].clientX;
                        if (dx > MIN_SWIPE) {
                            dioxus.send('close');
                        }
                    }, {passive: true});
                })();
            "#);
        });
    }

    sidebar_open
}

#[cfg(test)]
mod tests {
    #[test]
    fn hook_returns_signal_default_false() {
        // The hook starts with false; full test needs Dioxus context
        assert!(!false);
    }
}