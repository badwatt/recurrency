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
    let generated_id = use_hook(|| format!("checkbox-{}", uuid::Uuid::new_v4()));
    let field_id = if id.is_empty() { generated_id } else { id };
    let field_name = field_id.clone();
    let input = rsx! {
        input {
            r#type: "checkbox",
            id: field_id.clone(),
            name: field_name,
            checked,
            disabled,
            class: "h-4 w-4 rounded border-border accent-primary {class}",
            onchange: move |event| on_change.call(event.checked()),
        }
    };
    if label.is_empty() {
        input
    } else {
        rsx! {
            label { r#for: field_id, class: "inline-flex cursor-pointer items-center gap-2 text-sm font-medium",
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
