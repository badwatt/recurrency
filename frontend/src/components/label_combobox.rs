#![allow(clippy::suspicious_else_formatting)]
use dioxus::prelude::*;
use uuid::Uuid;

use crate::models::account::CreateAccountRequest;
use crate::models::category::CreateCategoryRequest;
use crate::state::accounts::ACCOUNTS;
use crate::state::categories::CATEGORIES;
use crate::storage::Storage;

/// Which label collection the combobox manages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelKind {
    Category,
    Account,
}

/// A category or account rendered by the combobox.
#[derive(Debug, Clone, PartialEq)]
pub struct LabelItem {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub color: String,
}

/// Shared searchable/create combobox. Used for categories and accounts so both
/// surfaces behave exactly the same way.
#[component]
pub fn LabelCombobox(
    kind: LabelKind,
    selected_id: String,
    on_change: Callback<String>,
    error: Option<String>,
) -> Element {
    let mut search = use_signal(String::new);
    let mut open = use_signal(|| false);
    let mut creating = use_signal(|| false);
    let mut create_name = use_signal(String::new);
    let mut creating_loading = use_signal(|| false);
    let mut create_error = use_signal(String::new);

    let items = match kind {
        LabelKind::Category => CATEGORIES()
            .categories
            .iter()
            .map(|c| LabelItem {
                id: c.id.to_string(),
                name: c.name.clone(),
                icon: c.icon.clone().unwrap_or_else(|| "Tag".into()),
                color: c.color.clone().unwrap_or_else(|| "#6b7280".into()),
            })
            .collect::<Vec<_>>(),
        LabelKind::Account => ACCOUNTS()
            .accounts
            .iter()
            .map(|a| LabelItem {
                id: a.id.to_string(),
                name: a.name.clone(),
                icon: a.icon.clone().unwrap_or_else(|| "Tag".into()),
                color: a.color.clone().unwrap_or_else(|| "#6b7280".into()),
            })
            .collect::<Vec<_>>(),
    };

    let (placeholder, search_placeholder, create_title, input_placeholder, empty_text) = match kind
    {
        LabelKind::Category => (
            "Select category...",
            "Search or create category...",
            "Create new category",
            "Category name",
            "No categories found",
        ),
        LabelKind::Account => (
            "Select account...",
            "Search or create account...",
            "Create new account",
            "Account name",
            "No accounts found",
        ),
    };

    let error = error.unwrap_or_default();
    let sel_id = selected_id.clone();
    let selected = items.iter().find(|item| item.id == sel_id);
    let display_text = selected.map(|item| item.name.clone()).unwrap_or_default();
    let display_color = selected
        .map(|item| item.color.clone())
        .unwrap_or_else(|| "#6b7280".into());

    let search_val = search();
    let filtered: Vec<&LabelItem> = items
        .iter()
        .filter(|item| {
            item.name
                .to_lowercase()
                .contains(&search_val.to_lowercase())
        })
        .collect();
    let exact_match = items
        .iter()
        .any(|item| item.name.to_lowercase() == search_val.to_lowercase());
    let show_create = !search_val.is_empty() && !exact_match;

    let do_create = use_callback(move |_: ()| {
        let name = create_name();
        if name.trim().is_empty() {
            return;
        }
        creating_loading.set(true);
        spawn(async move {
            match create_label(kind, name.trim().to_string()).await {
                Ok(id) => {
                    on_change.call(id.to_string());
                    open.set(false);
                    search.set(String::new());
                    create_error.set(String::new());
                    creating.set(false);
                }
                Err(e) => {
                    log::error!("Failed to create label: {}", e);
                    create_error.set(e);
                }
            }
            creating_loading.set(false);
        });
    });

    rsx! {
        div { class: "relative",
            button {
                class: if error.is_empty() {
                    "w-full h-9 rounded-md border bg-background px-2 text-sm flex items-center justify-between text-left hover:bg-muted"
                } else {
                    "w-full h-9 rounded-md border border-red-500 bg-background px-2 text-sm flex items-center justify-between text-left hover:bg-muted"
                },
                r#type: "button",
                onclick: move |_| open.toggle(),
                if sel_id.is_empty() {
                    span { class: "text-muted-foreground truncate", "{placeholder}" }
                } else {
                    div { class: "flex items-center gap-2 truncate",
                        span {
                            class: "inline-block h-3 w-3 rounded-full shrink-0",
                            style: "background-color: {display_color}",
                        }
                        span { class: "truncate", {display_text.to_string()} }
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

            if open() {
                div {
                    class: "absolute left-0 right-0 top-[calc(100%+4px)] z-50 rounded-md border bg-popover shadow-md",
                    onclick: move |e| e.stop_propagation(),

                    if creating() {
                        div { class: "p-3 space-y-3",
                            div { class: "text-sm font-medium", "{create_title}" },
                            input {
                                class: "w-full h-9 rounded-md border bg-background px-3 text-sm",
                                r#type: "text",
                                placeholder: "{input_placeholder}",
                                value: create_name.to_string(),
                                autofocus: true,
                                oninput: move |e: Event<FormData>| {
                                    create_name.set(e.value());
                                    create_error.set(String::new());
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() == Key::Enter {
                                        do_create.call(());
                                    }
                                },
                            }
                            if !create_error().is_empty() {
                                p { class: "text-xs text-red-500", "{create_error()}" }
                            }
                            div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2",
                                button {
                                    class: "px-3 py-1.5 text-sm rounded-md border hover:bg-muted",
                                    r#type: "button",
                                    disabled: creating_loading(),
                                    onclick: move |_| {
                                        creating.set(false);
                                        create_name.set(String::new());
                                        create_error.set(String::new());
                                    },
                                    "Cancel"
                                }
                                button {
                                    class: "px-3 py-1.5 text-sm rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50",
                                    r#type: "button",
                                    disabled: creating_loading() || create_name().trim().is_empty(),
                                    onclick: move |_| do_create.call(()),
                                    if creating_loading() { "Creating..." } else { "Create" }
                                }
                            }
                        }
                    } else {
                        div { class: "flex items-center border-b px-3",
                            input {
                                class: "w-full h-9 border-0 bg-transparent text-sm focus:outline-none focus:ring-0",
                                r#type: "text",
                                placeholder: "{search_placeholder}",
                                value: search_val.to_string(),
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

                        div { class: "max-h-[200px] overflow-y-auto py-1",
                            if filtered.is_empty() && !show_create {
                                div { class: "py-6 text-center text-sm text-muted-foreground",
                                    "{empty_text}"
                                }
                            }
                            for item in filtered.iter() {
                                {
                                    let item_id = item.id.clone();
                                    let item_name = item.name.clone();
                                    let item_icon = item.icon.clone();
                                    let item_color = item.color.clone();
                                    let is_selected = item_id == sel_id;
                                    let on_change_cb = on_change;
                                    let item_id_for_click = item_id.clone();
                                    rsx! {
                                        button {
                                            class: if is_selected {
                                                "relative flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent bg-accent"
                                            } else {
                                                "relative flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent"
                                            },
                                            r#type: "button",
                                            onclick: move |_| {
                                                on_change_cb.call(item_id_for_click.clone());
                                                search.set(String::new());
                                                open.set(false);
                                            },
                                            span {
                                                class: "inline-block h-3 w-3 rounded-full shrink-0",
                                                style: "background-color: {item_color}",
                                            }
                                            {crate::utils::icons::icon_display(&item_icon)}
                                            {item_name.to_string()}
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
                                                create_error.set(String::new());
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
                p { class: "text-xs text-red-500 mt-1", {error.to_string()} }
            }
        }
    }
}

/// Create the selected label kind and refresh its global list.
async fn create_label(kind: LabelKind, name: String) -> Result<Uuid, String> {
    let id = match kind {
        LabelKind::Category => Storage::create_category(&CreateCategoryRequest {
            name,
            icon: None,
            color: None,
        })
        .await
        .map(|category| category.id),
        LabelKind::Account => Storage::create_account(&CreateAccountRequest {
            name,
            icon: Some("Wallet".into()),
            color: Some("#6b7280".into()),
        })
        .await
        .map(|account| account.id),
    }
    .map_err(|e| e.to_string())?;

    match kind {
        LabelKind::Category => {
            if let Ok(data) = Storage::list_categories().await {
                CATEGORIES.write().set_data(data);
            }
        }
        LabelKind::Account => {
            if let Ok(data) = Storage::list_accounts().await {
                ACCOUNTS.write().set_data(data);
            }
        }
    }

    Ok(id)
}

#[component]
pub fn CategoryCombobox(
    selected_id: String,
    on_change: Callback<String>,
    error: Option<String>,
) -> Element {
    rsx! {
        LabelCombobox {
            kind: LabelKind::Category,
            selected_id,
            on_change,
            error,
        }
    }
}

#[component]
pub fn AccountCombobox(
    selected_id: String,
    on_change: Callback<String>,
    error: Option<String>,
) -> Element {
    rsx! {
        LabelCombobox {
            kind: LabelKind::Account,
            selected_id,
            on_change,
            error,
        }
    }
}

/// Pure function: filter items by search text (case-insensitive).
pub fn filter_items(items: &[LabelItem], search: &str) -> Vec<usize> {
    let search_lower = search.to_lowercase();
    items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.name.to_lowercase().contains(&search_lower))
        .map(|(index, _)| index)
        .collect()
}

/// Pure function: check if search text matches an existing item exactly.
pub fn is_exact_match(items: &[LabelItem], search: &str) -> bool {
    let search_lower = search.to_lowercase();
    items
        .iter()
        .any(|item| item.name.to_lowercase() == search_lower)
}

/// Pure function: determine whether to show the "Create..." option.
pub fn should_show_create(items: &[LabelItem], search: &str) -> bool {
    !search.is_empty() && !is_exact_match(items, search)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str) -> LabelItem {
        LabelItem {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            icon: "Tag".into(),
            color: "#6b7280".into(),
        }
    }

    #[test]
    fn filter_items_empty_search() {
        let items = vec![item("Bank"), item("Cash")];
        assert_eq!(filter_items(&items, ""), vec![0, 1]);
    }

    #[test]
    fn filter_items_partial_and_case_insensitive() {
        let items = vec![item("Bank"), item("Cash"), item("Savings")];
        assert_eq!(filter_items(&items, "an"), vec![0]);
        assert_eq!(filter_items(&items, "CASH"), vec![1]);
    }

    #[test]
    fn filter_items_no_match() {
        let items = vec![item("Bank")];
        assert!(filter_items(&items, "xyz").is_empty());
    }

    #[test]
    fn exact_match_is_case_insensitive() {
        let items = vec![item("Bank")];
        assert!(is_exact_match(&items, "bank"));
        assert!(!is_exact_match(&items, "Ban"));
    }

    #[test]
    fn show_create_only_for_new_non_empty_text() {
        let items = vec![item("Bank")];
        assert!(!should_show_create(&items, ""));
        assert!(!should_show_create(&items, "Bank"));
        assert!(should_show_create(&items, "Broker"));
    }

    #[test]
    fn label_kind_is_copy_and_eq() {
        let kind = LabelKind::Account;
        let copy = kind;
        assert_eq!(kind, copy);
    }
}
