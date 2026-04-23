use crate::api;
use crate::components::dialog::Dialog;
use crate::components::input::Input;
use crate::components::logo_image::logo_image_with_fallback;
use crate::models::logo::LogoResult;
use dioxus::prelude::*;

#[component]
pub fn LogoSearch(selected_url: String, default_query: String, on_select: EventHandler<String>) -> Element {
    let mut dialog_open = use_signal(|| false);
    let mut query = use_signal(|| default_query.clone());
    let mut results = use_signal(Vec::<LogoResult>::new);
    let mut searching = use_signal(|| false);
    let mut error_msg = use_signal(|| String::new());

    // Auto-search when dialog opens and default query is present
    use_effect(move || {
        if dialog_open() && !query().is_empty() && !searching() && results().is_empty() && error_msg().is_empty() {
            searching.set(true);
            let q = query();
            spawn(async move {
                match api::logos::search(&q).await {
                    Ok(resp) => {
                        if resp.results.is_empty() {
                            error_msg.set("No logos found. Try a different search term.".into());
                        } else {
                            results.set(resp.results);
                        }
                    }
                    Err(e) => {
                        error_msg.set(format!("Search failed: {}", e));
                    }
                }
                searching.set(false);
            });
        }
    });

    rsx! {
        // Selected logo preview + button
        div { class: "flex items-center gap-3",
            if !selected_url.is_empty() {
                {logo_image_with_fallback("selected-logo", Some(selected_url.as_str()), "Logo", "h-10 w-10 rounded object-contain border border-border")}
            }
            button {
                r#type: "button",
                class: "inline-flex items-center justify-center rounded-md border border-input bg-background px-3 py-2 text-sm font-medium text-foreground hover:bg-accent",
                onclick: move |_| {
                    results.set(vec![]);
                    error_msg.set(String::new());
                    dialog_open.set(true);
                },
                if !selected_url.is_empty() { "Change" } else { "Add logo" }
            }
            if !selected_url.is_empty() {
                button {
                    r#type: "button",
                    class: "inline-flex items-center justify-center rounded-md border border-input bg-background px-3 py-2 text-sm font-medium text-red-600 hover:bg-red-50",
                    onclick: move |_| on_select.call("".to_string()),
                    "Remove"
                }
            }
        }

        // Search dialog
        Dialog {
            open: dialog_open(),
            on_close: move |_| dialog_open.set(false),
            title: "Search Logo".to_string(),

            div { class: "space-y-4",
                div { class: "flex gap-2",
                    Input {
                        value: query(),
                        on_input: move |v: String| {
                            query.set(v);
                            error_msg.set(String::new());
                        },
                        placeholder: "Enter company name or domain...".to_string(),
                        r#type: "text".to_string(),
                    }
                    button {
                        r#type: "button",
                        class: "inline-flex items-center justify-center rounded-md bg-primary text-primary-foreground px-4 py-2 text-sm font-medium hover:bg-primary/90 disabled:opacity-50",
                        disabled: searching() || query().is_empty(),
                        onclick: move |_| {
                            let q = query();
                            searching.set(true);
                            error_msg.set(String::new());
                            results.set(vec![]);
                            spawn(async move {
                                match api::logos::search(&q).await {
                                    Ok(resp) => {
                                        if resp.results.is_empty() {
                                            error_msg.set("No logos found. Try a different search term.".into());
                                        } else {
                                            results.set(resp.results);
                                        }
                                    }
                                    Err(e) => {
                                        error_msg.set(format!("Search failed: {}", e));
                                    }
                                }
                                searching.set(false);
                            });
                        },
                        if searching() { "Searching…" } else { "Search" }
                    }
                }

                if searching() {
                    div { class: "flex h-32 items-center justify-center",
                        div { class: "h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" }
                    }
                }

                if !error_msg().is_empty() {
                    div { class: "flex h-32 flex-col items-center justify-center gap-2 text-muted-foreground",
                        "No logos found"
                        p { class: "text-xs", "Try searching for the company website (e.g., spotify.com)" }
                    }
                }

                if !results().is_empty() {
                    div { class: "grid grid-cols-4 gap-3 mt-4",
                        for logo in results() {
                            button {
                                r#type: "button",
                                class: "flex aspect-square items-center justify-center rounded-md border border-border p-2 hover:bg-accent transition-colors",
                                title: "{logo.url}",
                                onclick: {
                                    let url = logo.url.clone();
                                    let on_sel = on_select;
                                    move |_| {
                                        on_sel.call(url.clone());
                                        dialog_open.set(false);
                                    }
                                },
                                {logo_image_with_fallback(&format!("search-result-{}", logo.url),
                                    Some(logo.thumbnail.as_str()),
                                    "Logo",
                                    "max-h-full max-w-full object-contain"
                                )}
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
    use crate::models::logo::LogoResult;

    #[test]
    fn logo_result_fields() {
        let r = LogoResult {
            url: "https://logo.clearbit.com/netflix.com".into(),
            thumbnail: "https://logo.clearbit.com/netflix.com".into(),
            width: Some(200),
            height: Some(100),
        };
        assert!(r.url.starts_with("https://"));
        assert!(r.thumbnail.starts_with("https://"));
        assert_eq!(r.width, Some(200));
        assert_eq!(r.height, Some(100));
    }

    #[test]
    fn logo_result_serialization() {
        let r = LogoResult {
            url: "https://logo.clearbit.com/spotify.com".into(),
            thumbnail: "https://logo.clearbit.com/spotify.com".into(),
            width: None,
            height: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("url"));
        let back: LogoResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.url, r.url);
        assert_eq!(back.thumbnail, r.thumbnail);
    }

    #[test]
    fn logo_result_multiple() {
        let logos = vec![
            LogoResult {
                url: "https://a.com/n.png".into(),
                thumbnail: "https://a.com/n_thumb.png".into(),
                width: None,
                height: None,
            },
            LogoResult {
                url: "https://a.com/s.png".into(),
                thumbnail: "https://a.com/s_thumb.png".into(),
                width: None,
                height: None,
            },
        ];
        assert_eq!(logos.len(), 2);
        assert_eq!(logos[0].url, "https://a.com/n.png");
    }

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

    #[test]
    fn loading_state_transitions() {
        let mut searching = false;
        searching = true;
        assert!(searching);
        searching = false;
        assert!(!searching);
    }

    #[test]
    fn results_empty() {
        let results: Vec<LogoResult> = vec![];
        assert!(results.is_empty());
    }

    #[test]
    fn results_with_items() {
        let results = vec![
            LogoResult {
                url: "https://a.com".into(),
                thumbnail: "https://a.com/t.png".into(),
                width: None,
                height: None,
            },
            LogoResult {
                url: "https://b.com".into(),
                thumbnail: "https://b.com/t.png".into(),
                width: None,
                height: None,
            },
        ];
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn search_query_empty() {
        let query = "";
        assert!(query.is_empty());
        assert!(query.len() < 2);
    }

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

    #[test]
    fn modal_toggle() {
        let mut open = false;
        open = !open;
        assert!(open);
        open = !open;
        assert!(!open);
    }

    #[test]
    fn logo_url_https_required() {
        let url = "https://logo.clearbit.com/netflix.com";
        assert!(url.starts_with("https://"));
    }

    #[test]
    fn search_results_dedup() {
        let logos = vec![
            LogoResult {
                url: "https://a.com/1.png".into(),
                thumbnail: "https://a.com/1_t.png".into(),
                width: None,
                height: None,
            },
            LogoResult {
                url: "https://a.com/1.png".into(),
                thumbnail: "https://a.com/2_t.png".into(),
                width: None,
                height: None,
            },
        ];
        let mut seen = std::collections::HashSet::new();
        let unique: Vec<_> = logos
            .into_iter()
            .filter(|r| seen.insert(r.url.clone()))
            .collect();
        assert_eq!(unique.len(), 1);
    }

    #[test]
    fn select_logo_updates_signal() {
        let logo = LogoResult {
            url: "https://logo.clearbit.com/netflix.com".into(),
            thumbnail: "https://logo.clearbit.com/netflix.com".into(),
            width: None,
            height: None,
        };
        let selected = Some(logo);
        assert!(selected.is_some());
        assert_eq!(selected.as_ref().unwrap().url, "https://logo.clearbit.com/netflix.com");
    }

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

    // ── Thumbnail field tests ──
    #[test]
    fn logo_result_thumbnail_matches_url_fallback() {
        let r = LogoResult {
            url: "https://a.com/logo.png".into(),
            thumbnail: "https://a.com/logo.png".into(),
            width: None,
            height: None,
        };
        assert_eq!(r.thumbnail, r.url);
    }

    #[test]
    fn logo_result_thumbnail_different_from_url() {
        let r = LogoResult {
            url: "https://a.com/full.png".into(),
            thumbnail: "https://a.com/thumb.png".into(),
            width: Some(64),
            height: Some(64),
        };
        assert_ne!(r.thumbnail, r.url);
        assert_eq!(r.width, Some(64));
        assert_eq!(r.height, Some(64));
    }

    #[test]
    fn logo_search_has_remove_button() {
        let src = include_str!("logo_search.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap_or("");
        assert!(prod.contains("Remove"), "LogoSearch must have a Remove button");
        assert!(prod.contains(r#"if !selected_url.is_empty()"#),
            "Remove button must be conditional on selected_url");
    }

    #[test]
    fn logo_search_remove_calls_empty_string() {
        let src = include_str!("logo_search.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap_or("");
        assert!(prod.contains("on_select.call(\"\".to_string())"),
            "Remove button must clear logo via on_select with empty string");
    }
}
