//! Typography and spacing shared by the native screens.
//!
//! Colors come from `theme.json`, which mirrors the current dark palette.
//! The remaining palette catalogue will be moved here as the screens land.

pub const SANS: &str = "Cabinet Grotesk";
pub const MONO: &str = "JetBrains Mono";

pub const PAGE_GAP: f32 = 24.0;
pub const CARD_RADIUS: f32 = 12.0;

/// Parse an account/category `#rrggbb` color, falling back to the muted gray.
pub fn parse_hex(value: &str) -> u32 {
    u32::from_str_radix(value.trim_start_matches('#'), 16).unwrap_or(0x6b7280)
}

/// Success badges (active states) use the `success` token when the theme
/// defines it; the primary color is the fallback.
pub fn success() -> nsh_core::element::AdaptiveColor {
    nsh_base::theme()
        .color("success")
        .unwrap_or_else(|| nsh_base::theme().primary)
}

/// The app-wide palette catalogue. The ids are the keys the Dioxus frontend
/// keeps in local storage and in the user's settings, so the native app can
/// read and write the same selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeDefinition {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub is_dark: bool,
    pub background: u32,
    pub foreground: u32,
    pub card: u32,
    pub primary: u32,
}

pub const THEMES: &[ThemeDefinition] = &[
    ThemeDefinition {
        id: "mocha",
        label: "Catppuccin Mocha",
        description: "Soft lavender night.",
        is_dark: true,
        background: 0x1E1E2E,
        foreground: 0xCDD6F4,
        card: 0x181825,
        primary: 0xB4BEFE,
    },
    ThemeDefinition {
        id: "frappe",
        label: "Catppuccin Frappé",
        description: "Muted winter blue.",
        is_dark: true,
        background: 0x303446,
        foreground: 0xC6D0F5,
        card: 0x292C3C,
        primary: 0xCA9EE6,
    },
    ThemeDefinition {
        id: "macchiato",
        label: "Catppuccin Macchiato",
        description: "Deep violet dusk.",
        is_dark: true,
        background: 0x24273A,
        foreground: 0xCAD3F5,
        card: 0x1E2030,
        primary: 0xC6A0F6,
    },
    ThemeDefinition {
        id: "mocha-bloom",
        label: "Mocha Bloom",
        description: "Catppuccin, turned up.",
        is_dark: true,
        background: 0x221A2E,
        foreground: 0xF4ECFF,
        card: 0x2B213A,
        primary: 0xF5A9E6,
    },
    ThemeDefinition {
        id: "dracula",
        label: "Dracula",
        description: "Purple night, pink accents.",
        is_dark: true,
        background: 0x282A36,
        foreground: 0xF8F8F2,
        card: 0x21222C,
        primary: 0xBD93F9,
    },
    ThemeDefinition {
        id: "nord",
        label: "Nord",
        description: "Arctic blue.",
        is_dark: true,
        background: 0x2E3440,
        foreground: 0xECEFF4,
        card: 0x3B4252,
        primary: 0x88C0D0,
    },
    ThemeDefinition {
        id: "tokyo-night",
        label: "Tokyo Night",
        description: "Deep blue city.",
        is_dark: true,
        background: 0x1A1B26,
        foreground: 0xC0CAF5,
        card: 0x16161E,
        primary: 0x7AA2F7,
    },
    ThemeDefinition {
        id: "gruvbox",
        label: "Gruvbox",
        description: "Warm retro.",
        is_dark: true,
        background: 0x282828,
        foreground: 0xEBDBB2,
        card: 0x32302F,
        primary: 0xFABD2F,
    },
    ThemeDefinition {
        id: "rose-pine",
        label: "Rosé Pine",
        description: "Muted rose.",
        is_dark: true,
        background: 0x191724,
        foreground: 0xE0DEF4,
        card: 0x1F1D2E,
        primary: 0xC4A7E7,
    },
    ThemeDefinition {
        id: "one-dark",
        label: "One Dark",
        description: "Editor classic.",
        is_dark: true,
        background: 0x282C34,
        foreground: 0xABB2BF,
        card: 0x21252B,
        primary: 0x61AFEF,
    },
    ThemeDefinition {
        id: "kanagawa",
        label: "Kanagawa",
        description: "Ink and wave blue.",
        is_dark: true,
        background: 0x1F1F28,
        foreground: 0xDCD7BA,
        card: 0x16161D,
        primary: 0x7E9CD8,
    },
    ThemeDefinition {
        id: "everforest",
        label: "Everforest",
        description: "Mossy green night.",
        is_dark: true,
        background: 0x2D353B,
        foreground: 0xD3C6AA,
        card: 0x272E33,
        primary: 0xA7C080,
    },
    ThemeDefinition {
        id: "github-dark",
        label: "GitHub Dark",
        description: "GitHub’s dark canvas.",
        is_dark: true,
        background: 0x0D1117,
        foreground: 0xE6EDF3,
        card: 0x161B22,
        primary: 0x58A6FF,
    },
    ThemeDefinition {
        id: "dark",
        label: "Midnight Amber",
        description: "Near-black with amber.",
        is_dark: true,
        background: 0x0D0D11,
        foreground: 0xEDEDED,
        card: 0x131117,
        primary: 0xE9900C,
    },
    ThemeDefinition {
        id: "latte",
        label: "Catppuccin Latte",
        description: "Pastel daylight.",
        is_dark: false,
        background: 0xEFF1F5,
        foreground: 0x4C4F69,
        card: 0xFFFFFF,
        primary: 0x8839EF,
    },
    ThemeDefinition {
        id: "solarized-light",
        label: "Solarized Light",
        description: "Low-contrast daylight.",
        is_dark: false,
        background: 0xFDF6E3,
        foreground: 0x586E75,
        card: 0xFFFBF0,
        primary: 0x268BD2,
    },
    ThemeDefinition {
        id: "github-light",
        label: "GitHub Light",
        description: "GitHub's paper canvas.",
        is_dark: false,
        background: 0xFFFFFF,
        foreground: 0x1F2328,
        card: 0xF6F8FA,
        primary: 0x0969DA,
    },
    ThemeDefinition {
        id: "light",
        label: "Paper",
        description: "Warm paper, ink primary.",
        is_dark: false,
        background: 0xFAF9F2,
        foreground: 0x212121,
        card: 0xFFFFFF,
        primary: 0x212121,
    },
];

