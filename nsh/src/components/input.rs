//! Text input surface shared by forms and search fields.

use nsh_base::{input, theme};
use nsh_core::element::AutoFill;
use nsh_core::prelude::*;

pub fn text_input(value: Signal<String>, placeholder: &'static str) -> impl IntoElement {
    input()
        .value(value)
        .on_change(move |next: &str| value.set(next.to_string()))
        .placeholder(placeholder)
        .auto_fill(AutoFill::Off)
        .h(px(42.0))
        .flex_grow(1.0)
        .basis(px(0.0))
        .min_w(px(0.0))
        .px(px(12.0))
        .rounded(px(7.0))
        .border(px(1.0))
        .border_color(move || theme().input)
        .bg(move || theme().card)
        .text_size(px(14.0))
        .text_color(move || theme().foreground)
}

/// Input whose current value is passed by copy, for dynamic rows that cannot
/// allocate one `Signal` per field (provider configs, key/value editors).
pub fn text_input_value(
    value: String,
    placeholder: &'static str,
    on_change: impl Fn(String) + 'static,
) -> impl IntoElement {
    input()
        .value(value)
        .on_change(move |next: &str| on_change(next.to_string()))
        .placeholder(placeholder)
        .auto_fill(AutoFill::Off)
        .h(px(42.0))
        .flex_grow(1.0)
        .basis(px(0.0))
        .min_w(px(0.0))
        .px(px(12.0))
        .rounded(px(7.0))
        .border(px(1.0))
        .border_color(move || theme().input)
        .bg(move || theme().card)
        .text_size(px(14.0))
        .text_color(move || theme().foreground)
}
