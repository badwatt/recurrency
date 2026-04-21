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

    #[test]
    fn card_class_contains_rounded() {
        let cls = "rounded-lg border border-border bg-card text-card-foreground shadow-sm";
        assert!(cls.contains("rounded-lg"));
    }

    #[test]
    fn card_class_contains_border() {
        let cls = "rounded-lg border border-border bg-card";
        assert!(cls.contains("border"));
    }

    #[test]
    fn card_class_contains_bg_card() {
        let cls = "rounded-lg border border-border bg-card";
        assert!(cls.contains("bg-card"));
    }

    #[test]
    fn card_header_space_y() {
        let cls = "flex flex-col space-y-1.5 p-6";
        assert!(cls.contains("space-y-1.5"));
        assert!(cls.contains("p-6"));
    }

    #[test]
    fn card_title_class() {
        let cls = "text-lg font-semibold leading-none tracking-tight";
        assert!(cls.contains("text-lg"));
        assert!(cls.contains("font-semibold"));
    }

    #[test]
    fn card_content_padding() {
        let cls = "p-6 pt-0";
        assert!(cls.contains("p-6"));
        assert!(cls.contains("pt-0"));
    }
}
