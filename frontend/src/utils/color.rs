//! Hex color validation and small color helpers shared by account and
//! category forms, cards, and report exports.

/// Default color for new accounts and categories.
pub const DEFAULT_COLOR: &str = "#6b7280";
/// Color used when a stored color is missing or malformed.
pub const FALLBACK_COLOR: &str = "#6b7280";
/// Foreground used on dark swatches.
pub const LIGHT_TEXT: &str = "#ffffff";
/// Foreground used on light swatches.
pub const DARK_TEXT: &str = "#111827";

/// True for `#rrggbb` colors. The UI picker and the manual input use this.
pub fn is_valid_hex(value: &str) -> bool {
    normalize_hex(value).is_some()
}

/// Return the trimmed, lowercase `#rrggbb` form when `value` is valid.
pub fn normalize_hex(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.len() != 7
        || !trimmed.starts_with('#')
        || !trimmed[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}

/// Append an alpha channel (`00`–`ff`) to a valid hex color. Invalid colors
/// fall back to [`FALLBACK_COLOR`] so CSS never receives a broken value.
pub fn with_alpha(value: &str, alpha: u8) -> String {
    match normalize_hex(value) {
        Some(hex) => format!("{hex}{alpha:02x}"),
        None => FALLBACK_COLOR.to_string(),
    }
}

/// Split a `#rrggbb` color into its channels.
pub fn rgb(value: &str) -> Option<(u8, u8, u8)> {
    let hex = normalize_hex(value)?;
    let parse = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    Some((parse(1..3)?, parse(3..5)?, parse(5..7)?))
}

/// Pick a readable foreground for a background color.
pub fn readable_text(value: &str) -> &'static str {
    let Some((r, g, b)) = rgb(value) else {
        return LIGHT_TEXT;
    };
    let luminance = 299 * u32::from(r) + 587 * u32::from(g) + 114 * u32::from(b);
    if luminance > 160_000 {
        DARK_TEXT
    } else {
        LIGHT_TEXT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_lower_and_upper_hex() {
        assert!(is_valid_hex("#ef4444"));
        assert!(is_valid_hex("#EF4444"));
        assert!(is_valid_hex(" #3b82f6 "));
    }

    #[test]
    fn rejects_wrong_lengths_and_characters() {
        assert!(!is_valid_hex(""));
        assert!(!is_valid_hex("#fff"));
        assert!(!is_valid_hex("ef4444"));
        assert!(!is_valid_hex("#gg4444"));
        assert!(!is_valid_hex("#ef44445"));
    }

    #[test]
    fn normalize_returns_lowercase_hex() {
        assert_eq!(normalize_hex(" #EF4444 "), Some("#ef4444".to_string()));
        assert_eq!(normalize_hex("not-a-color"), None);
    }

    #[test]
    fn alpha_is_appended_only_to_valid_hex() {
        assert_eq!(with_alpha("#EF4444", 0x20), "#ef444420");
        assert_eq!(with_alpha("invalid", 0x20), FALLBACK_COLOR.to_string());
        assert_eq!(with_alpha("#123", 0x80), FALLBACK_COLOR.to_string());
    }

    #[test]
    fn rgb_parses_hex_channels() {
        assert_eq!(rgb("#0a1B2c"), Some((10, 27, 44)));
        assert_eq!(rgb("#ffffff"), Some((255, 255, 255)));
        assert_eq!(rgb("#000000"), Some((0, 0, 0)));
        assert_eq!(rgb("#xyzxyz"), None);
    }

    #[test]
    fn readable_text_picks_contrast() {
        assert_eq!(readable_text("#ffffff"), DARK_TEXT);
        assert_eq!(readable_text("#000000"), LIGHT_TEXT);
        assert_eq!(readable_text("#ef4444"), LIGHT_TEXT);
        assert_eq!(readable_text("#fde68a"), DARK_TEXT);
        assert_eq!(readable_text("invalid"), LIGHT_TEXT);
    }

    #[test]
    fn default_and_fallback_colors_are_valid() {
        assert!(is_valid_hex(DEFAULT_COLOR));
        assert!(is_valid_hex(FALLBACK_COLOR));
    }
}
