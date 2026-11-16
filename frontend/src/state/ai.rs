use dioxus::prelude::*;

use crate::api::ai::AiSettings;

/// Server-provided AI configuration. `None` means it has not loaded yet.
pub static AI_SETTINGS: GlobalSignal<Option<AiSettings>> = Signal::global(|| None);

pub fn is_configured() -> bool {
    // Use `read()` (not `try_read()`) so components rendering the AI entry
    // points subscribe to the global signal and appear as soon as settings
    // finish loading.
    AI_SETTINGS
        .read()
        .as_ref()
        .is_some_and(|settings| settings.enabled && settings.configured)
}

#[cfg(test)]
mod tests {
    #[test]
    fn ai_entry_points_subscribe_to_the_settings_signal() {
        let source = include_str!("ai.rs");
        assert!(source.contains("AI_SETTINGS\n        .read()"));
        assert!(!source.contains("AI_SETTINGS\n        .try_read()"));
    }
}
