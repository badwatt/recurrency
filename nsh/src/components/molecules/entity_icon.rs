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
        "bot" => Some(asset!("icons/bot.svg")),
        "cloud" => Some(asset!("icons/cloud.svg")),
        "cpu" => Some(asset!("icons/cpu.svg")),
        "gamepad" => Some(asset!("icons/gamepad.svg")),
        "globe" => Some(asset!("icons/globe.svg")),
        "key" => Some(asset!("icons/key.svg")),
        "lock" => Some(asset!("icons/lock.svg")),
        "mail" => Some(asset!("icons/mail.svg")),
        "mic" => Some(asset!("icons/mic.svg")),
        "monitor" => Some(asset!("icons/monitor.svg")),
        "smartphone" => Some(asset!("icons/smartphone.svg")),
        "shield" => Some(asset!("icons/shield.svg")),
        "star" => Some(asset!("icons/star.svg")),
        "sun" => Some(asset!("icons/sun.svg")),
        "upload" => Some(asset!("icons/upload.svg")),
        "webhook" => Some(asset!("icons/webhook.svg")),
        "sliders" => Some(asset!("icons/sliders.svg")),
        "sparkles" => Some(asset!("icons/sparkles.svg")),
        "hash" => Some(asset!("icons/hash.svg")),
        "play" => Some(asset!("icons/play.svg")),
        "pause" => Some(asset!("icons/pause.svg")),
        "download" => Some(asset!("icons/download.svg")),
        "server" => Some(asset!("icons/server.svg")),
        "info" => Some(asset!("icons/info.svg")),
        _ => None,
    }
}

const LIVE_ICONS: &[&str] = &[
    "wallet",
    "tag",
    "receipt",
    "card",
    "subscriptions",
    "transactions",
    "debts",
    "calendar",
    "reports",
    "settings",
    "database",
    "bell",
    "clock",
    "trending",
    "notes",
    "bot",
    "cloud",
    "cpu",
    "gamepad",
    "globe",
    "key",
    "lock",
    "mail",
    "mic",
    "monitor",
    "smartphone",
    "shield",
    "star",
    "sun",
    "upload",
    "webhook",
    "sliders",
    "sparkles",
    "hash",
    "play",
    "pause",
    "download",
    "server",
    "info",
];

/// Icon whose name and colour are signals, for editable previews.
pub fn entity_icon_live(name: Signal<String>, color: Signal<String>, size: f32) -> AnyElement {
    let mut container = view().items_center().justify_center();
    for key in LIVE_ICONS {
        let Some(icon) = icon_for(key) else { continue };
        let key = *key;
        container = container.child(
            when(
                move || name.get().eq_ignore_ascii_case(key),
                move || {
                    image(icon)
                        .size(px(size))
                        .pointer_events(PointerEvents::None)
                        .tint_color(move || rgba(color_value(&color.get())))
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        );
    }
    container.into_element()
}

/// Icon with a fixed name and a reactive colour.
pub fn entity_icon_colored(name: &str, color: Signal<String>, size: f32) -> AnyElement {
    match icon_for(name) {
        Some(icon) => view()
            .items_center()
            .justify_center()
            .child(
                image(icon)
                    .size(px(size))
                    .pointer_events(PointerEvents::None)
                    .tint_color(move || rgba(color_value(&color.get()))),
            )
            .into_element(),
        None => view()
            .h(px(size * 0.7))
            .w(px(size * 0.7))
            .rounded(px(999.0))
            .bg(move || rgba(color_value(&color.get())))
            .into_element(),
    }
}

fn color_value(hex: &str) -> u32 {
    let value = hex.trim_start_matches('#');
    let parsed = u32::from_str_radix(value, 16).unwrap_or(0x8b5cf6);
    (parsed << 8) | 0xff
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
