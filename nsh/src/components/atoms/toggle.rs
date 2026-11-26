//! Shared switch for notification and preference rows.

use nsh_base::{button, theme};
use nsh_core::prelude::*;
use std::rc::Rc;

pub fn toggle(
    checked: bool,
    label: String,
    on_change: impl Fn(bool) + 'static,
) -> impl IntoElement {
    let handler = Rc::new(on_change);
    let press = handler.clone();
    view()
        .h(px(22.0))
        .w(px(38.0))
        .rounded(px(999.0))
        .p(px(3.0))
        .bg(move || {
            if checked {
                theme().primary
            } else {
                theme().muted
            }
        })
        .child(
            button()
                .cursor_pointer()
                .cursor_pointer()
                .h(px(16.0))
                .w(px(16.0))
                .rounded(px(999.0))
                .bg(move || theme().card)
                .accessibility_label(label)
                .on_press(move || press(!checked)),
        )
}

/// Reactive switch bound to a signal; the knob and track update on press.
pub fn toggle_signal(
    checked: Signal<bool>,
    label: String,
    on_change: impl Fn(bool) + 'static,
) -> impl IntoElement {
    let handler = Rc::new(on_change);
    let press = handler.clone();
    view()
        .h(px(22.0))
        .w(px(38.0))
        .rounded(px(999.0))
        .p(px(3.0))
        .bg(move || {
            if checked.get() {
                theme().primary
            } else {
                theme().muted
            }
        })
        .child(view().flex_row().items_center().h_full().w_full())
        .child(
            button()
                .cursor_pointer()
                .h_full()
                .w_full()
                .flex_row()
                .items_center()
                .absolute()
                .inset_0()
                .p(px(3.0))
                .on_press(move || {
                    let next = !checked.get();
                    checked.set(next);
                    press(next);
                })
                .child(view().w(move || if checked.get() { px(16.0) } else { px(0.0) }))
                .child(
                    view()
                        .h(px(16.0))
                        .w(px(16.0))
                        .flex_shrink_0()
                        .rounded(px(999.0))
                        .bg(move || theme().card),
                )
                .accessibility_label(label),
        )
}
