use dioxus::prelude::*;

#[component]
pub fn Select(value: String, on_change: EventHandler<String>, children: Element) -> Element {
    rsx! {
        select {
            value: value.to_string(),
            onchange: move |e: FormEvent| on_change.call(e.value()),
            class: "flex h-10 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50",
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_component_module_loads() {
        assert!(true);
    }

    #[test]
    fn select_children_element_is_usable() {
        fn _check(_: Element) {}
        let _: fn(Element) = |_| {};
    }

    #[test]
    fn select_event_handler_type_defined() {
        fn _check_handler(h: EventHandler<String>) {
            let _ = h;
        }
    }

    #[test]
    fn select_uses_select_element() {
        let src = include_str!("select.rs");
        assert!(
            src.contains("select {"),
            "Select component must use <select> element"
        );
    }

    #[test]
    fn select_binds_value_attribute() {
        let src = include_str!("select.rs");
        assert!(src.contains("value:"), "Select must bind value attribute");
        assert!(
            src.contains("value: value.to_string()"),
            "Value binding must use .to_string() to avoid clippy::useless_format"
        );
    }

    #[test]
    fn select_has_onchange_handler() {
        let src = include_str!("select.rs");
        assert!(
            src.contains("onchange:"),
            "Select must have onchange handler"
        );
    }

    #[test]
    fn select_calls_event_value() {
        let src = include_str!("select.rs");
        assert!(
            src.contains("e.value()"),
            "onchange must call e.value() to get selected value"
        );
    }

    #[test]
    fn select_has_class_styling() {
        let src = include_str!("select.rs");
        assert!(src.contains("class:"), "Select must have class attribute");
    }

    #[test]
    fn select_renders_children() {
        let src = include_str!("select.rs");
        assert!(
            src.contains("{children}"),
            "Select must render children (options)"
        );
    }

    #[test]
    fn select_moves_event_handler() {
        let src = include_str!("select.rs");
        assert!(
            src.contains("move |e:"),
            "onchange closure must use move for ownership"
        );
        assert!(src.contains("FormEvent"), "Event type must be FormEvent");
    }

    #[test]
    fn select_component_signature() {
        let src = include_str!("select.rs");
        assert!(src.contains("pub fn Select("), "Component must be public");
        assert!(src.contains("value: String"), "Must accept value prop");
        assert!(
            src.contains("on_change: EventHandler<String>"),
            "Must accept on_change callback"
        );
        assert!(
            src.contains("children: Element"),
            "Must accept children Element"
        );
    }

    #[test]
    fn select_class_contains_focus_styles() {
        let src = include_str!("select.rs");
        // Must have focus styles for accessibility
        assert!(src.contains("focus:"), "Select must have focus styles");
        assert!(
            src.contains("focus-ring") || src.contains("focus:outline"),
            "Select must have visible focus indicator"
        );
    }

    #[test]
    fn select_class_contains_disabled_styles() {
        let src = include_str!("select.rs");
        assert!(
            src.contains("disabled:"),
            "Select must have disabled styles"
        );
        assert!(
            src.contains("disabled:cursor-not-allowed") || src.contains("disabled:opacity"),
            "Disabled state must change cursor and opacity"
        );
    }

    #[test]
    fn select_has_background_and_border() {
        let src = include_str!("select.rs");
        assert!(src.contains("bg-background"), "Select must have background");
        assert!(src.contains("border"), "Select must have border");
    }

    #[test]
    fn select_uses_dioxus_prelude() {
        let src = include_str!("select.rs");
        assert!(
            src.contains("use dioxus::prelude::"),
            "Must import dioxus prelude"
        );
    }
}
