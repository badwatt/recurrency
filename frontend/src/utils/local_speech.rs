//! On-device speech-to-text models for desktop builds.
//!
//! Models are downloaded on demand and run with sherpa-onnx. Android uses the
//! system SpeechRecognizer instead, so this module is intentionally desktop
//! only.

pub const TINY_ID: &str = "whisper-tiny";
pub const BASE_ID: &str = "whisper-base";
pub const SMALL_ID: &str = "whisper-small";
pub const MODEL_DIR: &str = "speech-models";
const GIB: u64 = 1024 * 1024 * 1024;
const MIB: u64 = 1024 * 1024;
const DOWNLOAD_TIMEOUT_SECONDS: u64 = 60 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelFile {
    pub name: &'static str,
    pub url: &'static str,
    pub bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalModel {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub size_label: &'static str,
    pub min_ram_gib: u64,
    pub speed: &'static str,
    pub files: &'static [ModelFile],
}

const TINY_FILES: &[ModelFile] = &[
    ModelFile {
        name: "tiny-encoder.int8.onnx",
        url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-tiny/resolve/main/tiny-encoder.int8.onnx",
        bytes: 13 * MIB,
    },
    ModelFile {
        name: "tiny-decoder.int8.onnx",
        url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-tiny/resolve/main/tiny-decoder.int8.onnx",
        bytes: 90 * MIB,
    },
    ModelFile {
        name: "tiny-tokens.txt",
        url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-tiny/resolve/main/tiny-tokens.txt",
        bytes: MIB,
    },
];

const BASE_FILES: &[ModelFile] = &[
    ModelFile {
        name: "base-encoder.int8.onnx",
        url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-base/resolve/main/base-encoder.int8.onnx",
        bytes: 29 * MIB,
    },
    ModelFile {
        name: "base-decoder.int8.onnx",
        url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-base/resolve/main/base-decoder.int8.onnx",
        bytes: 131 * MIB,
    },
    ModelFile {
        name: "base-tokens.txt",
        url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-base/resolve/main/base-tokens.txt",
        bytes: MIB,
    },
];

const SMALL_FILES: &[ModelFile] = &[
    ModelFile {
        name: "small-encoder.int8.onnx",
        url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-small/resolve/main/small-encoder.int8.onnx",
        bytes: 112 * MIB,
    },
    ModelFile {
        name: "small-decoder.int8.onnx",
        url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-small/resolve/main/small-decoder.int8.onnx",
        bytes: 262 * MIB,
    },
    ModelFile {
        name: "small-tokens.txt",
        url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-small/resolve/main/small-tokens.txt",
        bytes: MIB,
    },
];

pub const MODELS: &[LocalModel] = &[
    LocalModel {
        id: TINY_ID,
        name: "Whisper Tiny",
        description: "Fast multilingual dictation for modest laptops.",
        size_label: "104 MB",
        min_ram_gib: 4,
        speed: "Fastest",
        files: TINY_FILES,
    },
    LocalModel {
        id: BASE_ID,
        name: "Whisper Base",
        description: "Balanced accuracy and speed for everyday notes.",
        size_label: "161 MB",
        min_ram_gib: 8,
        speed: "Balanced",
        files: BASE_FILES,
    },
    LocalModel {
        id: SMALL_ID,
        name: "Whisper Small",
        description: "Better accuracy for longer, detailed dictation.",
        size_label: "375 MB",
        min_ram_gib: 16,
        speed: "Most accurate",
        files: SMALL_FILES,
    },
];

pub fn available() -> bool {
    cfg!(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        feature = "desktop"
    ))
}

pub fn find(id: &str) -> Option<&'static LocalModel> {
    MODELS.iter().find(|model| model.id == id)
}

pub fn recommended_for(ram_bytes: u64, cores: usize) -> &'static LocalModel {
    if ram_bytes < 8 * GIB || cores < 4 {
        &MODELS[0]
    } else if ram_bytes < 16 * GIB {
        &MODELS[1]
    } else {
        &MODELS[2]
    }
}

pub fn recommendation_reason(model: &LocalModel, ram_bytes: u64, cores: usize) -> String {
    let ram = ram_bytes as f64 / GIB as f64;
    format!(
        "Recommended for {ram:.0} GB RAM and {cores} CPU cores: {} ({})",
        model.name, model.speed
    )
}

pub fn total_bytes(model: &LocalModel) -> u64 {
    model.files.iter().map(|file| file.bytes).sum()
}

