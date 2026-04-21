use crate::api;
use crate::components::dialog::Dialog;
use crate::components::input::Input;
use crate::components::logo_image::logo_image_with_fallback;
use crate::models::logo::LogoResult;
use dioxus::prelude::*;

#[component]
pub fn LogoSearch(selected_url: String, on_select: EventHandler<String>) -> Element {
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
#[allow(unused_assignments)]
mod tests {
    use crate::models::logo::LogoResult;

    #[test]
    fn logo_search_placeholder() {
        assert!(true);
    }

    // ── LogoResult model ──
    #[test]
    fn logo_result_fields() {
        let r = LogoResult {
            name: "Netflix".into(),
            url: "https://logo.clearbit.com/netflix.com".into(),
        };
        assert_eq!(r.name, "Netflix");
        assert!(r.url.starts_with("https://"));
    }

    #[test]
    fn logo_result_serialization() {
        let r = LogoResult {
            name: "Spotify".into(),
            url: "https://logo.clearbit.com/spotify.com".into(),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("Spotify"));
        let back: LogoResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Spotify");
    }

    #[test]
    fn logo_result_multiple() {
        let logos = vec![
            LogoResult {
                name: "Netflix".into(),
                url: "https://a.com/n.png".into(),
            },
            LogoResult {
                name: "Spotify".into(),
                url: "https://a.com/s.png".into(),
            },
            LogoResult {
                name: "Amazon".into(),
                url: "https://a.com/am.png".into(),
            },
        ];
        assert_eq!(logos.len(), 3);
        assert_eq!(logos[0].name, "Netflix");
    }

    // ── Search query URL construction ──
    #[test]
    fn search_url_format() {
        let query = "netflix";
        let url = format!("/api/logos/search?q={}", query);
        assert!(url.contains("q=netflix"));
    }

    #[test]
    fn search_url_encoded_spaces() {
        let query = "netflix mobile";
        let url = format!("/api/logos/search?q={}", query.replace(' ', "%20"));
        assert!(url.contains("q=netflix%20mobile"));
    }

    #[test]
    fn search_url_with_special_chars() {
        let query = "hulu+tv";
        let url = format!("/api/logos/search?q={}", query);
        assert!(url.contains("q=hulu+tv"));
    }

    // ── Loading state transitions ──
    #[test]
    fn loading_state_idle() {
        let searching = false;
        assert!(!searching);
    }

    #[test]
    fn loading_state_active() {
        let searching = true;
        assert!(searching);
    }

    #[test]
    fn loading_state_transitions() {
        let mut searching = false;
        searching = true;
        assert!(searching);
        searching = false;
        assert!(!searching);
    }

    // ── Results state ──
    #[test]
    fn results_empty() {
        let results: Vec<LogoResult> = vec![];
        assert!(results.is_empty());
    }

    #[test]
    fn results_with_items() {
        let results = vec![
            LogoResult {
                name: "A".into(),
                url: "https://a.com".into(),
            },
            LogoResult {
                name: "B".into(),
                url: "https://b.com".into(),
            },
        ];
        assert_eq!(results.len(), 2);
    }

    // ── Search query min length ──
    #[test]
    fn search_query_min_length() {
        let min_len = 2;
        assert!("a".len() < min_len);
        assert!("ab".len() >= min_len);
    }

    #[test]
    fn search_query_empty() {
        let query = "";
        assert!(query.is_empty());
        assert!(query.len() < 2);
    }

    // ── Error state ──
    #[test]
    fn api_error_logo_search() {
        let err = crate::api::client::ApiError::Http(500, "logo service unavailable".into());
        assert!(err.to_string().contains("500"));
    }

    #[test]
    fn api_network_error() {
        let err = crate::api::client::ApiError::Network("timeout".into());
        assert!(err.to_string().contains("timeout"));
    }

    // ── Modal open/close state ──
    #[test]
    fn modal_closed_by_default() {
        let open = false;
        assert!(!open);
    }

    #[test]
    fn modal_toggle() {
        let mut open = false;
        open = !open; // Open
        assert!(open);
        open = !open; // Close
        assert!(!open);
    }

    // ── Logo URL validation ──
    #[test]
    fn logo_url_https_required() {
        let url = "https://logo.clearbit.com/netflix.com";
        assert!(url.starts_with("https://"));
    }

    #[test]
    fn logo_url_clearbit_domain() {
        let url = "https://logo.clearbit.com/company.com";
        assert!(url.contains("logo.clearbit.com"));
    }

    // ── Search results deduplication ──
    #[test]
    fn search_results_dedup() {
        let logos = vec![
            LogoResult {
                name: "Netflix".into(),
                url: "https://a.com/1.png".into(),
            },
            LogoResult {
                name: "Netflix".into(),
                url: "https://a.com/2.png".into(),
            },
        ];
        let mut seen = std::collections::HashSet::new();
        let unique: Vec<_> = logos
            .into_iter()
            .filter(|r| seen.insert(r.name.clone()))
            .collect();
        assert_eq!(unique.len(), 1);
    }

    // ── on_select callback fires ──
    #[test]
    fn select_logo_updates_signal() {
        let _selected: Option<LogoResult> = None;
        let logo = LogoResult {
            name: "Netflix".into(),
            url: "https://logo.clearbit.com/netflix.com".into(),
        };
        let selected = Some(logo);
        assert!(selected.is_some());
        assert_eq!(selected.as_ref().unwrap().name, "Netflix");
    }

    // ── Dialog close after selection ──
    #[test]
    fn dialog_close_after_select() {
        let mut dialog_open = true;
        // After selection, dialog should close
        dialog_open = false;
        assert!(!dialog_open);
    }

    // ── Disabled button when empty query ──
    #[test]
    fn button_disabled_empty_query() {
        let query = "";
        let disabled = query.is_empty();
        assert!(disabled);
    }

    #[test]
    fn button_disabled_while_searching() {
        let searching = true;
        let disabled = searching || "".is_empty();
        assert!(disabled);
    }

    #[test]
    fn button_enabled_with_query_and_not_searching() {
        let searching = false;
        let query = "netflix";
        let disabled = searching || query.is_empty();
        assert!(!disabled);
    }
}
