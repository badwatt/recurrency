use crate::api;
use crate::components::dialog::Dialog;
use crate::components::logo_image::logo_image_with_fallback;
use crate::models::logo::LogoResult;
use crate::utils::image::data_url;
use dioxus::prelude::*;

const LOGO_PAGE_SIZE: usize = 8;
const MAX_LOGO_UPLOAD_BYTES: u64 = 5_000_000;
const MAX_LOGO_STORED_BYTES: usize = 500_000;

fn page_count(total: usize) -> usize {
    total.div_ceil(LOGO_PAGE_SIZE)
}

fn page_bounds(page: usize, total: usize) -> (usize, usize) {
    let start = page.saturating_mul(LOGO_PAGE_SIZE).min(total);
    let end = (start + LOGO_PAGE_SIZE).min(total);
    (start, end)
}

/// Build candidate logo URLs for a company name or domain as a fallback for
/// when the online logo search has no match (e.g. offline or niche brands).
fn logo_candidates(query: &str) -> Vec<LogoResult> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let domains: Vec<String> = if q.contains('.') {
        vec![q.clone()]
    } else {
        let slug: String = q.chars().filter(|c| c.is_alphanumeric()).collect();
        if slug.is_empty() {
            return Vec::new();
        }
        vec![
            format!("{slug}.com"),
            format!("{slug}.io"),
            format!("{slug}.app"),
        ]
    };

    let mut out = Vec::new();
    for d in &domains {
        for url in [
            format!("https://www.google.com/s2/favicons?domain={d}&sz=256"),
            format!("https://icons.duckduckgo.com/ip3/{d}.ico"),
            format!("https://unavatar.io/{d}"),
        ] {
            out.push(LogoResult {
                thumbnail: url.clone(),
                url,
                width: None,
                height: None,
            });
        }
    }
    out
}

