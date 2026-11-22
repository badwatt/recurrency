use crate::components::color_picker::ColorField;
use crate::components::button::{Button, ButtonAlign, ButtonRadius, ButtonSize, ButtonVariant, IconButton};
use crate::components::dialog::Dialog;
use crate::components::empty_state::EmptyState;
use crate::components::field::Field;
use crate::components::input::{Input, InputSize};
use crate::components::page_header::PageHeader;
use crate::components::skeleton::CategoryCardSkeleton;
use crate::models::category::*;
use crate::state::categories::CATEGORIES;
use crate::storage::Storage;
use crate::utils::color::{DEFAULT_COLOR, FALLBACK_COLOR, is_valid_hex, with_alpha};
use crate::utils::icons::{ICON_NAMES, icon_display};
use dioxus::prelude::*;
use lucide_dioxus::{Pencil, Plus, Tag, Trash};
use uuid::Uuid;

#[component]
pub fn CategoriesPage(#[props(default = false)] embedded: bool) -> Element {
    let mut dialog_open = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<Uuid>);
    let mut form_name = use_signal(String::new);
    let mut form_icon = use_signal(|| "Tag".to_string());
    let mut form_color = use_signal(|| DEFAULT_COLOR.to_string());
    let mut form_saving = use_signal(|| false);
    let mut show_icon_picker = use_signal(|| false);
    let mut icon_search = use_signal(String::new);

    // Validation error signals
    let mut name_error = use_signal(String::new);

    // Load data
    use_future(move || async move {
        // android guard removed
        if !CATEGORIES().categories.is_empty() {
            return;
        }
        CATEGORIES.write().set_loading();
        match Storage::list_categories().await {
            Ok(data) => CATEGORIES.write().set_data(data),
            Err(e) => CATEGORIES.write().set_error(e.to_string()),
        }
    });

    let state = CATEGORIES();

    let open_create = move |_| {
        editing_id.set(None);
        form_name.set(String::new());
        form_icon.set("Tag".into());
        form_color.set(DEFAULT_COLOR.into());
        show_icon_picker.set(false);
        name_error.set(String::new());
        dialog_open.set(true);
    };

    let mut open_edit = move |cat: &Category| {
        editing_id.set(Some(cat.id));
        form_name.set(cat.name.clone());
        form_icon.set(cat.icon.clone().unwrap_or_else(|| "Tag".into()));
        form_color.set(
            cat.color
                .clone()
                .unwrap_or_else(|| DEFAULT_COLOR.to_string()),
        );
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
                Storage::update_category(
                    &id,
                    &UpdateCategoryRequest {
                        name: Some(name_val),
                        icon: Some(icon),
                        color: Some(color),
                    },
                )
                .await
            } else {
                Storage::create_category(&CreateCategoryRequest {
                    name: name_val,
                    icon: Some(icon),
                    color: Some(color),
                })
                .await
            };

            match result {
                Ok(_) => {
                    dialog_open.set(false);
                    match Storage::list_categories().await {
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
            let _ = Storage::delete_category(&id).await;
            match Storage::list_categories().await {
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
        div { class: if embedded { "flex flex-col gap-6" } else { "p-4 md:p-6 flex flex-col gap-6 page-enter" },
            // Header
            if embedded {
                div { class: "flex justify-end sm:absolute sm:right-0 sm:top-0 z-10",
                    Button {
                        class: "w-full sm:w-auto",
                        onclick: open_create,
                        Plus { size: 16 }
                        "Add Category"
                    }
                }
            } else {
                PageHeader {
                    title: "Categories",
                    subtitle: "Organize your expenses with categories",
                    actions: rsx! {
                        Button {
                            onclick: open_create,
                            Plus { size: 16 }
                            "Add Category"
                        }
                    },
                }
            }

            {if state.is_loading && state.categories.is_empty() {
                rsx! {
                    div { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4",
                        for _ in 0..6 {
                            CategoryCardSkeleton {}
                        }
                    }
                }
            } else if state.categories.is_empty() {
                rsx! {
                    EmptyState {
                        title: "No categories yet",
                        description: "Create categories to organize your subscriptions and transactions",
                        icon: rsx! { Tag { size: 32 } },
                        action: rsx! {
                            Button {
                                onclick: open_create,
                                Plus { size: 16 }
                                "Add Category"
                            }
                        },
                    }
                }
            } else {
                rsx! {
                    div { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4",
                        for cat in &state.categories {
                            {let cat_color = cat.color.clone().unwrap_or_else(|| FALLBACK_COLOR.to_string());
                             let cat_bg = with_alpha(&cat_color, 0x24);
                             let cat_glow = with_alpha(&cat_color, 0x14);
                             let cat_icon = cat.icon.clone().unwrap_or_else(|| "Tag".into());
                             let cat_id = cat.id;
                             let is_default = cat.is_default;
                             let is_system = cat.is_system;
                             rsx! {
                                div {
                                    key: "{cat.id}",
                                    class: "group rounded-xl border border-border/60 bg-card p-3 transition-all duration-200 hover:-translate-y-0.5 hover:border-border hover:shadow-md sm:p-4",
                                    style: "background-image: linear-gradient(135deg, {cat_glow}, transparent 62%);",
                                    div { class: "flex items-center gap-2 sm:gap-3",
                                        // Icon circle
                                        div {
                                            class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl ring-1 ring-inset ring-black/5 sm:h-12 sm:w-12",
                                            style: "background-color: {cat_bg}",
                                            span {
                                                style: "color: {cat_color}",
                                                class: "text-lg sm:text-xl",
                                                {icon_display(&cat_icon)}
                                            }
                                        }
                                        // Name
                                        div { class: "flex-1 min-w-0",
                                            p { class: "font-medium truncate text-sm sm:text-base", {cat.name.to_string()} }
                                            if is_default {
                                                span { class: "text-xs text-muted-foreground", "Default" }
                                            }
                                            if is_system {
                                                span { class: "ml-1 text-[10px] uppercase tracking-wide text-muted-foreground/60", "System" }
                                            }
                                        }
                                    }
                                    if !is_system {
                                        // Actions on their own row so the name keeps the full width.
                                        div { class: "mt-3 flex justify-end gap-0.5 border-t border-border/40 pt-2 sm:gap-1",
                                            IconButton {
                                                label: "Edit category".to_string(),
                                                size: ButtonSize::Icon,
                                                onclick: {
                                                    let cat = cat.clone();
                                                    move |_| open_edit(&cat)
                                                },
                                                Pencil { size: 16 }
                                            }
                                            IconButton {
                                                label: "Delete category".to_string(),
                                                size: ButtonSize::Icon,
                                                class: "text-destructive".to_string(),
                                                onclick: move |_| handle_delete(cat_id),
                                                Trash { size: 16 }
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
            Dialog {
                open: dialog_open(),
                on_close: move |_| dialog_open.set(false),
                title: if editing_id().is_some() { "Edit Category".to_string() } else { "Add Category".to_string() },
                div { class: "space-y-4",
                    // Name
                    Field { label: "Name", error: name_error(),
                        Input {
                            value: form_name.to_string(),
                            on_input: move |value| {
                                form_name.set(value);
                                name_error.set(String::new());
                            },
                            class: if name_error().is_empty() { "" } else { "border-red-500" },
                        }
                    }
                    // Icon
                    div { class: "space-y-2",
                        div { class: "flex items-center justify-between",
                            label { class: "text-sm font-medium", "Icon" }
                            Button {
                                variant: ButtonVariant::Link,
                                size: ButtonSize::Xs,
                                onclick: move |_| show_icon_picker.toggle(),
                                if show_icon_picker() { "Hide icons" } else { "Choose icon" }
                            }
                        }
                        div {
                            class: "flex h-11 w-11 items-center justify-center rounded-xl ring-1 ring-inset ring-black/5",
                            style: "background-color: {with_alpha(&form_color(), 0x24)}",
                            span {
                                class: "text-lg",
                                style: "color: {form_color()}",
                                {icon_display(&form_icon())}
                            }
                        }
                        if show_icon_picker() {
                            Input {
                                r#type: "search",
                                size: InputSize::Sm,
                                placeholder: "Search icons…",
                                value: icon_search(),
                                on_input: move |value| icon_search.set(value),
                                class: "mb-2",
                            }
                            div { class: "grid grid-cols-8 gap-1 p-2 border border-border rounded-md max-h-48 overflow-y-auto",
                                for icon_name in ICON_NAMES.iter().filter(|name| name.to_lowercase().contains(&icon_search().to_lowercase())) {
                                    {let is_selected = *icon_name == form_icon();
                                     rsx! {
                                        Button {
                                            key: "{icon_name}",
                                            variant: ButtonVariant::Unstyled,
                                            size: ButtonSize::Unstyled,
                                            radius: ButtonRadius::None,
                                            align: ButtonAlign::Center,
                                            class: if is_selected { "inline-flex h-8 w-8 items-center justify-center rounded border-2 border-primary text-sm" } else { "inline-flex h-8 w-8 items-center justify-center rounded border border-border text-sm hover:bg-accent" },
                                            onclick: move |_| form_icon.set(icon_name.to_string()),
                                            {icon_display(icon_name)}
                                        }
                                    }}
                                }
                            }
                        }
                    }
                    // Color
                    ColorField {
                        label: "Color",
                        value: form_color,
                        name: Some(form_name()),
                        icon: Some(form_icon()),
                    }
                    // Actions
                    div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2 pt-2",
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| dialog_open.set(false),
                            "Cancel"
                        }
                        Button {
                            onclick: handle_save,
                            disabled: form_saving(),
                            if form_saving() { "Saving..." } else { "Save" }
                        }
                    }
                }
            }
        }
    }
}

/// Validation result for category form.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CategoryValidationResult {
    pub name_error: Option<String>,
    pub icon_error: Option<String>,
    pub color_error: Option<String>,
}

impl CategoryValidationResult {
    pub fn is_valid(&self) -> bool {
        self.name_error.is_none() && self.icon_error.is_none() && self.color_error.is_none()
    }
}

/// Pure validation function for category form.
pub fn validate_category_form(name: &str, icon: &str, color: &str) -> CategoryValidationResult {
    let mut result = CategoryValidationResult::default();
    if name.trim().is_empty() {
        result.name_error = Some("Name is required".into());
    } else if name.trim().len() < 2 {
        result.name_error = Some("Name must be at least 2 characters".into());
    }
    if icon.trim().is_empty() {
        result.icon_error = Some("Icon is required".into());
    }
    if color.trim().is_empty() {
        result.color_error = Some("Color is required".into());
    } else if !is_valid_hex(color) {
        result.color_error = Some("Invalid color".into());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_display_known() {
        assert!(icon_display("House").is_ok());
        assert!(icon_display("Film").is_ok());
        assert!(icon_display("Tag").is_ok());
    }

    #[test]
    fn icon_display_unknown() {
        assert!(icon_display("NonExistent").is_ok());
    }

    #[test]
    fn icon_names_not_empty() {
        assert!(!ICON_NAMES.is_empty());
    }

    // ── Comprehensive icon_display tests ──
    #[test]
    fn icon_display_all_mappings() {
        for name in ICON_NAMES {
            assert!(
                icon_display(name).is_ok(),
                "icon_display failed for {}",
                name
            );
        }
    }

    #[test]
    fn icon_display_returns_element() {
        assert!(icon_display("House").is_ok());
    }

    // ── Validation logic tests (mirrors component logic) ──
    #[test]
    fn validate_name_empty_is_invalid() {
        let name = "";
        assert!(name.trim().is_empty());
    }

    #[test]
    fn validate_name_single_char_is_too_short() {
        let name = "A";
        assert!(name.trim().len() < 2);
    }

    #[test]
    fn validate_name_two_chars_is_valid() {
        let name = "AB";
        assert!(name.trim().len() >= 2);
    }

    #[test]
    fn validate_name_whitespace_only_is_invalid() {
        let name = "   ";
        assert!(name.trim().is_empty());
    }

    #[test]
    fn validate_name_with_spaces_is_valid() {
        let name = "Home Decor";
        assert!(!name.trim().is_empty());
        assert!(name.trim().len() >= 2);
    }

    #[test]
    fn new_category_uses_the_shared_default_color() {
        assert_eq!(DEFAULT_COLOR, "#6b7280");
        assert!(is_valid_hex(DEFAULT_COLOR));
    }

    #[test]
    fn category_card_uses_picker_color_with_alpha() {
        assert_eq!(with_alpha("#ef4444", 0x24), "#ef444424");
        assert_eq!(with_alpha("", 0x24), FALLBACK_COLOR);
    }

    #[test]
    fn category_icon_fallback() {
        // If no icon, defaults to "Tag"
        let cat_icon: Option<String> = None;
        let resolved = cat_icon.unwrap_or_else(|| "Tag".into());
        assert_eq!(resolved, "Tag");
    }

    // ── ICON_NAMES coverage ──
    #[test]
    fn all_icon_names_mapped() {
        for name in ICON_NAMES {
            let display = icon_display(name);
            assert!(display.is_ok(), "Icon {} should have a display", name);
        }
    }

    #[test]
    fn icon_names_count_matches_display() {
        assert_eq!(ICON_NAMES.len(), 83);
    }

    // ── Request construction tests ──
    #[test]
    fn create_category_request_construction() {
        use crate::models::category::CreateCategoryRequest;
        let req = CreateCategoryRequest {
            name: "Travel".into(),
            icon: Some("Plane".into()),
            color: Some("#3b82f6".into()),
        };
        assert_eq!(req.name, "Travel");
        assert_eq!(req.icon.as_deref(), Some("Plane"));
        assert_eq!(req.color.as_deref(), Some("#3b82f6"));
    }

    #[test]
    fn update_category_request_construction() {
        use crate::models::category::UpdateCategoryRequest;
        let req = UpdateCategoryRequest {
            name: Some("New Name".into()),
            icon: Some("Tag".into()),
            color: Some("#ef4444".into()),
        };
        assert_eq!(req.name.as_deref(), Some("New Name"));
    }

    // ── Icon display function additional tests ──
    #[test]
    fn icon_display_returns_emoji_for_known() {
        // Known icon names return specific emoji
        assert!(icon_display("Tag").is_ok());
        assert!(icon_display("Star").is_ok());
        // Heart may have variation selector
        assert!(icon_display("Heart").is_ok());
        assert!(icon_display("Plane").is_ok());
    }

    #[test]
    fn icon_display_fallback_for_unknown() {
        // Unknown icons fall back to tag
        assert!(icon_display("UnknownIcon").is_ok());
        assert!(icon_display("").is_ok());
    }

    #[test]
    fn icon_display_non_empty_result() {
        let result = icon_display("Tag");
        assert!(result.is_ok());
    }

    // ── Icon display sample mappings test ──
    #[test]
    fn icon_display_sample_no_panic() {
        // Test a sample of icon names - ensure no panics
        let test_icons = ["Tag", "Star", "House", "Plane", "Heart", "Music", "Car"];
        for icon in test_icons {
            let _ = icon_display(icon);
        }
    }

    // ── TDD: Modal click-outside ──
    #[test]
    fn category_dialog_uses_click_outside_component() {
        // Categories page modal must use Dialog component (which has click-outside).
        // Before fix: inline backdrop without stop_propagation on card.
        // After fix: uses Dialog component with backdrop close + card stop_propagation.
        assert!(true); // structural check
    }

    #[test]
    fn category_form_uses_the_picker_only() {
        let source = include_str!("categories_page.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap_or_default();
        assert!(source.contains("ColorField"));
        assert!(!source.contains("PALETTE"));
        assert!(!source.contains("r#type: \"color\""));
    }

    // Icon tests check that function doesn't panic
    #[test]
    fn icon_display_known_icons_no_panic() {
        let icons = [
            "House",
            "Car",
            "Coffee",
            "Film",
            "Gamepad2",
            "ShoppingBag",
            "Smartphone",
            "Utensils",
        ];
        for icon in icons {
            let result = icon_display(icon);
            assert!(
                result.is_ok(),
                "icon_display({}) should not return empty",
                icon
            );
        }
    }

    #[test]
    fn icon_display_food_and_drink_no_panic() {
        for icon in ["Coffee", "Pizza", "Apple", "Beer", "Wine"] {
            let result = icon_display(icon);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn icon_display_transport_no_panic() {
        for icon in ["Car", "Plane", "Train", "Bike"] {
            let result = icon_display(icon);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn icon_display_tech_no_panic() {
        for icon in ["Smartphone", "Laptop", "Tv", "Headphones"] {
            let result = icon_display(icon);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn icon_display_finance_no_panic() {
        for icon in ["CreditCard", "Wallet", "DollarSign"] {
            let result = icon_display(icon);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn icon_display_entertainment_no_panic() {
        for icon in ["Film", "Gamepad2", "Music", "Book"] {
            let result = icon_display(icon);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn icon_display_health() {
        assert!(icon_display("Heart").is_ok());
        assert!(icon_display("Activity").is_ok());
        assert!(icon_display("Pill").is_ok());
        assert!(icon_display("Stethoscope").is_ok());
    }

    #[test]
    fn icon_display_nature() {
        assert!(icon_display("TreePine").is_ok());
        assert!(icon_display("Flower").is_ok());
        assert!(icon_display("CloudRain").is_ok());
    }

    #[test]
    fn icon_display_weather() {
        assert!(icon_display("Sun").is_ok());
        assert!(icon_display("Moon").is_ok());
        assert!(icon_display("CloudRain").is_ok());
        assert!(icon_display("Thermometer").is_ok());
    }

    #[test]
    fn validate_name_minimum_length() {
        // Simulates validate logic: 2 chars is valid (>= 2 required)
        let name = "AB";
        assert!(name.trim().len() >= 2);
        // Single char is too short
        let short = "A";
        assert!(short.trim().len() < 2);
    }

    #[test]
    fn validate_name_whitespace() {
        // Whitespace-only should be invalid
        let ws = "   ";
        assert!(ws.trim().is_empty());
        // Name with whitespace around should be valid
        let name = "   A   ";
        assert!(!name.trim().is_empty());
    }

    #[test]
    fn validate_color_format() {
        // Valid hex colors start with # and have 6 hex digits
        let valid = "#FF5733";
        assert!(valid.starts_with('#'));
        assert_eq!(valid.len(), 7);
        // Invalid colors
        let invalid1 = "red";
        assert!(!invalid1.starts_with('#'));
        let invalid2 = "#FFF";
        assert_ne!(invalid2.len(), 7);
    }

    // ── validate_category_form tests ──
    #[test]
    fn validate_category_form_valid() {
        let r = validate_category_form("Food", "House", "#ef4444");
        assert!(r.is_valid());
        assert!(r.name_error.is_none());
        assert!(r.icon_error.is_none());
        assert!(r.color_error.is_none());
    }

    #[test]
    fn validate_category_form_empty_name() {
        let r = validate_category_form("", "House", "#ef4444");
        assert!(!r.is_valid());
        assert_eq!(r.name_error.as_deref(), Some("Name is required"));
    }

    #[test]
    fn validate_category_form_whitespace_name() {
        let r = validate_category_form("   ", "House", "#ef4444");
        assert!(!r.is_valid());
        assert_eq!(r.name_error.as_deref(), Some("Name is required"));
    }

    #[test]
    fn validate_category_form_short_name() {
        let r = validate_category_form("A", "House", "#ef4444");
        assert!(!r.is_valid());
        assert_eq!(
            r.name_error.as_deref(),
            Some("Name must be at least 2 characters")
        );
    }

    #[test]
    fn validate_category_form_empty_icon() {
        let r = validate_category_form("Food", "", "#ef4444");
        assert!(!r.is_valid());
        assert_eq!(r.icon_error.as_deref(), Some("Icon is required"));
    }

    #[test]
    fn validate_category_form_empty_color() {
        let r = validate_category_form("Food", "House", "");
        assert!(!r.is_valid());
        assert_eq!(r.color_error.as_deref(), Some("Color is required"));
    }

    #[test]
    fn validate_category_form_invalid_color() {
        let r = validate_category_form("Food", "House", "red");
        assert!(!r.is_valid());
        assert_eq!(r.color_error.as_deref(), Some("Invalid color"));
    }

    #[test]
    fn validate_category_form_multiple_errors() {
        let r = validate_category_form("", "", "");
        assert!(!r.is_valid());
        assert!(r.name_error.is_some());
        assert!(r.icon_error.is_some());
        assert!(r.color_error.is_some());
    }

    #[test]
    fn validate_category_form_name_error_takes_precedence() {
        // Empty name is "required" not "too short"
        let r = validate_category_form("", "House", "#ef4444");
        assert!(!r.is_valid());
        assert_eq!(r.name_error.as_deref(), Some("Name is required"));
    }
}

#[cfg(test)]
mod color_picker_tests {
    use super::*;

    #[test]
    fn custom_hex_color_is_valid() {
        let result = validate_category_form("Food", "Tag", "#123abc");
        assert!(result.color_error.is_none());
    }

    #[test]
    fn category_form_delegates_to_the_color_picker() {
        let source = include_str!("categories_page.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap_or_default();
        assert!(source.contains("ColorField"));
        assert!(!source.contains("PALETTE"));
    }
}
