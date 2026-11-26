//! Full-spectrum colour picker: hue × saturation surface, lightness strip,
//! live preview and exact hex value.

use crate::components::input::text_input;
use nsh_base::theme;
use nsh_core::prelude::*;

pub const DEFAULT_COLOR: &str = "#8b5cf6";

pub fn color_picker(value: Signal<String>) -> impl IntoElement {
    let current = value.get();
    let (hue, saturation, lightness) = hex_to_hsl(&current);
    let spectrum_value = value;
    let light_value = value;
    view()
        .w_full()
        .flex_col()
        .gap(px(8.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .child(
                    view()
                        .h(px(30.0))
                        .w(px(30.0))
                        .rounded(px(999.0))
                        .border(px(2.0))
                        .border_color(move || theme().border)
                        .bg(move || rgba(color_value(&value.get()))),
                )
                .child(
                    text(move || value.get().to_uppercase())
                        .text_size(px(12.0))
                        .font_family(crate::theme::MONO)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            view()
                .w_full()
                .h(px(88.0))
                .rounded(px(6.0))
                .overflow_hidden()
                .grid()
                .grid_template_columns(SharedString::from("repeat(18, minmax(0, 1fr))"))
                .children((0..8).flat_map(|row| {
                    (0..18).map(move |column| {
                        let h = column as f32 * 20.0;
                        let s = 100.0 - row as f32 * 13.0;
                        let color = hsl_to_hex(h, s, lightness);
                        let target = format!("#{color:06x}");
                        view()
                            .h_full()
                            .w_full()
                            .bg(rgb(color))
                            .cursor_pointer()
                            .on_press(move || spectrum_value.set(target.clone()))
                            .into_element()
                    })
                })),
        )
        .child(
            view()
                .w_full()
                .h(px(16.0))
                .rounded(px(999.0))
                .overflow_hidden()
                .flex_row()
                .children((0..24).map(move |step| {
                    let l = step as f32 * 100.0 / 23.0;
                    let color = hsl_to_hex(hue, saturation, l);
                    let target = format!("#{color:06x}");
                    view()
                        .flex_grow(1.0)
                        .h_full()
                        .bg(rgb(color))
                        .cursor_pointer()
                        .on_press(move || light_value.set(target.clone()))
                        .into_element()
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

fn hex_to_hsl(hex: &str) -> (f32, f32, f32) {
    let value = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0x8b5cf6);
    let r = ((value >> 16) & 0xff) as f32 / 255.0;
    let g = ((value >> 8) & 0xff) as f32 / 255.0;
    let b = (value & 0xff) as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let lightness = (max + min) / 2.0;
    if (max - min).abs() < f32::EPSILON {
        return (0.0, 0.0, lightness * 100.0);
    }
    let delta = max - min;
    let saturation = if lightness > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };
    let hue = if (max - r).abs() < f32::EPSILON {
        (g - b) / delta + if g < b { 6.0 } else { 0.0 }
    } else if (max - g).abs() < f32::EPSILON {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    (hue * 60.0, saturation * 100.0, lightness * 100.0)
}

fn hsl_to_hex(hue: f32, saturation: f32, lightness: f32) -> u32 {
    let h = hue.rem_euclid(360.0) / 360.0;
    let s = (saturation / 100.0).clamp(0.0, 1.0);
    let l = (lightness / 100.0).clamp(0.0, 1.0);
    if s == 0.0 {
        let value = (l * 255.0).round() as u32;
        return (value << 16) | (value << 8) | value;
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let channel = |t: f32| {
        let t = t.rem_euclid(1.0);
        let value = if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        };
        (value * 255.0).round() as u32
    };
    (channel(h + 1.0 / 3.0) << 16) | (channel(h) << 8) | channel(h - 1.0 / 3.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_convert_to_opaque_rgba() {
        assert_eq!(color_value("#000000"), 0x000000ff);
        assert_eq!(color_value("bad"), 0x8b5cf6ff);
    }

    #[test]
    fn hsl_round_trips_primary_colours() {
        assert_eq!(hsl_to_hex(0.0, 100.0, 50.0), 0xff0000);
        assert_eq!(hsl_to_hex(120.0, 100.0, 50.0), 0x00ff00);
        assert_eq!(hsl_to_hex(240.0, 100.0, 50.0), 0x0000ff);
    }
}
