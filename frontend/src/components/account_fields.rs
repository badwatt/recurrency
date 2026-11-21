use crate::components::button::{Button, ButtonAlign, ButtonRadius, ButtonSize, ButtonVariant};
use crate::components::color_picker::ColorField;
use crate::components::field::Field;
use crate::components::input::{Input, InputSize};
use crate::utils::icons::{ICON_NAMES, icon_display};
use dioxus::prelude::*;

/// Shared account form fields: name, icon picker, and color picker.
/// Used by the Accounts page and by the inline create flow in account selects.

#[component]
pub fn AccountFields(
    name: Signal<String>,
    icon: Signal<String>,
    color: Signal<String>,
    name_error: Signal<String>,
) -> Element {
    let mut show_icon_picker = use_signal(|| false);
    let mut icon_search = use_signal(String::new);

    rsx! {
        div { class: "space-y-4",
            // Name
            Field { label: "Name", error: name_error(),
                Input {
                    value: name(),
                    on_input: move |value| {
                        name.set(value);
                        name_error.set(String::new());
                    },
                    class: if name_error().is_empty() { "" } else { "border-red-500" },
                }
            }

            // Icon
            div { class: "space-y-2",
                div { class: "flex items-center justify-between",
                    label { class: "text-sm font-medium", "Icon" }
                    Button {
                        variant: ButtonVariant::Link,
                        size: ButtonSize::Xs,
                        onclick: move |_| show_icon_picker.toggle(),
                        if show_icon_picker() { "Hide icons" } else { "Choose icon" }
                    }
                }
                div {
                    class: "flex h-10 w-10 items-center justify-center rounded-full",
                    style: "background-color: {color()}20",
                    span { style: "color: {color()}", {icon_display(&icon())} }
                }
                if show_icon_picker() {
                    Input {
                        r#type: "search",
                        size: InputSize::Sm,
                        placeholder: "Search icons…",
                        value: icon_search(),
                        on_input: move |value| icon_search.set(value),
                        class: "mb-2",
                    }
                    div { class: "grid grid-cols-8 gap-1 p-2 border border-border rounded-md max-h-48 overflow-y-auto",
                        for icon_name in ICON_NAMES.iter().filter(|name| name.to_lowercase().contains(&icon_search().to_lowercase())) {
                            {let is_selected = *icon_name == icon();
                             rsx! {
                                Button {
                                    key: "{icon_name}",
                                    variant: ButtonVariant::Unstyled,
                                    size: ButtonSize::Unstyled,
                                    radius: ButtonRadius::None,
                                    align: ButtonAlign::Center,
                                    class: if is_selected { "inline-flex h-8 w-8 items-center justify-center rounded border-2 border-primary text-sm" } else { "inline-flex h-8 w-8 items-center justify-center rounded border border-border text-sm hover:bg-accent" },
                                    onclick: move |_| icon.set(icon_name.to_string()),
                                    {icon_display(icon_name)}
                                }
                            }}
                        }
                    }
                }
            }

            // Color
            ColorField {
                label: "Color",
                value: color,
                name: Some(name()),
                icon: Some(icon()),
            }
        }
    }
}
