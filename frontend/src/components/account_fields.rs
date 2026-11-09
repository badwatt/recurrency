use crate::utils::icons::{ICON_NAMES, icon_display};
use dioxus::prelude::*;

/// Shared account form fields: name, icon picker, and color palette.
/// Used by the Accounts page and by the inline create flow in account selects.
pub const COLOR_PALETTE: &[&str] = &[
    "#ef4444", "#f97316", "#f59e0b", "#eab308", "#84cc16", "#22c55e", "#10b981", "#14b8a6",
    "#06b6d4", "#0ea5e9", "#3b82f6", "#6366f1", "#8b5cf6", "#a855f7", "#d946ef", "#ec4899",
    "#f43f5e", "#6b7280", "#78716c", "#a3a3a3",
];

#[component]
pub fn AccountFields(
    name: Signal<String>,
    icon: Signal<String>,
    color: Signal<String>,
    name_error: Signal<String>,
) -> Element {
    let mut show_icon_picker = use_signal(|| false);

    rsx! {
        div { class: "space-y-4",
            // Name
            div { class: "space-y-2",
                label { class: "text-sm font-medium", "Name" }
                input {
                    r#type: "text",
                    value: name(),
                    oninput: move |e| {
                        name.set(e.value());
                        name_error.set(String::new());
                    },
                    class: if name_error().is_empty() { "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm" } else { "flex h-10 w-full rounded-md border border-red-500 bg-background px-3 py-2 text-sm" },
                }
                if !name_error().is_empty() {
                    p { class: "text-xs text-red-500", "{name_error()}" }
                }
            }

            // Icon
            div { class: "space-y-2",
                div { class: "flex items-center justify-between",
                    label { class: "text-sm font-medium", "Icon" }
                    button {
                        r#type: "button",
                        onclick: move |_| show_icon_picker.toggle(),
                        class: "text-sm text-primary hover:underline",
                        if show_icon_picker() { "Hide icons" } else { "Choose icon" }
                    }
                }
                div {
                    class: "flex h-10 w-10 items-center justify-center rounded-full",
                    style: "background-color: {color()}20",
                    span { style: "color: {color()}", {icon_display(&icon())} }
                }
                if show_icon_picker() {
                    div { class: "grid grid-cols-8 gap-1 p-2 border border-border rounded-md max-h-48 overflow-y-auto",
                        for icon_name in ICON_NAMES {
                            {let is_selected = *icon_name == icon();
                             rsx! {
                                button {
                                    r#type: "button",
                                    key: "{icon_name}",
                                    class: if is_selected { "h-8 w-8 flex items-center justify-center rounded border-2 border-primary text-sm" } else { "h-8 w-8 flex items-center justify-center rounded border border-border hover:bg-accent text-sm" },
                                    onclick: move |_| icon.set(icon_name.to_string()),
                                    {icon_display(icon_name)}
                                }
                            }}
                        }
                    }
                }
            }

            // Color
            div { class: "space-y-2",
                label { class: "text-sm font-medium", "Color" }
                div { class: "grid grid-cols-10 gap-1",
                    for palette_color in COLOR_PALETTE {
                        {let is_selected = *palette_color == color();
                         rsx! {
                            button {
                                r#type: "button",
                                key: "{palette_color}",
                                class: if is_selected { "h-8 w-8 rounded-full border-2 border-foreground ring-2 ring-ring" } else { "h-8 w-8 rounded-full border border-border hover:ring-1 hover:ring-ring" },
                                style: "background-color: {palette_color}",
                                onclick: move |_| color.set(palette_color.to_string()),
                            }
                        }}
                    }
                }
            }
        }
    }
}
