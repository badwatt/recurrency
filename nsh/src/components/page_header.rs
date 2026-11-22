//! Page title, description and the right-hand action slot.

use nsh_base::theme;
use nsh_core::prelude::*;

pub fn page_header(title: &'static str, subtitle: &'static str, actions: AnyElement) -> impl IntoElement {
    view()
        .w_full()
        .flex_row()
        .flex_wrap()
        .items_end()
        .justify_between()
        .gap(px(16.0))
        .child(
            view()
                .min_w(px(0.0))
                .flex_col()
                .gap(px(4.0))
                .child(
                    text(title)
                        .text_size(px(48.0))
                        .line_height(px(48.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text(subtitle)
                        .text_size(px(16.0))
                        .line_height(px(24.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            view()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(px(8.0))
                .child(actions),
        )
}
