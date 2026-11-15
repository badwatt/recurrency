//! Hex color validation shared by account and category forms.

/// True for `#rrggbb` colors. The UI picker and the manual input use this.
pub fn is_valid_hex(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.len() == 7
        && trimmed.starts_with('#')
        && trimmed[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
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
}
