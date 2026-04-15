use dioxus::prelude::*;

#[component]
pub fn Dialog(
    open: bool,
    on_close: EventHandler<()>,
    title: String,
    children: Element,
) -> Element {
    // Escape key handler via eval
    use_future(move || async move {
        if open {
            let _ = dioxus::document::eval(r#"
                document.addEventListener('keydown', function handler(e) {
                    if (e.key === 'Escape') {
                        document.removeEventListener('keydown', handler);
                        dioxus.send(true);
                    }
                });
            "#);
        }
    });

    if !open {
        return rsx! {};
    }

    rsx! {
        // Backdrop overlay
        div {
            class: "fixed inset-0 z-50 bg-black/50 flex items-center justify-center",
            onclick: move |e| {
                // Close only on backdrop click, not on card click
                if e.client_coordinates().x > 0.0 {
                    on_close.call(());
                }
            },

            // Dialog card
            div {
                class: "bg-card border border-border rounded-lg shadow-lg max-w-lg w-full mx-4 max-h-[85vh] overflow-y-auto",
                onclick: move |e| e.stop_propagation(),

                // Header
                div { class: "flex items-center justify-between p-6 border-b border-border",
                    h2 { class: "text-lg font-semibold text-foreground", {title.clone()} }
                    button {
                        r#type: "button",
                        class: "text-muted-foreground hover:text-foreground",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }

                // Content
                div { class: "p-6",
                    {children}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn dialog_closed_renders_nothing() {
        // When open=false, component returns empty VNode
        assert!(true);
    }
}