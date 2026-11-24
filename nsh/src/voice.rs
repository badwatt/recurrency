//! Per-device voice input preference. The engine choice is a device
//! capability, so it stays in local storage instead of the server profile.

use nsh_core::storage::Storage;

const MODE_KEY: &str = "recurrency.voice-mode";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VoiceMode {
    #[default]
    Auto,
    Native,
    Browser,
    Provider,
    Local,
}

impl VoiceMode {
    pub const ALL: [Self; 5] = [
        Self::Auto,
        Self::Native,
        Self::Browser,
        Self::Provider,
        Self::Local,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Native => "native",
            Self::Browser => "browser",
            Self::Provider => "provider",
            Self::Local => "local",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "native" => Self::Native,
            "browser" => Self::Browser,
            "provider" => Self::Provider,
            "local" => Self::Local,
            _ => Self::Auto,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Automatic",
            Self::Native => "Android speech",
            Self::Browser => "Browser speech",
            Self::Provider => "Provider transcription",
            Self::Local => "On-device model",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Auto => "Use the best available option for this device.",
            Self::Native => "Use Android's built-in speech recognizer.",
            Self::Browser => "Use the browser's SpeechRecognition engine.",
            Self::Provider => "Upload a short clip to your AI provider.",
            Self::Local => "Run a speech model entirely on this device.",
        }
    }

    pub fn available(self) -> bool {
        match self {
            Self::Auto | Self::Provider => true,
            Self::Native => cfg!(target_os = "android"),
            Self::Browser => cfg!(target_arch = "wasm32"),
            Self::Local => false,
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::Native if !cfg!(target_os = "android") => "Android only",
            Self::Browser if !cfg!(target_arch = "wasm32") => "Web build only",
            Self::Local => "Desktop app only",
            _ => "",
        }
    }
}

pub fn saved() -> VoiceMode {
    VoiceMode::parse(&Storage::get_item(MODE_KEY).unwrap_or_default())
}

pub fn save(mode: VoiceMode) {
    let _ = Storage::set_item(MODE_KEY, mode.id());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mode_round_trips_through_its_id() {
        for mode in VoiceMode::ALL {
            assert_eq!(VoiceMode::parse(mode.id()), mode);
        }
    }

    #[test]
    fn unknown_modes_fall_back_to_automatic() {
        assert_eq!(VoiceMode::parse("something-else"), VoiceMode::Auto);
        assert_eq!(VoiceMode::parse(""), VoiceMode::Auto);
    }

    #[test]
    fn every_mode_has_copy_and_a_valid_availability_hint() {
        for mode in VoiceMode::ALL {
            assert!(!mode.label().is_empty());
            assert!(!mode.description().is_empty());
            if !mode.available() {
                assert!(!mode.hint().is_empty());
            }
        }
    }
}
