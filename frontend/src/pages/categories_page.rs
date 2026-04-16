use dioxus::prelude::*;
use crate::state::categories::CATEGORIES;
use crate::api::categories;
use crate::models::category::*;
use crate::utils::icons::ICON_NAMES;
use uuid::Uuid;

/// Available color palette for category picker.
fn icon_tag() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "32", height: "32", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
        path { d: "M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z" }
        line { x1: "7", y1: "7", x2: "7.01", y2: "7" }
    }}
}

fn icon_star() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
        polygon { points: "12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" }
    }}
}

fn icon_pencil() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
        path { d: "M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z" }
    }}
}

fn icon_trash_2() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
        polyline { points: "3 6 5 6 21 6" }
        path { d: "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" }
        line { x1: "10", y1: "11", x2: "10", y2: "17" }
        line { x1: "14", y1: "11", x2: "14", y2: "17" }
    }}
}

fn icon_plus() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
        line { x1: "12", y1: "5", x2: "12", y2: "19" }
        line { x1: "5", y1: "12", x2: "19", y2: "12" }
    }}
}

const COLOR_PALETTE: &[&str] = &[
    "#ef4444", "#f97316", "#f59e0b", "#eab308",
    "#84cc16", "#22c55e", "#10b981", "#14b8a6",
    "#06b6d4", "#0ea5e9", "#3b82f6", "#6366f1",
    "#8b5cf6", "#a855f7", "#d946ef", "#ec4899",
    "#f43f5e", "#6b7280", "#78716c", "#a3a3a3",
];

