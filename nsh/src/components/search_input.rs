//! Search field with magnifier and clear affordance.

use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
use nsh_core::prelude::*;

pub fn search_input(value: Signal<String>, placeholder: &'static str) -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .h(px(40.0))
        .px(px(12.0))
        .rounded(px(8.0))
        .border(px(1.0))
        .border_color(move || theme().input)
        .bg(move || theme().card)
        .flex_grow(1.0)
        .basis(px(220.0))
        .min_w(px(0.0))
        .child(
            image(asset!("icons/search.svg"))
                .size(px(16.0))
                .tint_color(move || theme().muted_foreground),
        )
        .child(
            input()
                .value(value)
                .on_change(move |next: &str| value.set(next.to_string()))
                .placeholder(placeholder)
                .auto_fill(AutoFill::Off)
                .flex_grow(1.0)
                .min_w(px(0.0))
                .h(px(38.0))
                .px(px(0.0))
                .border(px(0.0))
                .bg(rgba(0x00000000))
                .text_size(px(14.0))
                .text_color(move || theme().foreground),
        )
        .child(
            when(
                move || !value.get().is_empty(),
                move || {
                    button()
                        .cursor_pointer()
                        .items_center()
                        .justify_center()
                        .h(px(24.0))
                        .w(px(24.0))
                        .rounded(px(5.0))
                        .on_press(move || value.set(String::new()))
                        .child(
                            image(asset!("icons/x.svg"))
                                .size(px(13.0))
                                .tint_color(move || theme().muted_foreground),
                        )
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
}
