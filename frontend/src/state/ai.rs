use dioxus::prelude::*;

use crate::api::ai::AiSettings;

/// Server-provided AI configuration. `None` means it has not loaded yet.
pub static AI_SETTINGS: GlobalSignal<Option<AiSettings>> = Signal::global(|| None);

pub fn is_configured() -> bool {
    AI_SETTINGS
        .try_read()
        .ok()
        .and_then(|settings| settings.clone())
        .is_some_and(|settings| settings.enabled && settings.configured)
}