#[component]
pub fn CategoriesPage() -> Element {
    let mut dialog_open = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<Uuid>);
    let mut form_name = use_signal(String::new);
    let mut form_icon = use_signal(|| "Tag".to_string());
    let mut form_color = use_signal(|| "#6b7280".to_string());
    let mut form_saving = use_signal(|| false);
    let mut show_icon_picker = use_signal(|| false);

    // Validation error signals
    let mut name_error = use_signal(String::new);

    // Load data
    use_future(move || async move {
        if !CATEGORIES().categories.is_empty() { return; }
        CATEGORIES.write().set_loading();
        match categories::list().await {
            Ok(data) => CATEGORIES.write().set_data(data),
            Err(e) => CATEGORIES.write().set_error(e.to_string()),
        }
    });

    let state = CATEGORIES();

    let open_create = move |_| {
        editing_id.set(None);
        form_name.set(String::new());
        form_icon.set("Tag".into());
        form_color.set("#6b7280".into());
        show_icon_picker.set(false);
        name_error.set(String::new());
        dialog_open.set(true);
    };

    let mut open_edit = move |cat: &Category| {
        editing_id.set(Some(cat.id));
        form_name.set(cat.name.clone());
        form_icon.set(cat.icon.clone().unwrap_or_else(|| "Tag".into()));
        form_color.set(cat.color.clone().unwrap_or_else(|| "#6b7280".into()));
        show_icon_picker.set(false);
        name_error.set(String::new());
        dialog_open.set(true);
    };

    let handle_save = move |_| {
        // Validate name
        let name_val = form_name();
        if name_val.trim().is_empty() {
            name_error.set("Name is required".into());
            return;
        } else if name_val.trim().len() < 2 {
            name_error.set("Name must be at least 2 characters".into());
            return;
        }

        let icon = form_icon();
        let color = form_color();
        let editing = editing_id();

        form_saving.set(true);
        spawn(async move {
            let result = if let Some(id) = editing {
                categories::update(&id, &UpdateCategoryRequest {
                    name: Some(name_val),
                    icon: Some(icon),
                    color: Some(color),
                }).await
            } else {
                categories::create(&CreateCategoryRequest {
                    name: name_val,
                    icon: Some(icon),
                    color: Some(color),
                }).await
            };

            match result {
                Ok(_) => {
                    dialog_open.set(false);
                    match categories::list().await {
                        Ok(data) => CATEGORIES.write().set_data(data),
                        Err(e) => CATEGORIES.write().set_error(e.to_string()),
                    }
                }
                Err(e) => log::error!("Save failed: {}", e),
            }
            form_saving.set(false);
        });
    };

    let handle_delete = move |id: Uuid| {
        spawn(async move {
            let _ = categories::delete(&id).await;
            match categories::list().await {
                Ok(data) => CATEGORIES.write().set_data(data),
                Err(e) => CATEGORIES.write().set_error(e.to_string()),
            }
        });
    };

    let handle_set_default = move |id: Uuid| {
        spawn(async move {
            let _ = categories::set_default(&id).await;
            match categories::list().await {
                Ok(data) => CATEGORIES.write().set_data(data),
                Err(e) => CATEGORIES.write().set_error(e.to_string()),
            }
        });
    };

    if let Some(err) = &state.error {
        return rsx! {
            div { class: "p-4 md:p-6",
                div { class: "rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-destructive",
                    {err.clone()}
                }
            }
        };
    }

    rsx! {
        div { class: "p-4 md:p-6 space-y-6",
            // Header
            div { class: "flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3",
                div {
                    h1 { class: "text-3xl font-bold tracking-tight", "Categories" }
                    p { class: "text-muted-foreground", "Organize your expenses with categories" }
                }
                button {
                    onclick: open_create,
                    class: "inline-flex items-center justify-center gap-2 h-10 px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 w-full sm:w-auto",
                    {icon_plus()} "Add Category"
                }
            }

            {if state.is_loading && state.categories.is_empty() {
                rsx! {
                    div { class: "flex h-64 items-center justify-center",
                        div { class: "h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" }
                    }
                }
            } else if state.categories.is_empty() {
                rsx! {
                    div { class: "rounded-lg border border-border bg-card p-8 flex flex-col items-center justify-center gap-4",
                        {icon_tag()}
                        div { class: "text-center",
                            p { class: "text-lg font-medium", "No categories yet" }
                            p { class: "text-sm text-muted-foreground", "Create categories to organize your subscriptions and transactions" }
                        }
                        button {
                            onclick: open_create,
                            class: "inline-flex items-center gap-2 h-10 px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90",
                            {icon_plus()} "Add Category"
                        }
                    }
                }
            } else {
                rsx! {
                    div { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4",
                        for cat in &state.categories {
                            {let cat_color = cat.color.clone().unwrap_or_else(|| "#888".into());
                             let cat_bg = format!("{}20", cat_color);
                             let cat_icon = cat.icon.clone().unwrap_or_else(|| "Tag".into());
                             let cat_id = cat.id;
                             let is_default = cat.is_default;
                             rsx! {
                                div { key: "{cat.id}", class: "rounded-lg border border-border bg-card p-3 sm:p-4",
                                    div { class: "flex items-center gap-2 sm:gap-3",
                                        // Icon circle
                                        div {
                                            class: "flex h-10 w-10 sm:h-12 sm:w-12 shrink-0 items-center justify-center rounded-full",
                                            style: "background-color: {cat_bg}",
                                            span {
                                                style: "color: {cat_color}",
                                                class: "text-lg sm:text-xl",
                                                {icon_display(&cat_icon)}
                                            }
                                        }
                                        // Name
                                        div { class: "flex-1 min-w-0",
                                            p { class: "font-medium truncate text-sm sm:text-base", "{cat.name}" }
                                            if is_default {
                                                span { class: "text-xs text-muted-foreground", "Default" }
                                            }
                                        }
                                        // Actions
                                        div { class: "flex gap-0.5 sm:gap-1 shrink-0",
                                            if !is_default {
                                                button {
                                                    class: "h-8 w-8 sm:h-9 sm:w-9 inline-flex items-center justify-center rounded-md hover:bg-accent",
                                                    title: "Set as default",
                                                    onclick: move |_| handle_set_default(cat_id),
                                                    {icon_star()}
                                                }
                                            }
                                            button {
                                                class: "h-8 w-8 sm:h-9 sm:w-9 inline-flex items-center justify-center rounded-md hover:bg-accent",
                                                onclick: {
                                                    let cat = cat.clone();
                                                    move |_| open_edit(&cat)
                                                },
                                                {icon_pencil()}
                                            }
                                            if !is_default {
                                                button {
                                                    class: "h-8 w-8 sm:h-9 sm:w-9 inline-flex items-center justify-center rounded-md hover:bg-accent text-destructive",
                                                    onclick: move |_| handle_delete(cat_id),
                                                    {icon_trash_2()}
                                                }
                                            }
                                        }
                                    }
                                }
                            }}
                        }
                    }
                }
            }}

            // Dialog
            if dialog_open() {
                div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
                    div { class: "w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-lg mx-4 max-h-[90vh] overflow-y-auto",
                        h2 { class: "text-lg font-semibold mb-4",
                            if editing_id().is_some() { "Edit Category" } else { "Add Category" }
                        }
                        div { class: "space-y-4",
                            // Name
                            div { class: "space-y-2",
                                label { class: "text-sm font-medium", "Name" }
                                input {
                                    r#type: "text",
                                    value: "{form_name}",
                                    oninput: move |e| {
                                        form_name.set(e.value());
                                        name_error.set(String::new());
                                    },
                                    class: if name_error().is_empty() { "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm" } else { "flex h-10 w-full rounded-md border border-red-500 bg-background px-3 py-2 text-sm" },
                                }
                                if !name_error().is_empty() {
                                    p { class: "text-xs text-red-500", "{name_error()}" }
                                }
                            }
                            // Icon
                            div { class: "space-y-2",
                                div { class: "flex items-center justify-between",
                                    label { class: "text-sm font-medium", "Icon" }
                                    button {
                                        onclick: move |_| show_icon_picker.toggle(),
                                        class: "text-sm text-primary hover:underline",
                                        if show_icon_picker() { "Hide icons" } else { "Choose icon" }
                                    }
                                }
                                div {
                                    class: "flex h-10 w-10 items-center justify-center rounded-full",
                                    style: "background-color: {form_color()}20",
                                    span { style: "color: {form_color()}", {icon_display(&form_icon())} }
                                }
                                if show_icon_picker() {
                                    div { class: "grid grid-cols-8 gap-1 p-2 border border-border rounded-md max-h-48 overflow-y-auto",
                                        for icon_name in ICON_NAMES {
                                            {let is_selected = *icon_name == form_icon();
                                             rsx! {
                                                button {
                                                    key: "{icon_name}",
                                                    class: if is_selected { "h-8 w-8 flex items-center justify-center rounded border-2 border-primary text-sm" } else { "h-8 w-8 flex items-center justify-center rounded border border-border hover:bg-accent text-sm" },
                                                    onclick: move |_| form_icon.set(icon_name.to_string()),
                                                    {icon_display(icon_name)}
                                                }
                                            }}
                                        }
                                    }
                                }
                            }
                            // Color
                            div { class: "space-y-2",
                                label { class: "text-sm font-medium", "Color" }
                                div { class: "grid grid-cols-10 gap-1",
                                    for color in COLOR_PALETTE {
                                        {let is_selected = *color == form_color();
                                         rsx! {
                                            button {
                                                key: "{color}",
                                                class: if is_selected { "h-8 w-8 rounded-full border-2 border-foreground ring-2 ring-ring" } else { "h-8 w-8 rounded-full border border-border hover:ring-1 hover:ring-ring" },
                                                style: "background-color: {color}",
                                                onclick: move |_| form_color.set(color.to_string()),
                                            }
                                        }}
                                    }
                                }
                            }
                            // Actions
                            div { class: "flex justify-end gap-2 pt-2",
                                button {
                                    onclick: move |_| dialog_open.set(false),
                                    class: "h-10 px-4 py-2 rounded-md border border-border text-sm font-medium hover:bg-accent",
                                    "Cancel"
                                }
                                button {
                                    onclick: handle_save,
                                    disabled: form_saving(),
                                    class: "h-10 px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50",
                                    if form_saving() { "Saving..." } else { "Save" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Map icon name to a display character (emoji/text shorthand).
/// Since lucide-dioxus isn't available, we use text representations.
fn icon_display(name: &str) -> String {
    match name {
        "Home" => "🏠",
        "ShoppingBag" => "🛍",
        "Coffee" => "☕",
        "Car" => "🚗",
        "Heart" => "❤",
        "Film" => "🎬",
        "Music" => "🎵",
        "Gamepad2" => "🎮",
        "Briefcase" => "💼",
        "GraduationCap" => "🎓",
        "Plane" => "✈",
        "Zap" => "⚡",
        "Wifi" => "📶",
        "Smartphone" => "📱",
        "Laptop" => "💻",
        "Monitor" => "🖥",
        "Cloud" => "☁",
        "Database" => "🗄",
        "Server" => "🖧",
        "Lock" => "🔒",
        "Shield" => "🛡",
        "CreditCard" => "💳",
        "Wallet" => "👛",
        "Gift" => "🎁",
        "Star" => "⭐",
        "Sun" => "☀",
        "Moon" => "🌙",
        "CloudRain" => "🌧",
        "Thermometer" => "🌡",
        "Activity" => "📈",
        "Pill" => "💊",
        "Stethoscope" => "🩺",
        "Baby" => "👶",
        "Flower" => "🌸",
        "TreePine" => "🌲",
        "Utensils" => "餐具",
        "Beer" => "🍺",
        "Wine" => "🍷",
        "Tag" => "🏷",
        _ => "🏷",
    }.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_display_known() {
        assert_eq!(icon_display("Home"), "🏠");
        assert_eq!(icon_display("Film"), "🎬");
        assert_eq!(icon_display("Tag"), "🏷");
    }

    #[test]
    fn icon_display_unknown() {
        assert_eq!(icon_display("NonExistent"), "🏷");
    }

    #[test]
    fn color_palette_not_empty() {
        assert!(!COLOR_PALETTE.is_empty());
        assert_eq!(COLOR_PALETTE.len(), 20);
    }

    #[test]
    fn icon_names_not_empty() {
        assert!(!ICON_NAMES.is_empty());
    }
}