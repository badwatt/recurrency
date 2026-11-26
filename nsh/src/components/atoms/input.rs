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
        .h(px(40.0))
        .flex_shrink(1.0)
        .w_full()
        .min_w(px(0.0))
        .px(px(12.0))
        .rounded(px(7.0))
        .border(px(1.0))
        .border_color(move || theme().input)
        .bg(move || theme().card)
        .text_size(px(14.0))
        .text_color(move || theme().foreground)
}

/// Keeps only ASCII digits, at most one decimal separator, and turns a comma
/// into a dot. A leading separator becomes `0.`.
pub fn sanitize_decimal(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut separator = false;
    for character in text.chars() {
        if character.is_ascii_digit() {
            out.push(character);
        } else if matches!(character, '.' | ',') && !separator {
            if out.is_empty() {
                out.push('0');
            }
            out.push('.');
            separator = true;
        }
    }
    out
}

/// Keeps only ASCII digits.
pub fn sanitize_integer(text: &str) -> String {
    text.chars().filter(char::is_ascii_digit).collect()
}

fn filtered_input(
    value: Signal<String>,
    placeholder: &'static str,
    filter: fn(&str) -> String,
) -> impl IntoElement {
    input()
        .value(value)
        .on_change(move |next: &str| {
            let clean = filter(next);
            if clean != next {
                value.set(next.to_string());
                let target = value;
                spawn(async move { target.set(clean) });
            } else {
                value.set(clean);
            }
        })
        .placeholder(placeholder)
        .auto_fill(AutoFill::Off)
        .h(px(40.0))
        .flex_shrink(1.0)
        .w_full()
        .min_w(px(0.0))
        .px(px(12.0))
        .rounded(px(7.0))
        .border(px(1.0))
        .border_color(move || theme().input)
        .bg(move || theme().card)
        .text_size(px(14.0))
        .text_color(move || theme().foreground)
}

/// Amount-like field: digits and one decimal separator.
pub fn numeric_input(value: Signal<String>, placeholder: &'static str) -> impl IntoElement {
    filtered_input(value, placeholder, sanitize_decimal)
}

/// Whole-number field: digits only.
pub fn integer_input(value: Signal<String>, placeholder: &'static str) -> impl IntoElement {
    filtered_input(value, placeholder, sanitize_integer)
}

fn filtered_input_value(
    value: String,
    placeholder: &'static str,
    filter: fn(&str) -> String,
    on_change: impl Fn(String) + 'static,
) -> impl IntoElement {
    let text = signal(value);
    input()
        .value(text)
        .on_change(move |next: &str| {
            let clean = filter(next);
            if clean != next {
                text.set(next.to_string());
                let target = text;
                let clean_for_reset = clean.clone();
                spawn(async move { target.set(clean_for_reset) });
            } else {
                text.set(clean.clone());
            }
            on_change(clean);
        })
        .placeholder(placeholder)
        .auto_fill(AutoFill::Off)
        .h(px(40.0))
        .flex_shrink(1.0)
        .w_full()
        .min_w(px(0.0))
        .px(px(12.0))
        .rounded(px(7.0))
        .border(px(1.0))
        .border_color(move || theme().input)
        .bg(move || theme().card)
        .text_size(px(14.0))
        .text_color(move || theme().foreground)
}

/// Amount-like field for dynamic rows.
pub fn numeric_input_value(
    value: String,
    placeholder: &'static str,
    on_change: impl Fn(String) + 'static,
) -> impl IntoElement {
    filtered_input_value(value, placeholder, sanitize_decimal, on_change)
}

/// Whole-number field for dynamic rows.
pub fn integer_input_value(
    value: String,
    placeholder: &'static str,
    on_change: impl Fn(String) + 'static,
) -> impl IntoElement {
    filtered_input_value(value, placeholder, sanitize_integer, on_change)
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
        .h(px(40.0))
        .flex_shrink_0()
        .w_full()
        .min_w(px(0.0))
        .px(px(12.0))
        .rounded(px(7.0))
        .border(px(1.0))
        .border_color(move || theme().input)
        .bg(move || theme().card)
        .text_size(px(14.0))
        .text_color(move || theme().foreground)
}

#[cfg(test)]
mod tests {
    use super::{sanitize_decimal, sanitize_integer};

    #[test]
    fn decimals_keep_digits_and_one_separator() {
        assert_eq!(sanitize_decimal("12a.3,4"), "12.34");
        assert_eq!(sanitize_decimal("abc"), "");
        assert_eq!(sanitize_decimal(".5"), "0.5");
    }

    #[test]
    fn integers_keep_only_digits() {
        assert_eq!(sanitize_integer("2a5,9"), "259");
    }
}
