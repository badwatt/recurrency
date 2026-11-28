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

/// Text drawn over a destructive fill: dark, as the rest of the app does.
pub fn on_destructive() -> nsh_core::element::AdaptiveColor {
    nsh_core::element::AdaptiveColor::from(nsh_core::color::rgba(0x1c1c1eff))
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
    pub card_foreground: u32,
    pub popover: u32,
    pub popover_foreground: u32,
    pub primary: u32,
    pub primary_foreground: u32,
    pub secondary: u32,
    pub secondary_foreground: u32,
    pub muted: u32,
    pub muted_foreground: u32,
    pub accent: u32,
    pub accent_foreground: u32,
    pub destructive: u32,
    pub destructive_foreground: u32,
    pub border: u32,
    pub input: u32,
    pub ring: u32,
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
        card_foreground: 0xCDD6F4,
        popover: 0x181825,
        popover_foreground: 0xCDD6F4,
        primary: 0xB4BEFE,
        primary_foreground: 0x12121C,
        secondary: 0x454759,
        secondary_foreground: 0xCDD6F4,
        muted: 0x454759,
        muted_foreground: 0x949AB3,
        accent: 0x313244,
        accent_foreground: 0xCDD6F4,
        destructive: 0xF38CA9,
        destructive_foreground: 0xCDD6F4,
        border: 0x454759,
        input: 0x454759,
        ring: 0xB4BEFE,
    },
    ThemeDefinition {
        id: "frappe",
        label: "Catppuccin Frappé",
        description: "Muted winter blue.",
        is_dark: true,
        background: 0x303446,
        foreground: 0xC7D1F5,
        card: 0x292C3D,
        card_foreground: 0xC7D1F5,
        popover: 0x292C3D,
        popover_foreground: 0xC7D1F5,
        primary: 0xCA9EE6,
        primary_foreground: 0x292C3D,
        secondary: 0x404459,
        secondary_foreground: 0xC7D1F5,
        muted: 0x404459,
        muted_foreground: 0xB6BFE2,
        accent: 0x50566D,
        accent_foreground: 0xC7D1F5,
        destructive: 0xE78384,
        destructive_foreground: 0x303446,
        border: 0x404459,
        input: 0x404459,
        ring: 0xCA9EE6,
    },
    ThemeDefinition {
        id: "macchiato",
        label: "Catppuccin Macchiato",
        description: "Deep violet dusk.",
        is_dark: true,
        background: 0x232638,
        foreground: 0xCCD5F5,
        card: 0x1D202F,
        card_foreground: 0xCCD5F5,
        popover: 0x1D202F,
        popover_foreground: 0xCCD5F5,
        primary: 0xC8A2F6,
        primary_foreground: 0x1D202F,
        secondary: 0x363A4F,
        secondary_foreground: 0xCCD5F5,
        muted: 0x363A4F,
        muted_foreground: 0xB8C0E0,
        accent: 0x494D65,
        accent_foreground: 0xCCD5F5,
        destructive: 0xED8796,
        destructive_foreground: 0x232638,
        border: 0x363A4F,
        input: 0x363A4F,
        ring: 0xC8A2F6,
    },
    ThemeDefinition {
        id: "mocha-bloom",
        label: "Mocha Bloom",
        description: "Catppuccin, turned up.",
        is_dark: true,
        background: 0x221A2E,
        foreground: 0xF3EBFF,
        card: 0x2B223A,
        card_foreground: 0xF3EBFF,
        popover: 0x2B223A,
        popover_foreground: 0xF3EBFF,
        primary: 0xF5A8E6,
        primary_foreground: 0x2D1128,
        secondary: 0x3D2F50,
        secondary_foreground: 0xF3EBFF,
        muted: 0x352B46,
        muted_foreground: 0xCAB9DF,
        accent: 0x402E56,
        accent_foreground: 0xF3EBFF,
        destructive: 0xFF8FA3,
        destructive_foreground: 0x2D1128,
        border: 0x4A3960,
        input: 0x4A3960,
        ring: 0xF5A8E6,
    },
    ThemeDefinition {
        id: "dracula",
        label: "Dracula",
        description: "Purple night, pink accents.",
        is_dark: true,
        background: 0x272935,
        foreground: 0xF8F8F2,
        card: 0x21222C,
        card_foreground: 0xF8F8F2,
        popover: 0x21222C,
        popover_foreground: 0xF8F8F2,
        primary: 0xBF95F9,
        primary_foreground: 0x272935,
        secondary: 0x44475A,
        secondary_foreground: 0xF8F8F2,
        muted: 0x44475A,
        muted_foreground: 0xB7B8C8,
        accent: 0x44475A,
        accent_foreground: 0xF8F8F2,
        destructive: 0xFF5757,
        destructive_foreground: 0x272935,
        border: 0x44475A,
        input: 0x44475A,
        ring: 0xBF95F9,
    },
    ThemeDefinition {
        id: "nord",
        label: "Nord",
        description: "Arctic blue.",
        is_dark: true,
        background: 0x2F3541,
        foreground: 0xECEFF4,
        card: 0x3C4353,
        card_foreground: 0xECEFF4,
        popover: 0x3C4353,
        popover_foreground: 0xECEFF4,
        primary: 0x87BFCF,
        primary_foreground: 0x2F3541,
        secondary: 0x444D5F,
        secondary_foreground: 0xECEFF4,
        muted: 0x444D5F,
        muted_foreground: 0xD8DEE9,
        accent: 0x444D5F,
        accent_foreground: 0xECEFF4,
        destructive: 0xBE6069,
        destructive_foreground: 0xECEFF4,
        border: 0x4D576A,
        input: 0x4D576A,
        ring: 0x87BFCF,
    },
    ThemeDefinition {
        id: "tokyo-night",
        label: "Tokyo Night",
        description: "Deep blue city.",
        is_dark: true,
        background: 0x1B1C27,
        foreground: 0xC1CBF5,
        card: 0x16161D,
        card_foreground: 0xC1CBF5,
        popover: 0x16161D,
        popover_foreground: 0xC1CBF5,
        primary: 0x78A0F7,
        primary_foreground: 0x1B1C27,
        secondary: 0x292E42,
        secondary_foreground: 0xC1CBF5,
        muted: 0x25293C,
        muted_foreground: 0xA9B1D6,
        accent: 0x292E42,
        accent_foreground: 0xC1CBF5,
        destructive: 0xF7788F,
        destructive_foreground: 0x1B1C27,
        border: 0x292E42,
        input: 0x292E42,
        ring: 0x78A0F7,
    },
    ThemeDefinition {
        id: "gruvbox",
        label: "Gruvbox",
        description: "Warm retro.",
        is_dark: true,
        background: 0x292929,
        foreground: 0xEBDBB2,
        card: 0x32302F,
        card_foreground: 0xEBDBB2,
        popover: 0x32302F,
        popover_foreground: 0xEBDBB2,
        primary: 0xFABD2E,
        primary_foreground: 0x292929,
        secondary: 0x3B3735,
        secondary_foreground: 0xEBDBB2,
        muted: 0x3B3735,
        muted_foreground: 0xA89985,
        accent: 0x4F4945,
        accent_foreground: 0xEBDBB2,
        destructive: 0xFB4632,
        destructive_foreground: 0x292929,
        border: 0x4F4945,
        input: 0x4F4945,
        ring: 0xFABD2E,
    },
    ThemeDefinition {
        id: "rose-pine",
        label: "Rosé Pine",
        description: "Muted rose.",
        is_dark: true,
        background: 0x1A1825,
        foreground: 0xDEDDF4,
        card: 0x201D2F,
        card_foreground: 0xDEDDF4,
        popover: 0x201D2F,
        popover_foreground: 0xDEDDF4,
        primary: 0xC4A7E7,
        primary_foreground: 0x1A1825,
        secondary: 0x252239,
        secondary_foreground: 0xDEDDF4,
        muted: 0x252239,
        muted_foreground: 0x918DAA,
        accent: 0x403D52,
        accent_foreground: 0xDEDDF4,
        destructive: 0xEB6F93,
        destructive_foreground: 0x1A1825,
        border: 0x403D52,
        input: 0x403D52,
        ring: 0xC4A7E7,
    },
    ThemeDefinition {
        id: "one-dark",
        label: "One Dark",
        description: "Editor classic.",
        is_dark: true,
        background: 0x282C34,
        foreground: 0xABB2BF,
        card: 0x21252B,
        card_foreground: 0xABB2BF,
        popover: 0x21252B,
        popover_foreground: 0xABB2BF,
        primary: 0x61AFEF,
        primary_foreground: 0x21252B,
        secondary: 0x3E4451,
        secondary_foreground: 0xABB2BF,
        muted: 0x2F333D,
        muted_foreground: 0x9DA5B4,
        accent: 0x3E4451,
        accent_foreground: 0xABB2BF,
        destructive: 0xE06C75,
        destructive_foreground: 0x21252B,
        border: 0x3E4451,
        input: 0x3E4451,
        ring: 0x61AFEF,
    },
    ThemeDefinition {
        id: "kanagawa",
        label: "Kanagawa",
        description: "Ink and wave blue.",
        is_dark: true,
        background: 0x1F1F28,
        foreground: 0xDDD8BB,
        card: 0x16161D,
        card_foreground: 0xDDD8BB,
        popover: 0x16161D,
        popover_foreground: 0xDDD8BB,
        primary: 0x7D9CD8,
        primary_foreground: 0x16161D,
        secondary: 0x2A2A37,
        secondary_foreground: 0xDDD8BB,
        muted: 0x2A2A37,
        muted_foreground: 0xC8C092,
        accent: 0x353545,
        accent_foreground: 0xDDD8BB,
        destructive: 0xE46776,
        destructive_foreground: 0x16161D,
        border: 0x353545,
        input: 0x353545,
        ring: 0x7D9CD8,
    },
    ThemeDefinition {
        id: "everforest",
        label: "Everforest",
        description: "Mossy green night.",
        is_dark: true,
        background: 0x2C343A,
        foreground: 0xD4C7AB,
        card: 0x282F34,
        card_foreground: 0xD4C7AB,
        popover: 0x282F34,
        popover_foreground: 0xD4C7AB,
        primary: 0xA8C181,
        primary_foreground: 0x282F34,
        secondary: 0x354045,
        secondary_foreground: 0xD4C7AB,
        muted: 0x354045,
        muted_foreground: 0x859389,
        accent: 0x3D484D,
        accent_foreground: 0xD4C7AB,
        destructive: 0xE77E80,
        destructive_foreground: 0x282F34,
        border: 0x3D484D,
        input: 0x3D484D,
        ring: 0xA8C181,
    },
    ThemeDefinition {
        id: "github-dark",
        label: "GitHub Dark",
        description: "GitHub’s dark canvas.",
        is_dark: true,
        background: 0x0D1117,
        foreground: 0xE7EEF3,
        card: 0x161B22,
        card_foreground: 0xE7EEF3,
        popover: 0x161B22,
        popover_foreground: 0xE7EEF3,
        primary: 0x57A5FF,
        primary_foreground: 0x0D1117,
        secondary: 0x21252C,
        secondary_foreground: 0xE7EEF3,
        muted: 0x21252C,
        muted_foreground: 0x8A939E,
        accent: 0x2F353C,
        accent_foreground: 0xE7EEF3,
        destructive: 0xF85249,
        destructive_foreground: 0x0D1117,
        border: 0x2F353C,
        input: 0x2F353C,
        ring: 0x57A5FF,
    },
    ThemeDefinition {
        id: "dark",
        label: "Midnight Amber",
        description: "Near-black with amber.",
        is_dark: true,
        background: 0x0D0D11,
        foreground: 0xEDEDED,
        card: 0x131117,
        card_foreground: 0xEDEDED,
        popover: 0x131117,
        popover_foreground: 0xEDEDED,
        primary: 0xE9900C,
        primary_foreground: 0xFFFFFF,
        secondary: 0x1C1A23,
        secondary_foreground: 0xEDEDED,
        muted: 0x24212B,
        muted_foreground: 0x9591A1,
        accent: 0x3A2403,
        accent_foreground: 0xF5AB3D,
        destructive: 0xA52727,
        destructive_foreground: 0xFFFFFF,
        border: 0x211E29,
        input: 0x211E29,
        ring: 0xE9900C,
    },
    ThemeDefinition {
        id: "latte",
        label: "Catppuccin Latte",
        description: "Pastel daylight.",
        is_dark: false,
        background: 0xEFF1F5,
        foreground: 0x4B4E68,
        card: 0xFFFFFF,
        card_foreground: 0x4B4E68,
        popover: 0xFFFFFF,
        popover_foreground: 0x4B4E68,
        primary: 0x8839EF,
        primary_foreground: 0xFFFFFF,
        secondary: 0xCDD1DB,
        secondary_foreground: 0x4B4E68,
        muted: 0xE6E9EF,
        muted_foreground: 0x6C6F84,
        accent: 0xDDE1E9,
        accent_foreground: 0x4B4E68,
        destructive: 0xD20F39,
        destructive_foreground: 0xFFFFFF,
        border: 0xCDD1DB,
        input: 0xCDD1DB,
        ring: 0x8839EF,
    },
    ThemeDefinition {
        id: "solarized-light",
        label: "Solarized Light",
        description: "Low-contrast daylight.",
        is_dark: false,
        background: 0xFDF6E2,
        foreground: 0x586E74,
        card: 0xFFFBF0,
        card_foreground: 0x586E74,
        popover: 0xFFFBF0,
        popover_foreground: 0x586E74,
        primary: 0x278BD3,
        primary_foreground: 0xFDF6E2,
        secondary: 0xEDE7D4,
        secondary_foreground: 0x586E74,
        muted: 0xEDE7D4,
        muted_foreground: 0x647A82,
        accent: 0xEDE7D4,
        accent_foreground: 0x586E74,
        destructive: 0xDC312E,
        destructive_foreground: 0xFDF6E2,
        border: 0xD8D2C0,
        input: 0xD8D2C0,
        ring: 0x278BD3,
    },
    ThemeDefinition {
        id: "github-light",
        label: "GitHub Light",
        description: "GitHub's paper canvas.",
        is_dark: false,
        background: 0xFFFFFF,
        foreground: 0x1F2328,
        card: 0xF5F7FA,
        card_foreground: 0x1F2328,
        popover: 0xF5F7FA,
        popover_foreground: 0x1F2328,
        primary: 0x096CDC,
        primary_foreground: 0xFFFFFF,
        secondary: 0xE9EDF1,
        secondary_foreground: 0x1F2328,
        muted: 0xF5F7FA,
        muted_foreground: 0x656D76,
        accent: 0xE9EDF1,
        accent_foreground: 0x1F2328,
        destructive: 0xCE222D,
        destructive_foreground: 0xFFFFFF,
        border: 0xCFD6DE,
        input: 0xCFD6DE,
        ring: 0x096CDC,
    },
    ThemeDefinition {
        id: "light",
        label: "Paper",
        description: "Warm paper, ink primary.",
        is_dark: false,
        background: 0xF9F8F0,
        foreground: 0x212121,
        card: 0xFFFFFF,
        card_foreground: 0x212121,
        popover: 0xFFFFFF,
        popover_foreground: 0x212121,
        primary: 0x212121,
        primary_foreground: 0xFFFFFF,
        secondary: 0xEDEDED,
        secondary_foreground: 0x212121,
        muted: 0xF2F2F2,
        muted_foreground: 0x707070,
        accent: 0xF3F1E7,
        accent_foreground: 0x212121,
        destructive: 0xDC2828,
        destructive_foreground: 0xFFFFFF,
        border: 0xDEDEDE,
        input: 0xDEDEDE,
        ring: 0x212121,
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

/// Rebuild the design tokens for `id` and hand them to the shell. The light
/// and dark slots carry the same values because the chosen catalogue entry
/// is a complete palette, not a mode toggle.
pub fn apply(id: &str) {
    let Some(definition) = theme_by_id(id) else {
        return;
    };
    let mut tokens = (*nsh_core::element::tokens()).clone();
    let color = |value: u32| nsh_core::element::AdaptiveColor::rgb(value, value);
    tokens.set_color("background", color(definition.background));
    tokens.set_color("foreground", color(definition.foreground));
    tokens.set_color("card", color(definition.card));
    tokens.set_color("card-foreground", color(definition.card_foreground));
    tokens.set_color("popover", color(definition.popover));
    tokens.set_color("popover-foreground", color(definition.popover_foreground));
    tokens.set_color("primary", color(definition.primary));
    tokens.set_color("primary-foreground", color(definition.primary_foreground));
    tokens.set_color("secondary", color(definition.secondary));
    tokens.set_color(
        "secondary-foreground",
        color(definition.secondary_foreground),
    );
    tokens.set_color("muted", color(definition.muted));
    tokens.set_color("muted-foreground", color(definition.muted_foreground));
    tokens.set_color("accent", color(definition.accent));
    tokens.set_color("accent-foreground", color(definition.accent_foreground));
    tokens.set_color("destructive", color(definition.destructive));
    tokens.set_color(
        "destructive-foreground",
        color(definition.destructive_foreground),
    );
    tokens.set_color("border", color(definition.border));
    tokens.set_color("input", color(definition.input));
    tokens.set_color("ring", color(definition.ring));
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
}
