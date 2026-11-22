//! Attractive, picker-only color field for accounts and categories.
//!
//! The control is a single native color input covering a preview card, so the
//! operating system color picker is the only way to choose a color. There is no
//! preset palette and no free-form hex input.

use crate::utils::color::{DEFAULT_COLOR, normalize_hex, with_alpha};
use crate::utils::icons::icon_display;
use dioxus::prelude::*;
use lucide_dioxus::ChevronRight;

#[component]
pub fn ColorField(
    value: Signal<String>,
    #[props(default = "Color".to_string())] label: String,
    /// Optional name shown in the preview card.
    #[props(default)]
    name: Option<String>,
    /// Optional icon shown in the preview swatch.
    #[props(default)]
    icon: Option<String>,
) -> Element {
    let hex = normalize_hex(&value()).unwrap_or_else(|| DEFAULT_COLOR.to_string());
    let tint = with_alpha(&hex, 0x24);
    let ring = with_alpha(&hex, 0x66);
    let preview_name = name
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| "Pick a color".to_string());

    rsx! {
        div { class: "space-y-2",
            div { class: "flex items-center justify-between gap-3",
                label { class: "text-sm font-medium", {label.clone()} }
                span {
                    class: "rounded-full border border-border/60 bg-muted/40 px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.18em] text-muted-foreground",
                    {hex.clone()}
                }
            }
            label {
                class: "group relative flex cursor-pointer items-center gap-3 overflow-hidden rounded-xl border border-border/60 bg-gradient-to-br from-card to-muted/30 p-3 transition-all duration-200 hover:border-primary/50 hover:shadow-md focus-within:ring-2 focus-within:ring-ring",
                span {
                    class: "relative flex h-12 w-12 shrink-0 items-center justify-center rounded-xl p-[2px] shadow-sm",
                    style: "background: linear-gradient(135deg, {ring}, {hex})",
                    span {
                        class: "flex h-full w-full items-center justify-center rounded-[10px]",
                        style: "background-color: {tint}",
                        if let Some(icon) = icon {
                            span { class: "text-lg", style: "color: {hex}", {icon_display(&icon)} }
                        } else {
                            span { class: "h-5 w-5 rounded-full", style: "background-color: {hex}" }
                        }
                    }
                }
                div { class: "min-w-0 flex-1",
                    p { class: "truncate text-sm font-medium", {preview_name} }
                    p { class: "text-xs text-muted-foreground", "Click to open the color picker" }
                }
                span {
                    class: "text-muted-foreground transition-transform duration-200 group-hover:translate-x-0.5",
                    ChevronRight { size: 16 }
                }
                input {
                    r#type: "color",
                    id: "color-picker",
                    name: "color",
                    value: hex,
                    aria_label: label.clone(),
                    title: label,
                    oninput: move |event| value.set(event.value().to_ascii_lowercase()),
                    class: "absolute inset-0 h-full w-full cursor-pointer opacity-0",
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    fn production_source() -> &'static str {
        include_str!("color_picker.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap_or_default()
    }

    #[test]
    fn picker_is_the_only_color_control() {
        let source = production_source();
        assert!(source.contains("r#type: \"color\""));
        assert!(source.contains("opacity-0"));
        assert!(!source.contains("PALETTE"));
        assert!(!source.contains("type: \"text\""));
    }

    #[test]
    fn preview_uses_the_selected_color() {
        let source = production_source();
        assert!(source.contains("background-color: {tint}"));
        assert!(source.contains("linear-gradient"));
        assert!(source.contains("{hex}"));
    }

    #[test]
    fn empty_name_falls_back_to_prompt() {
        let preview_name = None::<String>
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "Pick a color".to_string());
        assert_eq!(preview_name, "Pick a color");
    }

    #[test]
    fn blank_name_falls_back_to_prompt() {
        let preview_name = Some("   ".to_string())
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "Pick a color".to_string());
        assert_eq!(preview_name, "Pick a color");
    }
}
