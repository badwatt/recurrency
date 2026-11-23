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
