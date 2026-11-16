//! Per-device voice input preferences for the AI assistant.
//!
//! The provider and API key live on the server; the speech strategy is a
//! device capability and stays on the device.

use dioxus::prelude::*;

const MODE_KEY: &str = "recurrency-voice-mode";
const MODEL_KEY: &str = "recurrency-voice-model";
const MODE_FILE: &str = "voice_mode.txt";
const MODEL_FILE: &str = "voice_model.txt";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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
}

fn load_value(key: &str, file: &str) -> Option<String> {
    #[cfg(test)]
    {
        let _ = (key, file);
        return None;
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = file;
        return web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item(key).ok().flatten());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = key;
        let dir = crate::state::auth::get_app_data_dir()?;
        std::fs::read_to_string(dir.join("recurrency").join(file)).ok()
    }
}

fn store_value(key: &str, file: &str, value: &str) {
    #[cfg(test)]
    {
        let _ = (key, file, value);
        return;
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = file;
        if let Some(storage) =
            web_sys::window().and_then(|window| window.local_storage().ok().flatten())
        {
            let _ = storage.set_item(key, value);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = key;
        if let Some(dir) = crate::state::auth::get_app_data_dir() {
            let app_dir = dir.join("recurrency");
            if std::fs::create_dir_all(&app_dir).is_ok() {
                let _ = std::fs::write(app_dir.join(file), value);
            }
        }
    }
}

fn load_mode() -> VoiceMode {
    VoiceMode::parse(&load_value(MODE_KEY, MODE_FILE).unwrap_or_default())
}

fn load_model() -> String {
    load_value(MODEL_KEY, MODEL_FILE).unwrap_or_default()
}

pub static VOICE_MODE: GlobalSignal<VoiceMode> = Signal::global(load_mode);
pub static VOICE_MODEL: GlobalSignal<String> = Signal::global(load_model);

pub fn set_mode(mode: VoiceMode) {
    *VOICE_MODE.write() = mode;
    store_value(MODE_KEY, MODE_FILE, mode.id());
}

pub fn set_model(model_id: &str) {
    *VOICE_MODEL.write() = model_id.to_string();
    store_value(MODEL_KEY, MODEL_FILE, model_id);
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
    fn labels_and_descriptions_are_present() {
        for mode in VoiceMode::ALL {
            assert!(!mode.label().is_empty());
            assert!(!mode.description().is_empty());
        }
    }
}
