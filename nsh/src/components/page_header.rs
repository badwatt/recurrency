//! Page title, description and the right-hand action slot.

use nsh_base::theme;
use nsh_core::prelude::*;

pub fn page_header(
    title: &'static str,
    subtitle: &'static str,
    actions: AnyElement,
    compact: Signal<bool>,
    collapse_actions: bool,
    action_width: f32,
) -> impl IntoElement {
    view()
        .w_full()
        .flex_row()
        .flex_wrap()
        .items_end()
        .justify_between()
        .gap(px(12.0))
        .pb(px(2.0))
        .child(
            view()
                .min_w(px(0.0))
                .mt(px(-10.0))
                .pb(px(10.0))
                .flex_col()
                .child(
                    text(title)
                        .h(move || if compact.get() { px(40.0) } else { px(48.0) })
                        .mb(px(-8.0))
                        .text_size(move || if compact.get() { px(36.0) } else { px(48.0) })
                        .line_height(move || if compact.get() { px(40.0) } else { px(48.0) })
                        .letter_spacing(px(-2.4))
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
                .mt(px(1.0))
                .mb(px(-3.0))
                .overflow_hidden()
                .w(move || {
                    if collapse_actions && compact.get() {
                        nsh_core::geometry::Length::from(px(0.0))
                    } else if compact.get() {
                        nsh_core::geometry::Length::from(nsh_core::geometry::Percentage(1.0))
                    } else {
                        nsh_core::geometry::Length::from(px(action_width))
                    }
                })
                .opacity(move || {
                    if collapse_actions && compact.get() {
                        0.0_f32
                    } else {
                        1.0_f32
                    }
                })
                .justify_end()
                .child(actions),
        )
}
