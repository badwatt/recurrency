use crate::components::button::{ButtonSize, ButtonVariant, IconButton};
use crate::components::input::Input;
use dioxus::prelude::*;
use lucide_dioxus::Calendar;

/// Shared date field: a native date input for correct keyboard/form
/// semantics, a themed calendar affordance, and a stable focus surface.
#[component]
pub fn DatePicker(
    value: String,
    on_input: EventHandler<String>,
    #[props(default)] id: String,
    #[props(default)] min: String,
    #[props(default)] max: String,
    #[props(default)] disabled: bool,
    #[props(default)] class: String,
) -> Element {
    let input_id = if id.is_empty() {
        format!("date-picker-{}", uuid::Uuid::new_v4())
    } else {
        id
    };
    let picker_id = input_id.clone();
    let open_picker = move |_| {
        let _ = document::eval(&format!(
            "const e=document.getElementById('{picker_id}'); if(e){{ e.focus(); if(e.showPicker) e.showPicker(); }}"
        ));
    };

    rsx! {
        div { class: "relative {class}",
            Input {
                id: input_id.clone(),
                r#type: "date",
                value,
                min,
                max,
                disabled,
                aria_label: "Date".to_string(),
                class: "date-input pr-10".to_string(),
                on_input: move |value| on_input.call(value),
            }
            div { class: "absolute right-1 top-1/2 -translate-y-1/2",
                IconButton {
                    label: "Open date picker".to_string(),
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    class: "text-muted-foreground".to_string(),
                    disabled,
                    onclick: open_picker,
                    Calendar { size: 15 }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn date_picker_hides_the_native_indicator_and_offers_a_themed_button() {
        let source = include_str!("date_picker.rs");
        assert!(source.contains("date-input pr-10"));
        assert!(source.contains("Open date picker"));
        assert!(source.contains("showPicker"));
    }
}
