//! Optional date field hidden behind a small add action.

use chrono::Local;
use nsh_base::{button, theme};
use nsh_core::prelude::*;

pub fn optional_date(
    label: &'static str,
    add_label: &'static str,
    value: Signal<String>,
) -> impl IntoElement {
    view().flex_grow(1.0).basis(px(0.0)).min_w(px(0.0)).child(
        when(
            move || value.get().trim().is_empty(),
            move || {
                button()
                    .cursor_pointer()
                    .w_full()
                    .h(px(40.0))
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .gap(px(6.0))
                    .rounded(px(8.0))
                    .border(px(1.0))
                    .border_color(move || theme().primary.opacity(0.3))
                    .bg(move || theme().primary.opacity(0.05))
                    .on_press(move || value.set(Local::now().date_naive().to_string()))
                    .child(
                        image(asset!("icons/plus.svg"))
                            .size(px(13.0))
                            .tint_color(move || theme().primary),
                    )
                    .child(
                        text(add_label)
                            .text_size(px(12.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(move || theme().primary),
                    )
                    .into_element()
            },
        )
        .otherwise(move || {
            view()
                .w_full()
                .flex_col()
                .gap(px(6.0))
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            text(label)
                                .text_size(px(13.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            button()
                                .cursor_pointer()
                                .h(px(24.0))
                                .px(px(6.0))
                                .rounded(px(4.0))
                                .on_press(move || value.set(String::new()))
                                .child(
                                    text("Clear")
                                        .text_size(px(11.0))
                                        .text_color(move || theme().muted_foreground),
                                ),
                        ),
                )
                .child(
                    view()
                        .w_full()
                        .child(crate::components::date_picker::date_picker(value)),
                )
                .into_element()
        }),
    )
}
