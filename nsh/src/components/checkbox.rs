//! Shared checkbox with native semantics and the app's focus treatment.

use nsh_base::{button, theme};
use nsh_core::prelude::*;
use std::rc::Rc;

pub fn checkbox(checked: Signal<bool>, label: String) -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .child(
            button()
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
                .accessibility_label(label.clone())
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
            text(label)
                .text_size(px(13.0))
                .text_color(move || theme().foreground),
        )
}

/// Checkbox without an inline label, for cards and rows whose title already
/// explains the target.
pub fn checkbox_mark(
    checked: impl Fn() -> bool + 'static,
    label: String,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    let checked = Rc::new(checked);
    let checked_border = checked.clone();
    let checked_fill = checked.clone();
    let checked_icon = checked.clone();
    button()
        .h(px(16.0))
        .w(px(16.0))
        .flex_shrink(0.0)
        .items_center()
        .justify_center()
        .rounded(px(4.0))
        .border(px(1.0))
        .border_color(move || {
            if checked_border() {
                theme().primary
            } else {
                theme().border
            }
        })
        .bg(move || {
            if checked_fill() {
                theme().primary
            } else {
                theme().card
            }
        })
        .accessibility_label(label)
        .on_press(move || on_press())
        .child(
            when(
                move || checked_icon(),
                move || {
                    image(asset!("icons/check.svg"))
                        .size(px(11.0))
                        .tint_color(move || theme().primary_foreground)
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
}