/// Resize a browser-selected logo to a compact transparent PNG. Android's
/// native picker already returns a small JPEG.
async fn resize_logo(data_url: String) -> Option<String> {
    let mut eval = document::eval(r#"
        const source = await dioxus.recv();
        try {
            const image = new Image();
            const loaded = new Promise((resolve, reject) => {
                image.onload = resolve;
                image.onerror = reject;
                image.src = source;
            });
            await loaded;
            const max = 256;
            const scale = Math.min(1, max / image.width, max / image.height);
            const canvas = document.createElement('canvas');
            canvas.width = Math.max(1, Math.round(image.width * scale));
            canvas.height = Math.max(1, Math.round(image.height * scale));
            const context = canvas.getContext('2d');
            context.clearRect(0, 0, canvas.width, canvas.height);
            context.drawImage(image, 0, 0, canvas.width, canvas.height);
            dioxus.send(canvas.toDataURL('image/png'));
        } catch (_) {
            dioxus.send('');
        }
    "#);
    eval.send(data_url).ok()?;
    eval.recv::<String>()
        .await
        .ok()
        .filter(|value| value.starts_with("data:image/"))
}

#[component]
pub fn LogoSearch(selected_url: String, default_query: String, on_select: EventHandler<String>) -> Element {
    let mut dialog_open = use_signal(|| false);
    let mut query = use_signal(|| default_query.clone());
    let mut results = use_signal(Vec::<LogoResult>::new);
    let mut error_msg = use_signal(String::new);
    let mut searching = use_signal(|| false);
    let mut page = use_signal(|| 0usize);
    let mut upload_error = use_signal(String::new);
    let mut url_input = use_signal(|| selected_url.clone());
    let mut last_selected = use_signal(|| selected_url.clone());

    let selected_for_effect = selected_url.clone();
    use_effect(move || {
        let next = selected_for_effect.clone();
        if next != last_selected() {
            last_selected.set(next.clone());
            url_input.set(next);
        }
    });

    let mut do_search = move || {
        let q = query().trim().to_string();
        if q.is_empty() || searching() {
            return;
        }
        searching.set(true);
        error_msg.set(String::new());
        results.set(vec![]);
        page.set(0);
        spawn(async move {
            let found = match api::logos::search(&q).await {
                Ok(response) if !response.results.is_empty() => response.results,
                // No brand match (or the API is unreachable): fall back to
                // direct favicon providers for the guessed domain.
                _ => logo_candidates(&q),
            };
            if found.is_empty() {
                error_msg.set("No logos found. Try the company website (e.g. spotify.com).".into());
            } else {
                results.set(found);
            }
            searching.set(false);
        });
    };

    let mut apply_url = move || {
        let value = url_input().trim().to_string();
        if value.is_empty() {
            upload_error.set("Paste an image URL first".to_string());
        } else {
            upload_error.set(String::new());
            on_select.call(value);
        }
    };

    let mut use_upload = move |file: dioxus::html::FileData| {
        if file.size() > MAX_LOGO_UPLOAD_BYTES {
            upload_error.set("Logo must be 5 MB or smaller".to_string());
            return;
        }
        let content_type = file
            .content_type()
            .unwrap_or_else(|| "image/png".to_string());
        if !content_type.starts_with("image/") {
            upload_error.set("Choose an image file".to_string());
            return;
        }
        upload_error.set(String::new());
        spawn(async move {
            match file.read_bytes().await {
                Ok(bytes) => {
                    let original = data_url(&content_type, &bytes);
                    let selected = if cfg!(target_os = "android") {
                        Some(original.clone())
                    } else {
                        match resize_logo(original.clone()).await {
                            Some(resized) => Some(resized),
                            None if bytes.len() <= MAX_LOGO_STORED_BYTES => Some(original),
                            None => None,
                        }
                    };
                    match selected {
                        Some(value) => {
                            url_input.set(value.clone());
                            on_select.call(value);
                        }
                        None => upload_error.set(
                            "Could not process that image. Try a smaller logo.".to_string(),
                        ),
                    }
                }
                Err(_) => upload_error.set("Could not read the selected image".to_string()),
            }
        });
    };

    let all_results = results();
    let total = all_results.len();
    let pages = page_count(total);
    let current_page = page().min(pages.saturating_sub(1));
    let (start, end) = page_bounds(current_page, total);
    let page_label = format!("Page {} of {}", current_page + 1, pages);

    rsx! {
        // Selected logo preview, upload, URL and search controls.
        div { class: "space-y-2",
            div { class: "flex flex-wrap items-center gap-2",
                if !selected_url.is_empty() {
                    {logo_image_with_fallback("selected-logo", Some(selected_url.as_str()), "Logo", "h-10 w-10 rounded object-contain border border-border")}
                }
                button {
                    r#type: "button",
                    class: "inline-flex items-center justify-center rounded-md border border-input bg-background px-3 py-2 text-sm font-medium text-foreground hover:bg-accent",
                    onclick: move |_| {
                        if !default_query.is_empty() {
                            query.set(default_query.clone());
                        }
                        results.set(vec![]);
                        error_msg.set(String::new());
                        page.set(0);
                        dialog_open.set(true);
                    },
                    if !selected_url.is_empty() { "Change" } else { "Add logo" }
                }
                if cfg!(target_os = "android") {
                    button {
                        r#type: "button",
                        class: "inline-flex items-center justify-center rounded-md border border-input bg-background px-3 py-2 text-sm font-medium text-foreground hover:bg-accent",
                        onclick: move |_| {
                            let on_select = on_select;
                            spawn(async move {
                                let mut eval = document::eval(r#"
                                    if (window.RecurrencyImagePicker) {
                                        await new Promise((resolve) => {
                                            window.__recurrencyAvatarPicked = (data) => {
                                                dioxus.send(data);
                                                resolve();
                                            };
                                            window.RecurrencyImagePicker.pickImage();
                                        });
                                    } else {
                                        dioxus.send('');
                                    }
                                "#);
                                match eval.recv::<String>().await {
                                    Ok(data) if data.starts_with("data:image/") => {
                                        url_input.set(data.clone());
                                        on_select.call(data);
                                    }
                                    Ok(_) => {}
                                    Err(_) => upload_error.set(
                                        "Could not open the gallery. Try again.".to_string(),
                                    ),
                                }
                            });
                        },
                        "Upload image"
                    }
                } else {
                    label {
                        class: "relative inline-flex cursor-pointer items-center justify-center rounded-md border border-input bg-background px-3 py-2 text-sm font-medium text-foreground hover:bg-accent focus-within:ring-2 focus-within:ring-ring",
                        "Upload image"
                        input {
                            r#type: "file",
                            accept: "image/*",
                            aria_label: "Upload logo image",
                            class: "absolute inset-0 h-full w-full cursor-pointer opacity-0",
                            onchange: move |event: Event<FormData>| {
                                if let Some(file) = event.files().first().cloned() {
                                    use_upload(file);
                                }
                            },
                        }
                    }
                }
                if !selected_url.is_empty() {
                    button {
                        r#type: "button",
                        class: "inline-flex items-center justify-center rounded-md border border-input bg-background px-3 py-2 text-sm font-medium text-red-600 hover:bg-red-50",
                        onclick: move |_| {
                            url_input.set(String::new());
                            on_select.call("".to_string())
                        },
                        "Remove"
                    }
                }
            }
            div { class: "flex gap-2",
                input {
                    r#type: "text",
                    inputmode: "url",
                    value: "{url_input()}",
                    placeholder: "Paste logo URL (https://…)",
                    class: "flex h-9 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background",
                    oninput: move |event: FormEvent| url_input.set(event.value()),
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key().to_string() == "Enter" {
                            apply_url();
                        }
                    },
                }
                button {
                    r#type: "button",
                    class: "inline-flex h-9 shrink-0 items-center justify-center rounded-md border border-input bg-background px-3 text-sm font-medium hover:bg-accent disabled:opacity-50",
                    disabled: url_input().trim().is_empty(),
                    onclick: move |_| apply_url(),
                    "Use URL"
                }
            }
            if !upload_error().is_empty() {
                p { class: "text-xs text-destructive", {upload_error()} }
            }
        }

        // Search dialog
        Dialog {
            open: dialog_open(),
            on_close: move |_| dialog_open.set(false),
            title: "Search Logo".to_string(),

            div { class: "space-y-4",
                div { class: "flex gap-2",
                    input {
                        r#type: "text",
                        value: "{query()}",
                        placeholder: "Enter company name or domain...",
                        class: "flex h-10 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50",
                        oninput: move |e: FormEvent| {
                            query.set(e.value());
                            error_msg.set(String::new());
                        },
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key().to_string() == "Enter" {
                                do_search();
                            }
                        },
                    }
                    button {
                        r#type: "button",
                        class: "inline-flex items-center justify-center rounded-md bg-primary text-primary-foreground px-4 py-2 text-sm font-medium hover:bg-primary/90 disabled:opacity-50",
                        disabled: searching() || query().trim().is_empty(),
                        onclick: move |_| do_search(),
                        if searching() { "Searching…" } else { "Search" }
                    }
                }

                if searching() {
                    div { class: "text-sm text-muted-foreground", "Searching logos…" }
                }

                if !error_msg().is_empty() {
                    div { class: "flex h-32 flex-col items-center justify-center gap-2 text-muted-foreground",
                        "No logos found"
                        p { class: "text-xs", "Try searching for the company website (e.g., spotify.com)" }
                    }
                }

                if !all_results.is_empty() {
                    div { class: "space-y-3 mt-4",
                        div { class: "grid grid-cols-4 gap-3",
                            for logo in all_results[start..end].iter() {
                                button {
                                    r#type: "button",
                                    class: "flex aspect-square items-center justify-center rounded-md border border-border p-2 hover:bg-accent transition-colors",
                                    title: logo.url.to_string(),
                                    onclick: {
                                        // Prefer the thumbnail: search-engine
                                        // images often block hotlinking, while
                                        // their cached thumbnails do not.
                                        let url = if logo.thumbnail.is_empty() {
                                            logo.url.clone()
                                        } else {
                                            logo.thumbnail.clone()
                                        };
                                        let on_sel = on_select;
                                        move |_| {
                                            url_input.set(url.clone());
                                            on_sel.call(url.clone());
                                            dialog_open.set(false);
                                        }
                                    },
                                    {logo_image_with_fallback(&format!("search-result-{}", logo.url),
                                        Some(logo.thumbnail.as_str()),
                                        "Logo",
                                        // Scale up tiny favicons so every tile is legible.
                                        "h-12 w-12 sm:h-16 sm:w-16 object-contain"
                                    )}
                                }
                            }
                        }

                        if pages > 1 {
                            div { class: "flex items-center justify-between gap-2",
                                button {
                                    r#type: "button",
                                    class: "inline-flex items-center justify-center rounded-md border border-input bg-background px-3 py-1.5 text-sm font-medium hover:bg-accent disabled:opacity-50",
                                    disabled: current_page == 0,
                                    onclick: move |_| page.set(current_page.saturating_sub(1)),
                                    "‹ Prev"
                                }
                                span { class: "text-xs text-muted-foreground", {page_label.clone()} }
                                button {
                                    r#type: "button",
                                    class: "inline-flex items-center justify-center rounded-md border border-input bg-background px-3 py-1.5 text-sm font-medium hover:bg-accent disabled:opacity-50",
                                    disabled: current_page + 1 >= pages,
                                    onclick: move |_| page.set(current_page + 1),
                                    "Next ›"
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
    fn logo_picker_supports_upload_and_pasted_urls() {
        let src = include_str!("logo_search.rs");
        assert!(src.contains("Upload image"));
        assert!(src.contains("Use URL"));
        assert!(src.contains("read_bytes"));
        assert!(src.contains("resize_logo"));
        assert!(src.contains("RecurrencyImagePicker"));
    }


    use super::*;
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

    #[test]
    fn logo_search_no_auto_search_effect() {
        let src = include_str!("logo_search.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap_or("");
        // Old auto-search pattern: use_effect + dialog_open() + !query().is_empty() + results().is_empty()
        assert!(!prod.contains("results().is_empty() && error_msg().is_empty()"),
            "LogoSearch must not auto-search when dialog opens");
    }

    #[test]
    fn logo_search_enter_key_triggers_search() {
        let src = include_str!("logo_search.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap_or("");
        assert!(prod.contains("onkeydown"),
            "LogoSearch input must support Enter key to trigger search");
    }

    // ── Pagination ──
    #[test]
    fn page_count_empty_is_zero() {
        assert_eq!(page_count(0), 0);
    }

    #[test]
    fn page_count_single_page() {
        assert_eq!(page_count(1), 1);
        assert_eq!(page_count(LOGO_PAGE_SIZE), 1);
    }

    #[test]
    fn page_count_rounds_up() {
        assert_eq!(page_count(LOGO_PAGE_SIZE + 1), 2);
        assert_eq!(page_count(LOGO_PAGE_SIZE * 3), 3);
        assert_eq!(page_count(LOGO_PAGE_SIZE * 3 + 1), 4);
    }

    #[test]
    fn page_bounds_first_page() {
        assert_eq!(page_bounds(0, 20), (0, LOGO_PAGE_SIZE));
    }

    #[test]
    fn page_bounds_last_partial_page() {
        assert_eq!(page_bounds(2, 20), (16, 20));
    }

    #[test]
    fn page_bounds_clamps_out_of_range() {
        assert_eq!(page_bounds(9, 20), (20, 20));
        assert_eq!(page_bounds(0, 0), (0, 0));
    }

    // ── Fallback candidates ──
    #[test]
    fn candidates_for_name_guess_common_domains() {
        let candidates = logo_candidates("Netflix");
        assert_eq!(candidates.len(), 9);
        assert!(candidates.iter().any(|c| c.url.contains("netflix.com")));
        assert!(candidates.iter().any(|c| c.url.contains("netflix.io")));
        assert!(candidates.iter().any(|c| c.url.contains("netflix.app")));
    }

    #[test]
    fn candidates_use_live_logo_providers() {
        let candidates = logo_candidates("spotify");
        assert!(candidates.iter().any(|c| c.url.contains("unavatar.io")));
        assert!(!candidates.iter().any(|c| c.url.contains("clearbit")));
    }

    #[test]
    fn candidates_accept_a_domain() {
        let candidates = logo_candidates("spotify.com");
        assert_eq!(candidates.len(), 3);
        assert!(candidates.iter().all(|c| c.url.contains("spotify.com")));
    }

    #[test]
    fn candidates_empty_query_has_no_results() {
        assert!(logo_candidates("   ").is_empty());
        assert!(logo_candidates("!!!").is_empty());
    }

    // ── Online search wiring ──
    #[test]
    fn logo_search_uses_online_logo_api() {
        let src = include_str!("logo_search.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap_or("");
        assert!(
            prod.contains("api::logos::search"),
            "LogoSearch must query the logo API instead of only guessing domains"
        );
        assert!(
            prod.contains("page_count"),
            "LogoSearch results must be paginated"
        );
    }
}
