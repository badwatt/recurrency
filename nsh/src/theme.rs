//! Typography and spacing shared by the native screens.
//!
//! Colors come from `theme.json`, which mirrors the current dark palette.
//! The remaining palette catalogue will be moved here as the screens land.

pub const SANS: &str = "Cabinet Grotesk";
pub const MONO: &str = "JetBrains Mono";

pub const PAGE_GAP: f32 = 24.0;
pub const CARD_RADIUS: f32 = 12.0;

/// Success badges (active states) use the `success` token when the theme
/// defines it; the primary color is the fallback.
pub fn success() -> nsh_core::element::AdaptiveColor {
    nsh_base::theme().color("success").unwrap_or_else(|| nsh_base::theme().primary)
}
