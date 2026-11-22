/// Visual treatment for an item while bulk selection mode is active.
///
/// Unselected items are dimmed and behave like disabled cards; the selected
/// card returns to full opacity with a clear ring. Outside selection mode this
/// returns an empty string so the card keeps its normal styling.
pub fn selection_card_class(selection_mode: bool, selected: bool) -> &'static str {
    if !selection_mode {
        ""
    } else if selected {
        "cursor-pointer opacity-100 ring-2 ring-primary/50 border-primary/50 bg-primary/5"
    } else {
        "cursor-pointer opacity-55 hover:opacity-80"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_class_is_idle_outside_selection_mode() {
        assert_eq!(selection_card_class(false, false), "");
        assert_eq!(selection_card_class(false, true), "");
    }

    #[test]
    fn selection_class_dims_unselected_and_marks_selected() {
        let unselected = selection_card_class(true, false);
        assert!(unselected.contains("opacity-55"));
        assert!(!unselected.contains("ring-2"));

        let selected = selection_card_class(true, true);
        assert!(selected.contains("opacity-100"));
        assert!(selected.contains("ring-2"));
        assert!(selected.contains("border-primary/50"));
    }
}
