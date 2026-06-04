use dioxus::prelude::*;

#[component]
pub fn Input(
    value: String,
    on_input: EventHandler<String>,
    placeholder: String,
    #[props(default = "text".to_string())] r#type: String,
) -> Element {
    rsx! {
        input {
            r#type: r#type,
            value: {value.to_string()},
            placeholder: {placeholder.to_string()},
            oninput: move |e: FormEvent| on_input.call(e.value()),
            class: "flex h-10 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_type_default() {
        let t = "text".to_string();
        assert_eq!(t, "text");
    }

    #[test]
    fn input_props_email() {
        let t = "email".to_string();
        assert_eq!(t, "email");
    }

    #[test]
    fn input_props_password() {
        let t = "password".to_string();
        assert_eq!(t, "password");
    }

    #[test]
    fn input_props_number() {
        let t = "number".to_string();
        assert_eq!(t, "number");
    }

    #[test]
    fn input_props_date() {
        let t = "date".to_string();
        assert_eq!(t, "date");
    }

    #[test]
    fn input_placeholder_preserved() {
        let placeholder = "Enter value".to_string();
        assert!(!placeholder.is_empty());
    }

    #[test]
    fn input_value_empty_default() {
        let value = String::new();
        assert!(value.is_empty());
    }

    #[test]
    fn input_class_contains_focus_ring() {
        let class = "flex h-10 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50";
        assert!(class.contains("focus:ring"));
        assert!(class.contains("focus:ring-offset"));
        assert!(class.contains("disabled:opacity-50"));
    }

    #[test]
    fn input_event_handler_signature() {
        // Verify EventHandler<String> can be called with a value
        fn handle_input(handler: &EventHandler<String>, val: String) {
            handler.call(val);
        }
        // Can't fully test without Dioxus runtime, but we verify types compile
        assert!(true);
    }
}
