//! Small colored icon for an account or category row. Stored icon names map
//! to the shared SVG set; unknown names fall back to a neutral dot.

use nsh_core::prelude::*;

fn icon_for(name: &str) -> Option<ImageAsset> {
    match name.trim().to_ascii_lowercase().as_str() {
        "wallet" => Some(asset!("icons/wallet.svg")),
        "tag" => Some(asset!("icons/tag.svg")),
        "receipt" => Some(asset!("icons/receipt.svg")),
        "card" => Some(asset!("icons/card.svg")),
        "subscriptions" => Some(asset!("icons/subscriptions.svg")),
        "transactions" => Some(asset!("icons/transactions.svg")),
        "debts" => Some(asset!("icons/debts.svg")),
        "calendar" => Some(asset!("icons/calendar.svg")),
        "reports" => Some(asset!("icons/reports.svg")),
        "settings" => Some(asset!("icons/settings.svg")),
        "database" => Some(asset!("icons/database.svg")),
        "bell" => Some(asset!("icons/bell.svg")),
        "clock" => Some(asset!("icons/clock.svg")),
        "trending" => Some(asset!("icons/trending.svg")),
        "notes" => Some(asset!("icons/notes.svg")),
        _ => None,
    }
}

pub fn entity_icon(name: &str, color: u32, size: f32) -> AnyElement {
    match icon_for(name) {
        Some(icon) => view()
            .items_center()
            .justify_center()
            .child(image(icon).size(px(size)).tint_color(move || rgba(color)))
            .into_element(),
        None => view()
            .h(px(size * 0.7))
            .w(px(size * 0.7))
            .rounded(px(999.0))
            .bg(move || rgba(color))
            .into_element(),
    }
}

#[cfg(test)]
mod tests {
    use super::icon_for;

    #[test]
    fn known_icon_names_resolve_case_insensitively() {
        assert!(icon_for("Wallet").is_some());
        assert!(icon_for(" tag ").is_some());
        assert!(icon_for("unknown").is_none());
    }
}
