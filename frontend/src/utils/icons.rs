/// Icon name → display mapping.
/// Mirrors the getIcon() function from the React frontend.
/// In Dioxus, we'll use lucide-dioxus components, but this provides
/// the name lookup/normalization logic.
///
/// Available icon names (matching lucide-react icon names from original).
pub const ICON_NAMES: &[&str] = &[
    "Home",
    "ShoppingBag",
    "Coffee",
    "Car",
    "Heart",
    "Film",
    "Music",
    "Gamepad2",
    "Briefcase",
    "GraduationCap",
    "Plane",
    "Zap",
    "Wifi",
    "Smartphone",
    "Laptop",
    "Monitor",
    "Cloud",
    "Database",
    "Server",
    "Lock",
    "Shield",
    "CreditCard",
    "Wallet",
    "Gift",
    "Star",
    "Sun",
    "Moon",
    "CloudRain",
    "Thermometer",
    "Activity",
    "Pill",
    "Stethoscope",
    "Baby",
    "Flower",
    "TreePine",
    "Utensils",
    "Beer",
    "Wine",
    "Tag",
];

/// Default icon when name is unknown or null.
pub const DEFAULT_ICON: &str = "Tag";

/// Map icon name to a display character (emoji).
pub fn icon_display(name: &str) -> String {
    match name {
        "Home" => "🏠", "ShoppingBag" => "🛍", "Coffee" => "☕", "Car" => "🚗",
        "Heart" => "❤", "Film" => "🎬", "Music" => "🎵", "Gamepad2" => "🎮",
        "Briefcase" => "💼", "GraduationCap" => "🎓", "Plane" => "✈",
        "Zap" => "⚡", "Wifi" => "📶", "Smartphone" => "📱", "Laptop" => "💻",
        "Monitor" => "🖥", "Cloud" => "☁", "Database" => "🗄", "Server" => "🖧",
        "Lock" => "🔒", "Shield" => "🛡", "CreditCard" => "💳", "Wallet" => "👛",
        "Gift" => "🎁", "Star" => "⭐", "Sun" => "☀", "Moon" => "🌙",
        "CloudRain" => "🌧", "Thermometer" => "🌡", "Activity" => "📈",
        "Pill" => "💊", "Stethoscope" => "🩺", "Baby" => "👶",
        "Flower" => "🌸", "TreePine" => "🌲", "Utensils" => "餐具",
        "Beer" => "🍺", "Wine" => "🍷", "Tag" => "🏷", _ => "🏷",
    }.to_string()
}

/// Convert a kebab-case icon name from the backend to PascalCase.
/// e.g., "tree-pine" → "TreePine"
pub fn normalize_icon_name(name: &str) -> String {
    if name.is_empty() {
        return DEFAULT_ICON.to_string();
    }
    name.split('-')
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Check if an icon name is available.
pub fn is_valid_icon(name: &str) -> bool {
    let normalized = normalize_icon_name(name);
    ICON_NAMES.contains(&normalized.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_kebab_case() {
        assert_eq!(normalize_icon_name("tree-pine"), "TreePine");
        assert_eq!(normalize_icon_name("shopping-bag"), "ShoppingBag");
        assert_eq!(normalize_icon_name("gamepad-2"), "Gamepad2");
    }

    #[test]
    fn normalize_single_word() {
        assert_eq!(normalize_icon_name("home"), "Home");
        assert_eq!(normalize_icon_name("film"), "Film");
    }

    #[test]
    fn normalize_empty() {
        assert_eq!(normalize_icon_name(""), "Tag");
    }

    #[test]
    fn valid_icons() {
        assert!(is_valid_icon("home"));
        assert!(is_valid_icon("tree-pine"));
        assert!(is_valid_icon("Gamepad2"));
    }

    #[test]
    fn invalid_icon_falls_to_default() {
        assert!(!is_valid_icon("nonexistent"));
        assert_eq!(normalize_icon_name("nonexistent"), "Nonexistent");
    }

    #[test]
    fn icon_count() {
        assert_eq!(ICON_NAMES.len(), 39);
    }

    // ── Additional tests ──
    #[test]
    fn default_icon_is_tag() {
        assert_eq!(DEFAULT_ICON, "Tag");
    }

    #[test]
    fn normalize_already_pascal() {
        assert_eq!(normalize_icon_name("Home"), "Home");
        assert_eq!(normalize_icon_name("Film"), "Film");
    }

    #[test]
    fn is_valid_icon_direct() {
        // Direct PascalCase names should be found
        assert!(is_valid_icon("Home"));
        assert!(is_valid_icon("Tag"));
    }

    #[test]
    fn is_valid_icon_from_kebab() {
        // Kebab-case gets normalized and checked
        assert!(is_valid_icon("shopping-bag"));
    }

    #[test]
    fn is_valid_icon_empty_false() {
        // Empty string normalizes to Tag, which IS valid
        assert!(is_valid_icon("")); // empty = Tag = valid
    }

    #[test]
    fn icon_names_all_unique() {
        let mut seen = std::collections::HashSet::new();
        for name in ICON_NAMES {
            assert!(seen.insert(*name), "Duplicate icon name: {}", name);
        }
    }

    #[test]
    fn normalize_multiple_hyphens() {
        assert_eq!(normalize_icon_name("a-b-c"), "ABC");
    }

    #[test]
    fn normalize_single_char_parts() {
        assert_eq!(normalize_icon_name("a-b"), "AB");
    }

    #[test]
    fn normalize_preserves_numbers() {
        assert_eq!(normalize_icon_name("gamepad-2"), "Gamepad2");
    }

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
}
