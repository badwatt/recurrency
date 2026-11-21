use dioxus::prelude::*;

#[component]
pub fn Textarea(
    value: String,
    on_input: Option<EventHandler<String>>,
    #[props(default)] placeholder: String,
    #[props(default)] class: String,
    #[props(default)] rows: Option<u32>,
    #[props(default)] disabled: bool,
    #[props(default)] required: bool,
    on_keydown: Option<EventHandler<KeyboardEvent>>,
) -> Element {
    rsx! {
        textarea {
            value,
            placeholder,
            rows: rows.unwrap_or(3),
            class: "w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground outline-none transition-colors focus-visible:border-primary/60 focus-visible:ring-2 focus-visible:ring-ring/40 disabled:cursor-not-allowed disabled:opacity-50 {class}",
            disabled,
            required,
            oninput: move |event| {
                if let Some(handler) = on_input {
                    handler.call(event.value());
                }
            },
            onkeydown: move |event| {
                if let Some(handler) = on_keydown {
                    handler.call(event);
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn textarea_source_uses_the_shared_field_surface() {
        let source = include_str!("textarea.rs");
        assert!(source.contains("border-input"));
        assert!(source.contains("focus-visible:ring-2"));
    }
}
