//! Multiline text surface shared by notes and future long-form fields.

use nsh_base::{input, theme};
use nsh_core::element::AutoFill;
use nsh_core::prelude::*;

pub fn text_area(value: Signal<String>, placeholder: &'static str) -> impl IntoElement {
    text_area_with_height(value, placeholder, 60.0)
}

pub fn text_area_with_height(
    value: Signal<String>,
    placeholder: &'static str,
    height: f32,
) -> impl IntoElement {
    input()
        .value(value)
        .on_change(move |next: &str| value.set(next.to_string()))
        .placeholder(placeholder)
        .auto_fill(AutoFill::Off)
        .multiline(true)
        .h(px(height))
        .min_h(px(height))
        .flex_shrink_0()
        .mt(px(-13.0))
        .w_full()
        .p(px(10.0))
        .rounded(px(7.0))
        .border(px(1.0))
        .border_color(move || theme().input)
        .bg(move || theme().background)
        .text_size(px(13.0))
        .text_color(move || theme().foreground)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_shared_textarea_is_multiline() {
        let source = include_str!("textarea.rs");
        assert!(source.contains("multiline(true)"));
        assert!(source.contains("AutoFill::Off"));
    }
}