fn clean_transcription(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let lower = trimmed.to_ascii_lowercase();
    let is_sound_tag = (trimmed.starts_with('[') && trimmed.ends_with(']'))
        || (trimmed.starts_with('(') && trimmed.ends_with(')'));
    if is_sound_tag
        && [
            "music",
            "silence",
            "blank_audio",
            "background noise",
            "no speech",
        ]
        .iter()
        .any(|tag| lower.contains(tag))
    {
        return String::new();
    }
    trimmed.to_string()
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes >= GIB {
        format!("{:.1} GB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.0} MB", bytes as f64 / MIB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "android"),
    feature = "desktop"
))]
mod desktop {
    use super::*;
    use base64::Engine;
    use futures_util::StreamExt;
    use std::path::PathBuf;

    pub fn hardware() -> (u64, usize) {
        let mut system = sysinfo::System::new();
        system.refresh_memory();
        let ram = system.total_memory();
        let cores = std::thread::available_parallelism()
            .map(|value| value.get())
            .unwrap_or(1);
        (ram, cores)
    }

    pub fn model_dir(model: &LocalModel) -> Option<PathBuf> {
        crate::state::auth::get_app_data_dir()
            .map(|dir| dir.join("recurrency").join(MODEL_DIR).join(model.id))
    }

    pub fn installed(model: &LocalModel) -> bool {
        let Some(dir) = model_dir(model) else {
            return false;
        };
        model.files.iter().all(|file| {
            std::fs::metadata(dir.join(file.name))
                .map(|metadata| metadata.len() > 0)
                .unwrap_or(false)
        })
    }

    pub fn installed_bytes(model: &LocalModel) -> u64 {
        let Some(dir) = model_dir(model) else {
            return 0;
        };
        model
            .files
            .iter()
            .filter_map(|file| std::fs::metadata(dir.join(file.name)).ok())
            .map(|metadata| metadata.len())
            .sum()
    }

    pub async fn download(
        model: &'static LocalModel,
        mut progress: impl FnMut(u64, u64),
    ) -> Result<(), String> {
        let dir = model_dir(model).ok_or_else(|| "No data directory".to_string())?;
        std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
        let total = total_bytes(model);
        let mut downloaded = installed_bytes(model).min(total);
        progress(downloaded, total);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(DOWNLOAD_TIMEOUT_SECONDS))
            .build()
            .map_err(|error| error.to_string())?;
        for file in model.files {
            let target = dir.join(file.name);
            if std::fs::metadata(&target)
                .map(|metadata| metadata.len() > 0)
                .unwrap_or(false)
            {
                continue;
            }
            let part = target.with_extension("part");
            let response = client
                .get(file.url)
                .send()
                .await
                .map_err(|error| format!("Could not download {}: {error}", file.name))?;
            if !response.status().is_success() {
                return Err(format!(
                    "Could not download {}: HTTP {}",
                    file.name,
                    response.status()
                ));
            }
            let mut stream = response.bytes_stream();
            let mut output = std::fs::File::create(&part).map_err(|error| error.to_string())?;
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|error| error.to_string())?;
                use std::io::Write;
                output
                    .write_all(&chunk)
                    .map_err(|error| error.to_string())?;
                downloaded += chunk.len() as u64;
                progress(downloaded.min(total), total);
            }
            drop(output);
            std::fs::rename(&part, &target).map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    pub fn delete(model: &LocalModel) -> Result<(), String> {
        let dir = model_dir(model).ok_or_else(|| "No data directory".to_string())?;
        if dir.is_dir() {
            std::fs::remove_dir_all(dir).map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    pub async fn transcribe_pcm(model_id: &str, base64_audio: &str) -> Result<String, String> {
        let raw = base64::engine::general_purpose::STANDARD
            .decode(base64_audio)
            .map_err(|_| "The recorded audio is invalid".to_string())?;
        if raw.len() < 4 || raw.len() % 4 != 0 {
            return Err("The recorded audio is empty".to_string());
        }
        let samples: Vec<f32> = raw
            .as_chunks::<4>()
            .0
            .iter()
            .map(|chunk| f32::from_le_bytes(*chunk))
            .collect();
        let model = find(model_id)
            .filter(|model| installed(model))
            .or_else(|| MODELS.iter().find(|model| installed(model)))
            .ok_or_else(|| "Download a local speech model first".to_string())?;
        let dir = model_dir(model).ok_or_else(|| "No data directory".to_string())?;
        let file_path = |suffix: &str| {
            model
                .files
                .iter()
                .find(|file| file.name.contains(suffix))
                .map(|file| dir.join(file.name).to_string_lossy().into_owned())
        };
        let tokens = file_path("tokens").ok_or_else(|| "Speech tokens are missing".to_string())?;
        let encoder =
            file_path("encoder").ok_or_else(|| "Speech encoder is missing".to_string())?;
        let decoder =
            file_path("decoder").ok_or_else(|| "Speech decoder is missing".to_string())?;
        tokio::task::spawn_blocking(move || {
            let mut config = sherpa_onnx::OfflineRecognizerConfig::default();
            config.model_config.whisper = sherpa_onnx::OfflineWhisperModelConfig {
                encoder: Some(encoder),
                decoder: Some(decoder),
                language: None,
                task: Some("transcribe".to_string()),
                tail_paddings: -1,
                ..Default::default()
            };
            config.model_config.tokens = Some(tokens);
            config.model_config.num_threads = std::thread::available_parallelism()
                .map(|v| v.get())
                .unwrap_or(1) as i32;
            let recognizer = sherpa_onnx::OfflineRecognizer::create(&config)
                .ok_or_else(|| "Could not load the speech model".to_string())?;
            let stream = recognizer.create_stream();
            stream.accept_waveform(16_000, &samples);
            recognizer.decode(&stream);
            stream
                .get_result()
                .map(|result| clean_transcription(&result.text))
                .filter(|text| !text.is_empty())
                .ok_or_else(|| "No speech detected".to_string())
        })
        .await
        .map_err(|error| error.to_string())?
    }

    pub fn installed_path(model: &LocalModel) -> Option<PathBuf> {
        let dir = model_dir(model)?;
        if model.files.iter().all(|file| dir.join(file.name).is_file()) {
            Some(dir)
        } else {
            None
        }
    }
}

#[cfg(not(all(
    not(target_arch = "wasm32"),
    not(target_os = "android"),
    feature = "desktop"
)))]
mod desktop {
    use super::*;

