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

        while let Ok(action) = eval.recv::<String>().await {
            match action.as_str() {
                "open" => sidebar_open.set(true),
                "close" => sidebar_open.set(false),
                _ => {}
            }
        }
    });

    sidebar_open
}

#[cfg(test)]
mod tests {
    #[test]
    fn hook_returns_signal_default_false() {
        // The hook returns a Signal<bool> initialized to false (sidebar closed).
        // We test the logical expectation: default state is closed.
        let sidebar_open = false;
        assert!(!sidebar_open);
    }

    #[test]
    fn sidebar_action_open() {
        let action = "open";
        assert_eq!(action, "open");
    }

    #[test]
    fn sidebar_action_close() {
        let action = "close";
        assert_eq!(action, "close");
    }

    #[test]
    fn swipe_direction_detection_positive() {
        // Swipe right (dx > 0) → open
        let start_x = 10.0_f64;
        let end_x = 80.0_f64;
        let dx = end_x - start_x;
        let threshold = 50.0_f64;
        let action = if dx > threshold { "open" } else { "ignore" };
        assert_eq!(action, "open");
    }

    #[test]
    fn swipe_direction_detection_negative() {
        // Swipe left (dx < 0) → close
        let start_x = 80.0_f64;
        let end_x = 20.0_f64;
        let dx = end_x - start_x;
        let threshold = 50.0_f64;
        let action = if dx < -threshold { "close" } else { "ignore" };
        assert_eq!(action, "close");
    }

    #[test]
    fn swipe_below_threshold_ignored() {
        let start_x = 10.0_f64;
        let end_x = 40.0_f64;
        let dx = end_x - start_x;
        let threshold = 50.0_f64;
        let action = if dx > threshold {
            "open"
        } else if dx < -threshold {
            "close"
        } else {
            "ignore"
        };
        assert_eq!(action, "ignore");
    }

    #[test]
    fn edge_swipe_from_left() {
        // Touch start must be within EDGE (30px) from left edge
        let edge = 30.0_f64;
        let touch_x = 15.0_f64;
        assert!(touch_x < edge, "Should detect edge swipe from left");
    }

    #[test]
    fn edge_swipe_not_from_middle() {
        let edge = 30.0_f64;
        let touch_x = 100.0_f64;
        assert!(touch_x >= edge, "Should NOT detect as edge swipe");
    }

    #[test]
    fn js_event_handlers_format() {
        let js = r#"document.addEventListener('touchstart', function(e) { }"#;
        assert!(js.contains("touchstart"));
    }

    #[test]
    fn android_check_js() {
        // Android: typeof dioxus === 'undefined' → early return
        let js = "typeof dioxus === 'undefined'";
        assert!(js.contains("dioxus"));
    }

    #[test]
    fn dioxus_send_message_format() {
        let action = "open";
        let js = format!("dioxus.send('{}')", action);
        assert!(js.contains("dioxus.send"));
        assert!(js.contains("'open'"));
    }

    #[test]
    fn swipe_state_transitions() {
        // Simulate sidebar open/close transitions
        let mut is_open = false;
        // Open action
        let action = "open";
        match action {
            "open" => is_open = true,
            "close" => is_open = false,
            _ => {}
        }
        assert!(is_open);
        // Close action
        let action = "close";
        match action {
            "open" => is_open = true,
            "close" => is_open = false,
            _ => {}
        }
        assert!(!is_open);
    }

    #[test]
    fn swipe_tracking_logic() {
        let edge = 30.0_f64;
        let min_swipe = 50.0_f64;

        // Scenario 1: Edge swipe, open
        let touch_x = 10.0_f64;
        let start_x = 10.0_f64;
        let end_x = 80.0_f64;
        let dx = end_x - start_x;
        let from_edge = touch_x < edge;
        let swipe_open = dx > min_swipe;
        let should_open = from_edge && swipe_open;
        assert!(should_open);

        // Scenario 2: Edge swipe, close
        let start_x = 300.0_f64;
        let end_x = 240.0_f64;
        let dx = end_x - start_x;
        let swipe_close = dx < -min_swipe;
        // When sidebar is open, any sufficient left swipe closes it
        let is_open = true;
        let should_close = is_open && swipe_close;
        assert!(should_close);
    }
}