pub const DEFAULT_THEME: &str = "mocha";
const THEME_KEY: &str = "recurrency.theme";

pub fn theme_by_id(id: &str) -> Option<&'static ThemeDefinition> {
    THEMES.iter().find(|theme| theme.id == id)
}

pub fn saved() -> String {
    nsh_core::storage::Storage::get_item(THEME_KEY)
        .filter(|value| theme_by_id(value).is_some())
        .unwrap_or_else(|| DEFAULT_THEME.to_string())
}

pub fn save(id: &str) {
    let _ = nsh_core::storage::Storage::set_item(THEME_KEY, id);
}

fn contrast(primary: u32) -> u32 {
    let red = (primary >> 16) & 0xFF;
    let green = (primary >> 8) & 0xFF;
    let blue = primary & 0xFF;
    let luminance = red * 299 + green * 587 + blue * 114;
    if luminance > 150_000 {
        0x000000
    } else {
        0xFFFFFF
    }
}

/// Rebuild the design tokens for `id` and hand them to the shell. The light
/// and dark slots carry the same values because the chosen catalogue entry
/// is a complete palette, not a mode toggle.
pub fn apply(id: &str) {
    let Some(definition) = theme_by_id(id) else {
        return;
    };
    let mut tokens = (*nsh_core::element::tokens()).clone();
    let bg = nsh_core::element::AdaptiveColor::rgb(definition.background, definition.background);
    let fg = nsh_core::element::AdaptiveColor::rgb(definition.foreground, definition.foreground);
    let card = nsh_core::element::AdaptiveColor::rgb(definition.card, definition.card);
    let primary = nsh_core::element::AdaptiveColor::rgb(definition.primary, definition.primary);
    let on_primary = nsh_core::element::AdaptiveColor::rgb(
        contrast(definition.primary),
        contrast(definition.primary),
    );
    tokens.set_color("background", bg);
    tokens.set_color("foreground", fg);
    tokens.set_color("card", card);
    tokens.set_color("card-foreground", fg);
    tokens.set_color("popover", card);
    tokens.set_color("popover-foreground", fg);
    tokens.set_color("primary", primary);
    tokens.set_color("primary-foreground", on_primary);
    tokens.set_color("secondary-foreground", fg);
    tokens.set_color("muted-foreground", fg.opacity(0.66));
    tokens.set_color("accent-foreground", fg);
    tokens.set_color("ring", primary);
    nsh_core::element::set_tokens(tokens);
}

#[cfg(test)]
mod catalog_tests {
    use super::*;

    #[test]
    fn catalogue_has_eighteen_unique_themes() {
        let mut ids: Vec<_> = THEMES.iter().map(|theme| theme.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 18);
    }

    #[test]
    fn legacy_theme_ids_stay_valid() {
        assert!(theme_by_id("light").is_some());
        assert!(theme_by_id("dark").is_some());
        assert!(theme_by_id("mocha").is_some());
    }

    #[test]
    fn contrast_picks_black_on_light_primaries() {
        assert_eq!(contrast(0xFABD2F), 0x000000);
        assert_eq!(contrast(0x1E1E2E), 0xFFFFFF);
    }
}