    pub fn hardware() -> (u64, usize) {
        (0, 1)
    }

    pub fn installed(_model: &LocalModel) -> bool {
        false
    }

    pub fn installed_bytes(_model: &LocalModel) -> u64 {
        0
    }

    pub async fn download(
        _model: &'static LocalModel,
        _progress: impl FnMut(u64, u64),
    ) -> Result<(), String> {
        Err("Local speech models are available in the desktop app".to_string())
    }

    pub fn delete(_model: &LocalModel) -> Result<(), String> {
        Err("Local speech models are available in the desktop app".to_string())
    }

    pub async fn transcribe_pcm(_model_id: &str, _base64_audio: &str) -> Result<String, String> {
        Err("Local speech models are available in the desktop app".to_string())
    }
}

pub use desktop::{delete, download, hardware, installed, transcribe_pcm};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommended_model_tracks_ram_and_cores() {
        assert_eq!(recommended_for(4 * GIB, 4).id, TINY_ID);
        assert_eq!(recommended_for(8 * GIB, 8).id, BASE_ID);
        assert_eq!(recommended_for(32 * GIB, 16).id, SMALL_ID);
        assert_eq!(recommended_for(32 * GIB, 2).id, TINY_ID);
    }

    #[test]
    fn catalog_models_have_download_files() {
        for model in MODELS {
            assert!(!model.files.is_empty());
            assert!(
                model
                    .files
                    .iter()
                    .all(|file| file.url.starts_with("https://"))
            );
            assert!(total_bytes(model) > 0);
        }
    }

    #[test]
    fn recommendation_mentions_hardware() {
        let reason = recommendation_reason(&MODELS[1], 8 * GIB, 8);
        assert!(reason.contains("8 GB RAM"));
        assert!(reason.contains("Whisper Base"));
    }

    #[test]
    fn silence_annotations_are_not_inserted_into_the_chat() {
        assert!(clean_transcription("[music]").is_empty());
        assert!(clean_transcription("(silence)").is_empty());
        assert_eq!(clean_transcription("Create Netflix"), "Create Netflix");
    }

    #[test]
    fn sizes_are_human_readable() {
        assert_eq!(format_bytes(104 * MIB), "104 MB");
        assert_eq!(format_bytes(2 * GIB), "2.0 GB");
    }

    #[test]
    fn availability_tracks_the_desktop_feature() {
        assert_eq!(
            available(),
            cfg!(all(
                not(target_arch = "wasm32"),
                not(target_os = "android"),
                feature = "desktop"
            ))
        );
    }
}
