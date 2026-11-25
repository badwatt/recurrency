//! Unit tests for the Recurrency frontend.

pub mod android_auth_tests;
pub mod android_icon_tests;
pub mod android_persistence_tests;
pub mod android_theme_tests;
pub mod dashboard_redesign_tests;
pub mod design_system_tests;
pub mod empty_states_tests;
pub mod index_html_tests;
mod layout_animation_tests;
pub mod layout_redesign_tests;
pub mod page_render_tests;
pub mod privacy_tests;
pub mod shared_page_render_tests;
pub mod shared_ui_tests;
#[cfg(test)]
mod dioxus_config_tests {
    #[test]
    fn microphone_permission_is_declared_for_native_bundles() {
        let config = include_str!("../../../Dioxus.toml");
        assert!(config.contains("[permissions]"));
        assert!(config.contains("microphone"));
        assert!(config.contains("Record voice input"));
    }
}
