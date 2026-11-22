//! Eye button that masks every amount in the app.

use nsh_base::{button, theme};
use nsh_core::prelude::*;

pub fn privacy_toggle(active: bool, on_press: impl Fn() + 'static) -> impl IntoElement {
    button()
        .items_center()
        .justify_center()
        .h(px(30.0))
        .w(px(30.0))
        .rounded(px(6.0))
        .accessibility_label("Toggle privacy mode")
        .on_press(on_press)
        .child(
            image(asset!("icons/eye.svg"))
                .size(px(17.0))
                .tint_color(move || {
                    if active {
                        theme().primary
                    } else {
                        theme().muted_foreground
                    }
                }),
        )
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_privacy_toggle_uses_the_shared_eye_icon() {
        let source = include_str!("privacy_toggle.rs");
        assert!(source.contains("icons/eye.svg"));
        assert!(source.contains("Toggle privacy mode"));
    }
}
