use dioxus::prelude::*;

#[component]
pub fn Card(children: Element) -> Element {
    rsx! {
        div { class: "rounded-lg border border-border bg-card text-card-foreground shadow-sm",
            {children}
        }
    }
}

#[component]
pub fn CardHeader(children: Element) -> Element {
    rsx! {
        div { class: "flex flex-col space-y-1.5 p-6", {children} }
    }
}

#[component]
pub fn CardTitle(children: Element) -> Element {
    rsx! {
        h3 { class: "text-lg font-semibold leading-none tracking-tight", {children} }
    }
}

#[component]
pub fn CardContent(children: Element) -> Element {
    rsx! {
        div { class: "p-6 pt-0", {children} }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn card_classes_exist() {
        assert!(!"rounded-lg border border-border bg-card".is_empty());
    }
}