use dioxus::prelude::*;

/// Shared page header. Title + subtitle, matches Reports style.
/// Animation belongs on the wrapper div, not here.
#[component]
pub fn PageHeader(title: String, subtitle: String) -> Element {
    rsx! {
        div {
            h1 { class: "text-4xl md:text-5xl font-bold tracking-tighter", "{title}" }
            p { class: "text-muted-foreground", "{subtitle}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use dioxus::prelude::*;

    #[test]
    fn page_header_renders_title() {
        let el: Element = rsx! {
            div {
                h1 { class: "text-4xl md:text-5xl font-bold tracking-tighter", "Subscriptions" }
                p { class: "text-muted-foreground", "Track recurring payments" }
            }
        };
        let s = format!("{el:?}");
        assert!(s.contains("Subscriptions"));
        assert!(s.contains("Track recurring payments"));
        assert!(s.contains("text-4xl"));
        assert!(s.contains("tracking-tighter"));
    }

    #[test]
    fn page_header_renders_subtitle() {
        let el: Element = rsx! {
            div {
                h1 { class: "text-4xl md:text-5xl font-bold tracking-tighter", "Reports" }
                p { class: "text-muted-foreground", "Analyze spending" }
            }
        };
        let s = format!("{el:?}");
        assert!(s.contains("Reports"));
        assert!(s.contains("Analyze spending"));
    }

    #[test]
    fn page_header_classes_match_reports() {
        let el: Element = rsx! {
            div {
                h1 { class: "text-4xl md:text-5xl font-bold tracking-tighter", "Calendar" }
                p { class: "text-muted-foreground", "Upcoming payments" }
            }
        };
        let s = format!("{el:?}");
        assert!(s.contains("md:text-5xl"));
        assert!(s.contains("font-bold"));
        assert!(s.contains("tracking-tighter"));
    }
}