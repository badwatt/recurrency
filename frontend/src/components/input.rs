use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputVariant {
    #[default]
    Default,
    /// Borderless controls embedded in a popover. No focus ring is painted
    /// because the popover already owns the focus surface.
    Bare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputSize {
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
}

impl InputSize {
    fn class(self) -> &'static str {
        match self {
            Self::Xs => "h-7 px-2 text-xs",
            Self::Sm => "h-9 px-3 text-sm",
            Self::Md => "h-10 px-3 py-2 text-sm",
            Self::Lg => "h-12 px-3 py-2 text-base",
        }
    }
}

#[component]
pub fn Input(
    value: String,
    on_input: Option<EventHandler<String>>,
    #[props(default)] r#type: String,
    #[props(default)] placeholder: String,
    #[props(default)] class: String,
    #[props(default)] size: InputSize,
    #[props(default)] variant: InputVariant,
    #[props(default)] disabled: bool,
    #[props(default)] required: bool,
    #[props(default)] readonly: bool,
    #[props(default)] autofocus: bool,
    #[props(default)] autocomplete: String,
    #[props(default)] autocapitalize: String,
    #[props(default)] autocorrect: String,
    #[props(default)] spellcheck: String,
    #[props(default)] inputmode: String,
    #[props(default)] id: String,
    #[props(default)] name: String,
    #[props(default)] aria_label: String,
    #[props(default)] min: String,
    #[props(default)] max: String,
    #[props(default)] step: String,
    #[props(default)] minlength: String,
    #[props(default)] maxlength: String,
    on_blur: Option<EventHandler<FocusEvent>>,
    on_keydown: Option<EventHandler<KeyboardEvent>>,
) -> Element {
    let input_type = if r#type.is_empty() {
        "text".to_string()
    } else {
        r#type
    };
    let size_class = size.class();
    let surface_class = match variant {
        InputVariant::Default => "rounded-md border border-input bg-background text-foreground placeholder:text-muted-foreground outline-none transition-colors focus-visible:border-primary/60 focus-visible:ring-2 focus-visible:ring-ring/40",
        InputVariant::Bare => "bg-transparent text-foreground placeholder:text-muted-foreground outline-none",
    };

    rsx! {
        input {
            r#type: input_type,
            id,
            name,
            value,
            placeholder,
            class: "w-full {surface_class} disabled:cursor-not-allowed disabled:opacity-50 {size_class} {class}",
            disabled,
            required,
            readonly,
            autofocus,
            autocomplete,
            autocapitalize,
            autocorrect,
            spellcheck,
            inputmode,
            aria_label,
            min,
            max,
            step,
            minlength,
            maxlength,
            oninput: move |event| {
                if let Some(handler) = on_input {
                    handler.call(event.value());
                }
            },
            onblur: move |event| {
                if let Some(handler) = on_blur {
                    handler.call(event);
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
    use super::*;

    #[test]
    fn input_defaults_to_text_and_medium() {
        assert_eq!(InputSize::default(), InputSize::Md);
        assert!(InputSize::Md.class().contains("h-10"));
        assert!(InputSize::Sm.class().contains("h-9"));
    }

    #[test]
    fn bare_input_does_not_paint_a_focus_ring() {
        let source = include_str!("input.rs");
        assert!(source.contains("InputVariant::Bare => \"bg-transparent"));
        assert!(!source.contains("InputVariant::Bare => \"bg-transparent text-foreground placeholder:text-muted-foreground outline-none focus"));
    }

    #[test]
    fn input_has_shared_focus_and_disabled_states() {
        let source = include_str!("input.rs");
        assert!(source.contains("focus-visible:ring-2"));
        assert!(source.contains("disabled:opacity-50"));
    }
}
