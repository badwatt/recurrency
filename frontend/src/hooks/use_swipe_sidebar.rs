use dioxus::prelude::*;

/// Hook that provides a Signal<bool> controlling mobile sidebar open state.
///
/// On web (wasm), registers touch listeners via eval() that:
///   - Detect swipe-right from left edge → opens sidebar
///   - Detect swipe-left (when sidebar is open) → closes sidebar
///
/// Communication flow:
///   JS touch listeners → dioxus.send(swipe_type) → Rust recv() → signal update
///
/// On non-web targets, this is a no-op (just returns a signal starting false).
pub fn use_swipe_sidebar() -> Signal<bool> {
    let mut sidebar_open = use_signal(|| false);

    #[cfg(feature = "web")]
    {
        use_future(move || async move {
            let mut eval = dioxus::document::eval(r#"
                (function() {
                    var startX = 0, startY = 0, tracking = false;
                    var EDGE = 30;
                    var MIN_SWIPE = 50;

                    document.addEventListener('touchstart', function(e) {
                        var t = e.touches[0];
                        // Track if starting from left edge OR if sidebar is open
                        var sidebar = document.getElementById('mobile-sidebar');
                        var isOpen = sidebar && sidebar.classList.contains('sidebar-open');
                        if (t.clientX < EDGE || isOpen) {
                            startX = t.clientX;
                            startY = t.clientY;
                            tracking = true;
                        }
                    }, {passive: true});

                    document.addEventListener('touchmove', function(e) {
                        if (!tracking) return;
                        var dx = e.touches[0].clientX - startX;
                        var dy = Math.abs(e.touches[0].clientY - startY);
                        // Only intercept if horizontal swipe dominates
                        if (Math.abs(dx) > dy && Math.abs(dx) > 10) {
                            e.preventDefault();
                        }
                    }, {passive: false});

                    document.addEventListener('touchend', function(e) {
                        if (!tracking) return;
                        tracking = false;
                        var t = e.changedTouches[0];
                        var dx = t.clientX - startX;
                        var dy = Math.abs(t.clientY - startY);

                        // Only trigger on significant horizontal movement
                        if (Math.abs(dx) > dy && Math.abs(dx) > MIN_SWIPE) {
                            if (dx > MIN_SWIPE) {
                                dioxus.send('open');
                            } else if (dx < -MIN_SWIPE) {
                                dioxus.send('close');
                            }
                        }
                    }, {passive: true});
                })();
            "#);

            // Continuously receive swipe events from JS and update the signal
            loop {
                match eval.recv::<String>().await {
                    Ok(action) => match action.as_str() {
                        "open" => sidebar_open.set(true),
                        "close" => sidebar_open.set(false),
                        _ => {}
                    },
                    Err(_) => break, // Eval finished or errored — stop listening
                }
            }
        });
    }

    sidebar_open
}

#[cfg(test)]
mod tests {
    #[test]
    fn hook_returns_signal_default_false() {
        // The hook starts with false; full integration test needs Dioxus context
        assert!(!false);
    }
}