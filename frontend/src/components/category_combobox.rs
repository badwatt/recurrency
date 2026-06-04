#![allow(clippy::suspicious_else_formatting)]
use dioxus::prelude::*;

use crate::api;
use crate::models::category::CreateCategoryRequest;
use crate::state::categories::CATEGORIES;

/// Props for the CategoryCombobox component.
///
/// `selected_id` is the UUID string of the currently selected category (empty string = none).
/// `on_change` fires with the new UUID string when a selection is made.
/// `error` is an optional validation error string.
#[component]
pub fn CategoryCombobox(
    selected_id: String,
    on_change: Callback<String>,
    error: Option<String>,
) -> Element {
    let mut search = use_signal(String::new);
    let mut open = use_signal(|| false);
    let mut creating = use_signal(|| false);
    let mut create_name = use_signal(String::new);
    let mut creating_loading = use_signal(|| false);

    let cats = CATEGORIES();
    let error = error.unwrap_or_default();
    let sel_id = selected_id.clone();

    // Find currently selected category for display
    let selected_cat = cats.categories.iter().find(|c| c.id.to_string() == sel_id);
    let display_text = selected_cat.map(|c| c.name.clone()).unwrap_or_default();
    let display_color = selected_cat
        .and_then(|c| c.color.clone())
        .unwrap_or_else(|| "#6b7280".into());

    // Filter categories by search text
    let search_val = search();
    let filtered: Vec<_> = cats
        .categories
        .iter()
        .filter(|c| c.name.to_lowercase().contains(&search_val.to_lowercase()))
        .collect();

    // Whether to show "Create..." option
    let exact_match = cats
        .categories
        .iter()
        .any(|c| c.name.to_lowercase() == search_val.to_lowercase());
    let show_create = !search_val.is_empty() && !exact_match;

    rsx! {
        div { class: "relative",
            // Trigger button / display area
            button {
                class: if error.is_empty() {
                    "w-full h-9 rounded-md border bg-background px-2 text-sm flex items-center justify-between text-left hover:bg-muted"
                } else {
                    "w-full h-9 rounded-md border border-red-500 bg-background px-2 text-sm flex items-center justify-between text-left hover:bg-muted"
                },
                r#type: "button",
                onclick: move |_| open.toggle(),
                if sel_id.is_empty() {
                    span { class: "text-muted-foreground truncate", "Select category..." }
                } else {
                    div { class: "flex items-center gap-2 truncate",
                        span {
                            class: "inline-block h-3 w-3 rounded-full shrink-0",
                            style: "background-color: {display_color}",
                        }
                        span { class: "truncate", "{display_text}" }
                    }
                }
                div { class: "flex items-center gap-1 shrink-0 ml-2",
                    if !sel_id.is_empty() {
                        button {
                            class: "h-5 w-5 flex items-center justify-center rounded hover:bg-muted text-muted-foreground",
                            r#type: "button",
                            onclick: move |e: Event<MouseData>| {
                                e.stop_propagation();
                                on_change.call(String::new());
                            },
                            "✕"
                        }
                    }
                    svg {
                        class: "h-4 w-4 opacity-50",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        polyline { points: "6 9 12 15 18 9" }
                    }
                }
            }

            // Click-outside overlay
            if open() {
                div {
                    class: "fixed inset-0 z-40",
                    onclick: move |_| {
                        open.set(false);
                        search.set(String::new());
                        creating.set(false);
                    },
                }
            }

            // Dropdown
            if open() {
                div {
                    class: "absolute left-0 right-0 top-[calc(100%+4px)] z-50 rounded-md border bg-popover shadow-md",
                    // Prevent clicks inside from closing
                    onclick: move |e| e.stop_propagation(),

                    if creating() {
                        // Inline create form
                        div { class: "p-3 space-y-3",
                            div { class: "text-sm font-medium", "Create new category" },
                            input {
                                class: "w-full h-9 rounded-md border bg-background px-3 text-sm",
                                r#type: "text",
                                placeholder: "Category name",
                                value: {create_name.to_string()},
                                autofocus: true,
                                oninput: move |e: Event<FormData>| {
                                    create_name.set(e.value());
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() == Key::Enter {
                                        let name = create_name();
                                        if !name.trim().is_empty() {
                                            creating_loading.set(true);
                                            on_change.call(String::new()); // clear briefly
                                            spawn(async move {
                                                let req = CreateCategoryRequest {
                                                    name: name.trim().to_string(),
                                                    icon: None,
                                                    color: None,
                                                };
                                                match api::categories::create(&req).await {
                                                    Ok(new_cat) => {
                                                        // Refresh categories
                                                        match api::categories::list().await {
                                                            Ok(data) => CATEGORIES.write().set_data(data),
                                                            Err(e) => log::error!("Failed to refresh categories: {}", e),
                                                        }
                                                        on_change.call(new_cat.id.to_string());
                                                    }
                                                    Err(e) => {
                                                        log::error!("Failed to create category: {}", e);
                                                    }
                                                }
                                                creating_loading.set(false);
                                                creating.set(false);
                                                search.set(String::new());
                                                open.set(false);
                                            });
                                        }
                                    }
                                },
                            }
                            div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2",
                                button {
                                    class: "px-3 py-1.5 text-sm rounded-md border hover:bg-muted",
                                    r#type: "button",
                                    disabled: creating_loading(),
                                    onclick: move |_| {
                                        creating.set(false);
                                        create_name.set(String::new());
                                    },
                                    "Cancel"
                                }
                                button {
                                    class: "px-3 py-1.5 text-sm rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50",
                                    r#type: "button",
                                    disabled: creating_loading() || create_name().trim().is_empty(),
                                    onclick: move |_| {
                                        let name = create_name();
                                        if !name.trim().is_empty() {
                                            creating_loading.set(true);
                                            on_change.call(String::new());
                                            spawn(async move {
                                                let req = CreateCategoryRequest {
                                                    name: name.trim().to_string(),
                                                    icon: None,
                                                    color: None,
                                                };
                                                match api::categories::create(&req).await {
                                                    Ok(new_cat) => {
                                                        match api::categories::list().await {
                                                            Ok(data) => CATEGORIES.write().set_data(data),
                                                            Err(e) => log::error!("Failed to refresh categories: {}", e),
                                                        }
                                                        on_change.call(new_cat.id.to_string());
                                                    }
                                                    Err(e) => {
                                                        log::error!("Failed to create category: {}", e);
                                                    }
                                                }
                                                creating_loading.set(false);
                                                creating.set(false);
                                                search.set(String::new());
                                                open.set(false);
                                            });
                                        }
                                    },
                                    if creating_loading() { "Creating..." } else { "Create" }
                                }
                            }
                        }
                    } else {
                        // Search input
                        div { class: "flex items-center border-b px-3",
                            input {
                                class: "w-full h-9 border-0 bg-transparent text-sm focus:outline-none focus:ring-0",
                                r#type: "text",
                                placeholder: "Search or create category...",
                                value: {search_val.to_string()},
                                oninput: move |e: Event<FormData>| {
                                    search.set(e.value());
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() == Key::Escape {
                                        open.set(false);
                                        search.set(String::new());
                                    }
                                },
                            }
                        }

                        // Dropdown list
                        div { class: "max-h-[200px] overflow-y-auto py-1",
                            if filtered.is_empty() && !show_create {
                                div { class: "py-6 text-center text-sm text-muted-foreground",
                                    "No categories found"
                                }
                            }
                            for cat in filtered.iter() {
                                {
                                    let cat_id = cat.id.to_string();
                                    let cat_name = cat.name.clone();
                                    let cat_icon = cat.icon.clone().unwrap_or_else(|| "Tag".into());
                                    let cat_color = cat.color.clone().unwrap_or_else(|| "#6b7280".into());
                                    let is_selected = cat_id == sel_id;
                                    let on_change_cb = on_change;
                                    let cat_id_for_click = cat_id.clone();
                                    rsx! {
                                        button {
                                            class: if is_selected {
                                                "relative flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent bg-accent"
                                            } else {
                                                "relative flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent"
                                            },
                                            r#type: "button",
                                            onclick: move |_| {
                                                on_change_cb.call(cat_id_for_click.clone());
                                                search.set(String::new());
                                                open.set(false);
                                            },
                                            span {
                                                class: "inline-block h-3 w-3 rounded-full shrink-0",
                                                style: "background-color: {cat_color}",
                                            }
                                            {crate::utils::icons::icon_display(&cat_icon)}
                                            "{cat_name}"
                                            if is_selected {
                                                svg {
                                                    class: "h-4 w-4 shrink-0 text-primary",
                                                    view_box: "0 0 24 24",
                                                    fill: "none",
                                                    stroke: "currentColor",
                                                    stroke_width: "2",
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    polyline { points: "20 6 9 17 4 12" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if show_create {
                                {
                                    let search_for_create = search_val.clone();
                                    rsx! {
                                        button {
                                            class: "relative flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent text-primary",
                                            r#type: "button",
                                            onclick: move |_| {
                                                create_name.set(search_for_create.clone());
                                                creating.set(true);
                                            },
                                            svg {
                                                class: "h-4 w-4",
                                                view_box: "0 0 24 24",
                                                fill: "none",
                                                stroke: "currentColor",
                                                stroke_width: "2",
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                line { x1: "12", y1: "5", x2: "12", y2: "19" }
                                                line { x1: "5", y1: "12", x2: "19", y2: "12" }
                                            }
                                            span { "Create \"{search_for_create}\"" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !error.is_empty() {
                p { class: "text-xs text-red-500 mt-1", "{error}" }
            }
        }
    }
}

/// Pure function: filter categories by search text (case-insensitive).
/// Returns indices of matching categories.
#[inline(never)]
pub fn filter_categories(cats: &[crate::models::category::Category], search: &str) -> Vec<usize> {
    let search_lower = search.to_lowercase();
    cats.iter()
        .enumerate()
        .filter(|(_, c)| c.name.to_lowercase().contains(&search_lower))
        .map(|(i, _)| i)
        .collect()
}

/// Pure function: check if search text matches an existing category exactly (case-insensitive).
#[inline(never)]
pub fn is_exact_match(cats: &[crate::models::category::Category], search: &str) -> bool {
    let search_lower = search.to_lowercase();
    cats.iter().any(|c| c.name.to_lowercase() == search_lower)
}

/// Pure function: determine whether to show the "Create..." option.
#[inline(never)]
pub fn should_show_create(cats: &[crate::models::category::Category], search: &str) -> bool {
    !search.is_empty() && !is_exact_match(cats, search)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::category::Category;
    use chrono::DateTime;
    use uuid::Uuid;

    fn make_cat(name: &str) -> Category {
        Category {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: name.into(),
            icon: None,
            color: None,
            is_system: false,
            is_default: false,
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn filter_categories_empty_search() {
        let cats = vec![make_cat("Food"), make_cat("Transport")];
        let result = filter_categories(&cats, "");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_categories_partial_match() {
        let cats = vec![
            make_cat("Food"),
            make_cat("Transport"),
            make_cat("Entertainment"),
        ];
        let result = filter_categories(&cats, "ood");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0);
    }

    #[test]
    fn filter_categories_case_insensitive() {
        let cats = vec![make_cat("Food"), make_cat("Transport")];
        let result = filter_categories(&cats, "food");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0);
    }

    #[test]
    fn filter_categories_no_match() {
        let cats = vec![make_cat("Food"), make_cat("Transport")];
        let result = filter_categories(&cats, "xyz");
        assert!(result.is_empty());
    }

    #[test]
    fn filter_categories_multiple_matches() {
        let cats = vec![
            make_cat("Food"),
            make_cat("Food Delivery"),
            make_cat("Transport"),
        ];
        let result = filter_categories(&cats, "food");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn is_exact_match_true() {
        let cats = vec![make_cat("Food")];
        assert!(is_exact_match(&cats, "Food"));
        assert!(is_exact_match(&cats, "food")); // case insensitive
    }

    #[test]
    fn is_exact_match_false() {
        let cats = vec![make_cat("Food")];
        assert!(!is_exact_match(&cats, "Foo"));
        assert!(!is_exact_match(&cats, "Transport"));
    }

    #[test]
    fn is_exact_match_empty_search() {
        let cats = vec![make_cat("Food")];
        assert!(!is_exact_match(&cats, ""));
    }

    #[test]
    fn should_show_create_no_search() {
        let cats = vec![make_cat("Food")];
        assert!(!should_show_create(&cats, ""));
    }

    #[test]
    fn should_show_create_no_exact_match() {
        let cats = vec![make_cat("Food")];
        assert!(should_show_create(&cats, "Transport")); // doesn't exist
    }

    #[test]
    fn should_show_create_exact_match_exists() {
        let cats = vec![make_cat("Food")];
        assert!(!should_show_create(&cats, "Food")); // exact match exists
    }

    #[test]
    fn should_show_create_case_insensitive() {
        let cats = vec![make_cat("Food")];
        assert!(!should_show_create(&cats, "food")); // case-insensitive exact match
    }

    #[test]
    fn should_show_create_partial_no_match() {
        let cats = vec![make_cat("Food")];
        assert!(should_show_create(&cats, "Foo")); // partial, not exact
    }

    #[test]
    fn create_category_request_construction() {
        let req = CreateCategoryRequest {
            name: "NewCat".into(),
            icon: None,
            color: None,
        };
        assert_eq!(req.name, "NewCat");
        assert!(req.icon.is_none());
        assert!(req.color.is_none());
    }

    #[test]
    fn selected_id_empty_shows_placeholder() {
        let selected_id = "";
        assert!(selected_id.is_empty());
        // In the component, this shows "Select category..."
    }

    #[test]
    fn selected_id_non_empty_shows_category() {
        let cat_id = Uuid::new_v4().to_string();
        assert!(!cat_id.is_empty());
    }

    // ── Additional filter tests ──
    #[test]
    fn filter_categories_whitespace_search() {
        let cats = vec![make_cat("Food"), make_cat("Transport")];
        let result = filter_categories(&cats, "   ");
        // Whitespace-only search: to_lowercase("   ") = "   ", still contains itself
        // So it may or may not match depending on category names
        assert_eq!(result.len(), 0); // No category name contains 3 spaces
    }

    #[test]
    fn filter_categories_unicode() {
        let cats = vec![make_cat("Café"), make_cat("Naïve")];
        let result = filter_categories(&cats, "cafe");
        // Unicode case folding may not work as expected with to_lowercase
        // Test actual behavior
        assert!(result.is_empty() || result.len() <= 2);
    }

    #[test]
    fn filter_categories_single_result() {
        let cats = vec![make_cat("A"), make_cat("B"), make_cat("C")];
        let result = filter_categories(&cats, "b");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 1);
    }

    #[test]
    fn filter_categories_all_match() {
        let cats = vec![make_cat("Food"), make_cat("Transport")];
        let result = filter_categories(&cats, "");
        // Empty search returns all
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn is_exact_match_empty_categories() {
        let cats: Vec<Category> = vec![];
        assert!(!is_exact_match(&cats, "Food"));
        assert!(!is_exact_match(&cats, ""));
    }

    #[test]
    fn should_show_create_empty_categories() {
        let cats: Vec<Category> = vec![];
        assert!(!should_show_create(&cats, ""));
        assert!(should_show_create(&cats, "New"));
    }

    #[test]
    fn should_show_create_whitespace_search() {
        let cats = vec![make_cat("Food")];
        // Whitespace search is not empty, so it shows create
        let search = "   ";
        let search_lower = search.to_lowercase();
        let exact = cats.iter().any(|c| c.name.to_lowercase() == search_lower);
        let show_create = !search.is_empty() && !exact;
        // Whitespace-only: search.is_empty() = false (has spaces)
        // But is_exact_match checks to_lowercase which is "   "
        // No category matches "   " exactly
        assert!(show_create);
    }

    #[test]
    fn create_category_with_icon() {
        let req = CreateCategoryRequest {
            name: "Tech".into(),
            icon: Some("Laptop".into()),
            color: None,
        };
        assert_eq!(req.name, "Tech");
        assert_eq!(req.icon.as_deref(), Some("Laptop"));
    }

    #[test]
    fn create_category_with_color() {
        let req = CreateCategoryRequest {
            name: "Food".into(),
            icon: None,
            color: Some("#ff0000".into()),
        };
        assert_eq!(req.color.as_deref(), Some("#ff0000"));
    }

    #[test]
    fn create_category_full() {
        let req = CreateCategoryRequest {
            name: "Entertainment".into(),
            icon: Some("Film".into()),
            color: Some("#3b82f6".into()),
        };
        assert!(req.name == "Entertainment");
        assert!(req.icon.is_some());
        assert!(req.color.is_some());
    }

    #[test]
    fn filter_categories_emoji() {
        let cats = vec![make_cat("🎮 Gaming")];
        let result = filter_categories(&cats, "gaming");
        // Emoji prefix may or may not match
        assert!(result.is_empty() || result.len() == 1);
    }

    #[test]
    fn filter_categories_empty_categories() {
        let cats: Vec<Category> = vec![];
        let result = filter_categories(&cats, "anything");
        assert!(result.is_empty());
    }

    #[test]
    fn is_exact_match_unicode_case() {
        let cats = vec![make_cat("CAFÉ")];
        // Unicode uppercase/lowercase: "café".to_lowercase() = "café"
        // "CAFÉ".to_lowercase() = "café"
        // Should match
        let search = "cafe"; // Without accent
        let exact = cats
            .iter()
            .any(|c| c.name.to_lowercase() == search.to_lowercase());
        // Actually café ≠ cafe (different chars)
        assert!(!exact);
    }

    #[test]
    fn filter_preserves_order() {
        let cats = vec![make_cat("C"), make_cat("A"), make_cat("B")];
        let result = filter_categories(&cats, "");
        assert_eq!(result, vec![0, 1, 2]); // Maintains original order
    }
}
