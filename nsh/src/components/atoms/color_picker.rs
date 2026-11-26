//! Shared color field for categories and accounts. Native Shell has no
//! platform color dialog, so the field offers a curated palette plus the
//! current value as a readable hex label.

use crate::components::input::text_input;
use nsh_base::{button, theme};
use nsh_core::prelude::*;

pub const DEFAULT_COLOR: &str = "#8b5cf6";
pub const COLORS: &[&str] = &[
    "#8b5cf6", "#6366f1", "#3b82f6", "#0ea5e9", "#14b8a6", "#22c55e", "#84cc16", "#eab308",
    "#f59e0b", "#f97316", "#ef4444", "#ec4899",
];

pub fn color_picker(value: Signal<String>) -> impl IntoElement {
    view()
        .w_full()
        .flex_col()
        .gap(px(8.0))
        .child(
            view()
                .flex_row()
                .flex_wrap()
                .gap(px(6.0))
                .children(COLORS.iter().map(|color| {
                    let color = *color;
                    let active = value.get().eq_ignore_ascii_case(color);
                    button()
                        .cursor_pointer()
                        .h(px(24.0))
                        .w(px(24.0))
                        .rounded(px(999.0))
                        .border(px(2.0))
                        .border_color(move || {
                            if active {
                                theme().foreground
                            } else {
                                theme().border
                            }
                        })
                        .bg(rgba(color_value(color)))
                        .accessibility_label(color)
                        .on_press(move || value.set(color.to_string()))
                })),
        )
        .child(text_input(value, "#8b5cf6"))
}

pub fn color_value(hex: &str) -> u32 {
    let value = hex.trim_start_matches('#');
    if value.len() != 6 {
        return 0x8b5cf6ff;
    }
    let parsed = u32::from_str_radix(value, 16).unwrap_or(0x8b5cf6);
    (parsed << 8) | 0xff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_palette_color_is_a_six_digit_hex() {
        for color in COLORS {
            assert_eq!(color.len(), 7);
            assert!(color.starts_with('#'));
            assert!(u32::from_str_radix(&color[1..], 16).is_ok());
        }
    }

    #[test]
    fn colors_convert_to_opaque_rgba() {
        assert_eq!(color_value("#000000"), 0x000000ff);
        assert_eq!(color_value("bad"), 0x8b5cf6ff);
    }
}
