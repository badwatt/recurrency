//! Eye button that masks every amount in the app.

use nsh_base::{button, theme};
use nsh_core::prelude::*;

pub fn privacy_toggle(active: Signal<bool>, on_press: impl Fn() + 'static) -> impl IntoElement {
    button()
        .cursor_pointer()
        .items_center()
        .justify_center()
        .h(px(36.0))
        .w(px(36.0))
        .rounded(px(6.0))
        .accessibility_label("Hide amounts")
        .on_press(on_press)
        .child(
            when(
                move || active.get(),
                move || {
                    image(asset!("icons/eye-off.svg"))
                        .size(px(19.0))
                        .tint_color(move || theme().primary)
                        .into_element()
                },
            )
            .otherwise(move || {
                image(asset!("icons/eye.svg"))
                    .size(px(19.0))
                    .tint_color(move || theme().muted_foreground)
                    .into_element()
            }),
        )
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_privacy_toggle_uses_the_shared_eye_icon() {
        let source = include_str!("privacy_toggle.rs");
        assert!(source.contains("icons/eye.svg"));
        assert!(source.contains("Hide amounts"));
    }
}
