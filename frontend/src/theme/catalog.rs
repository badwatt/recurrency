//! The theme catalogue is the single source of truth for theme identity,
//! native colours and the picker UI.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeDefinition {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub group: &'static str,
    pub is_dark: bool,
    pub background: &'static str,
    pub foreground: &'static str,
    pub card: &'static str,
    pub primary: &'static str,
}

pub const THEME_GROUPS: [&str; 3] = ["Catppuccin", "Popular", "Classic"];

pub const THEMES: &[ThemeDefinition] = &[
    ThemeDefinition {
        id: "mocha",
        label: "Catppuccin Mocha",
        description: "Soft lavender night.",
        group: "Catppuccin",
        is_dark: true,
        background: "#1E1E2E",
        foreground: "#CDD6F4",
        card: "#181825",
        primary: "#B4BEFE",
    },
    ThemeDefinition {
        id: "mocha-bloom",
        label: "Mocha Bloom",
        description: "Catppuccin, turned up.",
        group: "Catppuccin",
        is_dark: true,
        background: "#221A2E",
        foreground: "#F4ECFF",
        card: "#2B213A",
        primary: "#F5A9E6",
    },
    ThemeDefinition {
        id: "latte",
        label: "Catppuccin Latte",
        description: "Pastel daylight.",
        group: "Catppuccin",
        is_dark: false,
        background: "#EFF1F5",
        foreground: "#4C4F69",
        card: "#FFFFFF",
        primary: "#8839EF",
    },
    ThemeDefinition {
        id: "dracula",
        label: "Dracula",
        description: "Purple night, pink accents.",
        group: "Popular",
        is_dark: true,
        background: "#282A36",
        foreground: "#F8F8F2",
        card: "#21222C",
        primary: "#BD93F9",
    },
    ThemeDefinition {
        id: "nord",
        label: "Nord",
        description: "Arctic blue.",
        group: "Popular",
        is_dark: true,
        background: "#2E3440",
        foreground: "#ECEFF4",
        card: "#3B4252",
        primary: "#88C0D0",
    },
    ThemeDefinition {
        id: "tokyo-night",
        label: "Tokyo Night",
        description: "Deep blue city.",
        group: "Popular",
        is_dark: true,
        background: "#1A1B26",
        foreground: "#C0CAF5",
        card: "#16161E",
        primary: "#7AA2F7",
    },
    ThemeDefinition {
        id: "gruvbox",
        label: "Gruvbox",
        description: "Warm retro.",
        group: "Popular",
        is_dark: true,
        background: "#282828",
        foreground: "#EBDBB2",
        card: "#32302F",
        primary: "#FABD2F",
    },
    ThemeDefinition {
        id: "rose-pine",
        label: "Rosé Pine",
        description: "Muted rose.",
        group: "Popular",
        is_dark: true,
        background: "#191724",
        foreground: "#E0DEF4",
        card: "#1F1D2E",
        primary: "#C4A7E7",
    },
    ThemeDefinition {
        id: "one-dark",
        label: "One Dark",
        description: "Editor classic.",
        group: "Popular",
        is_dark: true,
        background: "#282C34",
        foreground: "#ABB2BF",
        card: "#21252B",
        primary: "#61AFEF",
    },
    ThemeDefinition {
        id: "solarized-light",
        label: "Solarized Light",
        description: "Low-contrast daylight.",
        group: "Popular",
        is_dark: false,
        background: "#FDF6E3",
        foreground: "#586E75",
        card: "#FFFBF0",
        primary: "#268BD2",
    },
    ThemeDefinition {
        id: "dark",
        label: "Midnight Amber",
        description: "Near-black with amber.",
        group: "Classic",
        is_dark: true,
        background: "#0D0D11",
        foreground: "#EDEDED",
        card: "#131117",
        primary: "#E9900C",
    },
    ThemeDefinition {
        id: "light",
        label: "Paper",
        description: "Warm paper, ink primary.",
        group: "Classic",
        is_dark: false,
        background: "#FAF9F2",
        foreground: "#212121",
        card: "#FFFFFF",
        primary: "#212121",
    },
];

pub fn theme_by_id(id: &str) -> Option<&'static ThemeDefinition> {
    THEMES.iter().find(|theme| theme.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn theme_ids_are_unique() {
        let ids: HashSet<_> = THEMES.iter().map(|theme| theme.id).collect();
        assert_eq!(ids.len(), THEMES.len());
    }

    #[test]
    fn every_group_has_at_least_one_theme() {
        for group in THEME_GROUPS {
            assert!(
                THEMES.iter().any(|theme| theme.group == group),
                "group {group} must not be empty"
            );
        }
    }

    #[test]
    fn mocha_remains_the_default_theme() {
        assert_eq!(THEMES[0].id, "mocha");
        assert!(THEMES[0].is_dark);
    }

    #[test]
    fn catalog_covers_light_and_dark_themes() {
        assert!(THEMES.iter().any(|theme| theme.is_dark));
        assert!(THEMES.iter().any(|theme| !theme.is_dark));
    }

    #[test]
    fn catalog_has_the_new_colorful_and_popular_themes() {
        for id in [
            "mocha-bloom",
            "latte",
            "dracula",
            "nord",
            "tokyo-night",
            "gruvbox",
            "rose-pine",
            "one-dark",
            "solarized-light",
        ] {
            assert!(theme_by_id(id).is_some(), "missing theme {id}");
        }
    }

    #[test]
    fn css_defines_every_catalogue_theme() {
        let css = include_str!("../../assets/styles/tailwind.src.css");
        for theme in THEMES {
            assert!(
                css.contains(&format!("html.{} {{", theme.id)),
                "tailwind.src.css must define html.{}",
                theme.id
            );
        }
    }

    #[test]
    fn unknown_theme_is_not_resolved() {
        assert!(theme_by_id("system").is_none());
        assert!(theme_by_id("").is_none());
    }
}
