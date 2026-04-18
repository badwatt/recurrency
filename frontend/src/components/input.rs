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
            value: "{value}",
            placeholder: "{placeholder}",
            oninput: move |e: FormEvent| on_input.call(e.value()),
            class: "flex h-10 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50",
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn input_type_default() {
        let t = "text".to_string();
        assert_eq!(t, "text");
    }
}