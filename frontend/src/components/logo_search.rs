use dioxus::prelude::*;
use crate::api;
use crate::components::dialog::Dialog;
use crate::components::input::Input;
use crate::components::logo_image::logo_image_with_fallback;
use crate::models::logo::LogoResult;

#[component]
pub fn LogoSearch(
    selected_url: String,
    on_select: EventHandler<String>,
) -> Element {
    let mut dialog_open = use_signal(|| false);
    let mut query = use_signal(String::new);
    let mut results = use_signal(Vec::<LogoResult>::new);
    let mut searching = use_signal(|| false);

    rsx! {
        // Selected logo preview + button
        div { class: "flex items-center gap-3",
            if !selected_url.is_empty() {
                {logo_image_with_fallback("selected-logo", Some(selected_url.as_str()), "Logo", "h-10 w-10 rounded object-contain border border-border")}
            }
            button {
                r#type: "button",
                class: "inline-flex items-center justify-center rounded-md border border-input bg-background px-3 py-2 text-sm font-medium text-foreground hover:bg-accent",
                onclick: move |_| dialog_open.set(true),
                "Add logo"
            }
        }

        // Search dialog
        Dialog {
            open: dialog_open(),
            on_close: move |_| dialog_open.set(false),
            title: "Search Logo".to_string(),

            div { class: "space-y-4",
                Input {
                    value: query(),
                    on_input: move |v: String| query.set(v),
                    placeholder: "Search for a brand logo…".to_string(),
                    r#type: "text".to_string(),
                }

                button {
                    r#type: "button",
                    class: "inline-flex items-center justify-center rounded-md bg-primary text-primary-foreground px-4 py-2 text-sm font-medium hover:bg-primary/90 disabled:opacity-50",
                    disabled: searching() || query().is_empty(),
                    onclick: move |_| {
                        let q = query();
                        searching.set(true);
                        spawn(async move {
                            match api::logos::search(&q).await {
                                Ok(resp) => results.set(resp.results),
                                Err(_) => results.set(vec![]),
                            }
                            searching.set(false);
                        });
                    },
                    if searching() { "Searching…" } else { "Search" }
                }

                if !results().is_empty() {
                    div { class: "grid grid-cols-4 gap-3 mt-4",
                        for logo in results() {
                            button {
                                r#type: "button",
                                class: "flex flex-col items-center gap-1 rounded-md border border-border p-2 hover:bg-accent",
                                onclick: {
                                    let url = logo.url.clone();
                                    let on_sel = on_select.clone();
                                    move |_| {
                                        on_sel.call(url.clone());
                                        dialog_open.set(false);
                                    }
                                },
                                {logo_image_with_fallback(&format!("search-result-{}", logo.name), Some(logo.url.as_str()), &logo.name, "h-10 w-10 object-contain")}
                                span { class: "text-xs text-muted-foreground truncate w-full text-center",
                                    {logo.name.clone()}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn logo_search_placeholder() {
        assert!(true);
    }
}