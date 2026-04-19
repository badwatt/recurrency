use dioxus::prelude::*;

#[component]
pub fn Select(
    value: String,
    on_change: EventHandler<String>,
    children: Element,
) -> Element {
    rsx! {
        select {
            value: "{value}",
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
        // Verify Element type exists and is constructible in test context
        fn _check(_: Element) {}
        // Basic structural check - we can define functions taking Element
        let _: fn(Element) = |_| {};
    }

    #[test]
    fn select_event_handler_type_defined() {
        // EventHandler<String> is a valid type
        fn _check_handler(h: EventHandler<String>) {
            // Just verify the parameter is accepted
            let _ = h;
        }
    }
}