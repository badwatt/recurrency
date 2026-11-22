//! Empty list state with optional icon and action slot.

use nsh_base::theme;
use nsh_core::prelude::*;

pub fn empty_state(
    icon: ImageAsset,
    title: &'static str,
    description: &'static str,
    action: AnyElement,
) -> impl IntoElement {
    view()
        .w_full()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(14.0))
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .p(px(32.0))
        .child(
            image(icon)
                .size(px(28.0))
                .tint_color(move || theme().muted_foreground),
        )
        .child(
            view()
                .flex_col()
                .items_center()
                .gap(px(4.0))
                .child(
                    text(title)
                        .text_size(px(16.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text(description)
                        .text_size(px(13.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(action)
}
