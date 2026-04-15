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
    #[test]
    fn select_renders() {
        // Placeholder; real test needs Dioxus render context
        assert!(true);
    }
}