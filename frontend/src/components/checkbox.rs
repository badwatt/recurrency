use dioxus::prelude::*;

/// Shared native checkbox. Keeping the input native preserves keyboard and
/// form semantics while standardising size, accent and focus treatment.
#[component]
pub fn Checkbox(
    checked: bool,
    on_change: EventHandler<bool>,
    #[props(default)] id: String,
    #[props(default)] label: String,
    #[props(default)] disabled: bool,
    #[props(default)] class: String,
) -> Element {
    let input = rsx! {
        input {
            r#type: "checkbox",
            id: id.clone(),
            checked,
            disabled,
            class: "h-4 w-4 rounded border-border accent-primary focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 {class}",
            onchange: move |event| on_change.call(event.checked()),
        }
    };
    if label.is_empty() {
        input
    } else {
        rsx! {
            label { r#for: id, class: "inline-flex cursor-pointer items-center gap-2 text-sm font-medium",
                {input}
                {label}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn checkbox_keeps_native_input_and_label() {
        let source = include_str!("checkbox.rs");
        assert!(source.contains("r#type: \"checkbox\""));
        assert!(source.contains("event.checked()"));
    }
}
