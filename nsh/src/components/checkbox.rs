//! Shared checkbox with native semantics and the app's focus treatment.

use nsh_base::{button, theme};
use nsh_core::prelude::*;

pub fn checkbox(checked: Signal<bool>, label: String) -> impl IntoElement {
    let has_label = !label.is_empty();
    let label_for_aria = label.clone();
    let label_text = label.clone();
    view()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .child(
            button()
                .cursor_pointer()
                .cursor_pointer()
                .h(px(16.0))
                .w(px(16.0))
                .items_center()
                .justify_center()
                .rounded(px(4.0))
                .border(px(1.0))
                .border_color(move || {
                    if checked.get() {
                        theme().primary
                    } else {
                        theme().border
                    }
                })
                .bg(move || {
                    if checked.get() {
                        theme().primary
                    } else {
                        theme().card
                    }
                })
                .accessibility_label(label_for_aria)
                .on_press(move || checked.set(!checked.get()))
                .child(
                    when(
                        move || checked.get(),
                        move || {
                            image(asset!("icons/check.svg"))
                                .size(px(11.0))
                                .tint_color(move || theme().primary_foreground)
                                .into_element()
                        },
                    )
                    .otherwise(|| view().h(px(0.0)).into_element()),
                ),
        )
        .child(
            when(
                move || has_label,
                move || {
                    text(label_text.clone())
                        .text_size(px(13.0))
                        .text_color(move || theme().foreground)
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
}
