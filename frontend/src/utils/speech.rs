//! Voice dictation for the AI chat.
//!
//! The browser speech API is used when available. Android WebViews and
//! desktop WebViews without `SpeechRecognition` fall back to recording audio
//! with `MediaRecorder`; the backend transcribes it with the configured AI
//! provider. Native Android/desktop clients inherit the same behavior.

use crate::state::voice::VoiceMode;
use dioxus::prelude::*;

const TEXT_PREFIX: &str = "TEXT:";
const AUDIO_PREFIX: &str = "AUDIO:";
const PCM_PREFIX: &str = "PCM:";
const ERROR_PREFIX: &str = "ERROR:";
const CANCELLED: &str = "CANCELLED";
const RESULT_HEADER_PARTS: usize = 3;
const MAX_RECORDING_MS: u32 = 60_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpeechInput {
    Text(String),
    Audio { mime_type: String, base64: String },
    Pcm(String),
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "android"),
    feature = "desktop"
))]
#[path = "speech_native.rs"]
mod native_audio;

/// Start dictation and wait for a final transcript or a recorded audio clip.
pub async fn listen(mode: VoiceMode) -> Result<SpeechInput, String> {
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        feature = "desktop"
    ))]
    {
        if mode == VoiceMode::Browser {
            listen_js(mode).await
        } else {
            native_audio::listen(mode).await
        }
    }
    #[cfg(not(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        feature = "desktop"
    )))]
    {
        listen_js(mode).await
    }
}

async fn listen_js(mode: VoiceMode) -> Result<SpeechInput, String> {
    let result = listen_once(mode).await;
    if mode == VoiceMode::Auto
        && let Err(error) = &result
        && should_retry_with_provider(error)
    {
        return listen_once(VoiceMode::Provider).await;
    }
    result
}

async fn listen_once(mode: VoiceMode) -> Result<SpeechInput, String> {
    let mut eval = document::eval(&start_script(mode));
    let raw = eval
        .recv::<String>()
        .await
        .map_err(|_| "Voice input was interrupted".to_string())?;
    parse_result(&raw)
}

fn should_retry_with_provider(error: &str) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("speech service is unavailable")
        || error.contains("service-not-allowed")
        || error.contains("not supported on this device")
}

/// Stop the active recording or browser recognition. The pending `listen`
/// future resolves with whatever was captured so far.
pub fn stop() {
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        feature = "desktop"
    ))]
    native_audio::stop();
    let _ = document::eval(
        "try { window.__recurrencySpeechStop && window.__recurrencySpeechStop(); } catch (_) {}",
    );
}

fn parse_result(raw: &str) -> Result<SpeechInput, String> {
    if raw == CANCELLED {
        return Err("Dictation cancelled".to_string());
    }
    if let Some(text) = raw.strip_prefix(TEXT_PREFIX) {
        let text = text.trim();
        return if text.is_empty() {
            Err("No speech detected".to_string())
        } else {
            Ok(SpeechInput::Text(text.to_string()))
        };
    }
    if let Some(pcm) = raw.strip_prefix(PCM_PREFIX) {
        if pcm.trim().is_empty() {
            return Err("The recorded audio is empty".to_string());
        }
        return Ok(SpeechInput::Pcm(pcm.trim().to_string()));
    }
    if let Some(rest) = raw.strip_prefix(AUDIO_PREFIX) {
        let mut parts = rest.splitn(RESULT_HEADER_PARTS, ':');
        let mime_type = parts.next().unwrap_or_default().trim();
        let base64 = parts.next().unwrap_or_default().trim();
        if mime_type.starts_with("audio/") && !base64.is_empty() {
            return Ok(SpeechInput::Audio {
                mime_type: mime_type.to_string(),
                base64: base64.to_string(),
            });
        }
        return Err("The recorded audio is invalid".to_string());
    }
    if let Some(error) = raw.strip_prefix(ERROR_PREFIX) {
        return Err(speech_error(error));
    }
    Err("Voice input failed".to_string())
}

fn speech_error(code: &str) -> String {
    match code {
        "unsupported" => {
            "Voice input is not supported on this device. Check the microphone permission."
                .to_string()
        }
        "not-allowed" | "microphone" | "audio-capture" => {
            "Microphone permission denied or no microphone was found".to_string()
        }
        "no-speech" => "No speech detected. Try again".to_string(),
        "network" | "service-not-allowed" => "The speech service is unavailable".to_string(),
        "aborted" => "Dictation was cancelled".to_string(),
        "recording" => "Could not record audio".to_string(),
        other => format!("Voice input failed: {other}"),
    }
}

const SPEECH_START_JS: &str = r#"
(async function () {
    const mode = __VOICE_MODE__;
    const send = (value) => {
        try { dioxus.send(value); } catch (_) {}
    };
    const Speech = window.SpeechRecognition || window.webkitSpeechRecognition;
    const nativeBridge = window.RecurrencySpeech;

    if ((mode === 'native' || mode === 'auto') && nativeBridge && nativeBridge.start) {
        await new Promise((resolve) => {
            let finished = false;
            const finish = (value) => {
                if (finished) return;
                finished = true;
                send(value);
                resolve();
            };
            window.__recurrencySpeechResult = finish;
            window.__recurrencySpeechStop = () => {
                try { nativeBridge.stop && nativeBridge.stop(); } catch (_) {}
                finish('CANCELLED');
            };
            try {
                nativeBridge.start();
            } catch (_) {
                finish('ERROR:unsupported');
            }
        });
        return;
    }
    if (mode === 'native') {
        send('ERROR:unsupported');
        return;
    }

    if ((mode === 'browser' || mode === 'auto') && Speech) {
        await new Promise((resolve) => {
            let finished = false;
            let cancelled = false;
            let heard = false;
            const recognition = new Speech();
            const finish = (value) => {
                if (finished) return;
                finished = true;
                send(value);
                resolve();
            };
            recognition.lang = navigator.language || 'en-US';
            recognition.continuous = false;
            recognition.interimResults = false;
            recognition.maxAlternatives = 1;
            recognition.onresult = (event) => {
                let text = '';
                for (let i = event.resultIndex; i < event.results.length; i += 1) {
                    text += event.results[i][0].transcript || '';
                }
                text = text.trim();
                if (text) {
                    heard = true;
                    finish('TEXT:' + text);
                }
            };
            recognition.onerror = (event) => {
                if (cancelled) {
                    finish('CANCELLED');
                } else {
                    finish('ERROR:' + (event.error || 'speech'));
                }
            };
            recognition.onend = () => {
                if (!heard) finish(cancelled ? 'CANCELLED' : 'ERROR:no-speech');
            };
            window.__recurrencySpeechStop = () => {
                cancelled = true;
                try { recognition.stop(); } catch (_) { finish('CANCELLED'); }
            };
            try {
                recognition.start();
            } catch (_) {
                finish('ERROR:unsupported');
            }
        });
        return;
    }
    if (mode === 'browser') {
        send('ERROR:unsupported');
        return;
    }

    if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia || typeof MediaRecorder === 'undefined') {
        send('ERROR:unsupported');
        return;
    }

    let stream;
    try {
        stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    } catch (_) {
        send('ERROR:microphone');
        return;
    }

    const preferred = ['audio/webm;codecs=opus', 'audio/webm', 'audio/mp4', 'audio/ogg'];
    const supported = typeof MediaRecorder.isTypeSupported === 'function';
    const selected = preferred.find((type) => !supported || MediaRecorder.isTypeSupported(type)) || '';
    let recorder;
    try {
        recorder = selected ? new MediaRecorder(stream, { mimeType: selected }) : new MediaRecorder(stream);
    } catch (_) {
        stream.getTracks().forEach((track) => track.stop());
        send('ERROR:recording');
        return;
    }

    const chunks = [];
    await new Promise((resolve) => {
        let finished = false;
        const finish = (value) => {
            if (finished) return;
            finished = true;
            stream.getTracks().forEach((track) => track.stop());
            send(value);
            resolve();
        };
        recorder.ondataavailable = (event) => {
            if (event.data && event.data.size > 0) chunks.push(event.data);
        };
        recorder.onerror = () => finish('ERROR:recording');
        recorder.onstop = async () => {
            try {
                const type = (recorder.mimeType || selected || 'audio/webm').split(';')[0];
                const blob = new Blob(chunks, { type });
                const buffer = await blob.arrayBuffer();
                if (buffer.byteLength === 0) {
                    finish('ERROR:no-speech');
                    return;
                }
                if (mode === 'local') {
                    const context = new AudioContext();
                    const decoded = await context.decodeAudioData(buffer);
                    await context.close();
                    const rate = 16000;
                    const frames = Math.max(1, Math.ceil(decoded.duration * rate));
                    const offline = new OfflineAudioContext(1, frames, rate);
                    const source = offline.createBufferSource();
                    source.buffer = decoded;
                    source.connect(offline.destination);
                    source.start();
                    const rendered = await offline.startRendering();
                    const samples = rendered.getChannelData(0);
                    const bytes = new Uint8Array(samples.buffer.slice(0));
                    let binary = '';
                    const chunk = 0x8000;
                    for (let i = 0; i < bytes.length; i += chunk) {
                        binary += String.fromCharCode.apply(null, bytes.subarray(i, i + chunk));
                    }
                    finish('PCM:' + btoa(binary));
                    return;
                }
                const bytes = new Uint8Array(buffer);
                let binary = '';
                const chunk = 0x8000;
                for (let i = 0; i < bytes.length; i += chunk) {
                    binary += String.fromCharCode.apply(null, bytes.subarray(i, i + chunk));
                }
                finish('AUDIO:' + type + ':' + btoa(binary));
            } catch (_) {
                finish('ERROR:recording');
            }
        };
        window.__recurrencySpeechStop = () => {
            try {
                if (recorder.state !== 'inactive') recorder.stop();
            } catch (_) {
                finish('ERROR:recording');
            }
        };
        try {
            recorder.start();
        } catch (_) {
            finish('ERROR:recording');
            return;
        }
        setTimeout(() => {
            try {
                if (recorder.state !== 'inactive') recorder.stop();
            } catch (_) {}
        }, MAX_RECORDING_MS_PLACEHOLDER);
    });
})();
"#;

fn start_script(mode: VoiceMode) -> String {
    SPEECH_START_JS
        .replace("__VOICE_MODE__", &format!("\"{}\"", mode.id()))
        .replace(
            "MAX_RECORDING_MS_PLACEHOLDER",
            &MAX_RECORDING_MS.to_string(),
        )
}

/// Use the generated script so the recording limit stays configurable in Rust.
pub fn start_script_for_test(mode: VoiceMode) -> String {
    start_script(mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_final_transcript() {
        assert_eq!(
            parse_result("TEXT:Hola, crea Netflix"),
            Ok(SpeechInput::Text("Hola, crea Netflix".to_string()))
        );
    }

    #[test]
    fn parses_recorded_pcm_for_local_models() {
        assert_eq!(
            parse_result("PCM:YWJj"),
            Ok(SpeechInput::Pcm("YWJj".to_string()))
        );
        assert!(parse_result("PCM:").is_err());
    }

    #[test]
    fn parses_recorded_audio() {
        let input = parse_result("AUDIO:audio/webm:YWJj").unwrap();
        assert_eq!(
            input,
            SpeechInput::Audio {
                mime_type: "audio/webm".to_string(),
                base64: "YWJj".to_string()
            }
        );
    }

    #[test]
    fn rejects_unknown_audio_and_cancelled_input() {
        assert!(parse_result("AUDIO:text/plain:YWJj").is_err());
        assert!(parse_result("CANCELLED").is_err());
        assert!(parse_result("TEXT:   ").is_err());
    }

    #[test]
    fn maps_microphone_errors_to_readable_messages() {
        assert!(
            parse_result("ERROR:not-allowed")
                .unwrap_err()
                .contains("Microphone")
        );
        assert!(
            parse_result("ERROR:no-speech")
                .unwrap_err()
                .contains("No speech")
        );
        assert!(
            parse_result("ERROR:unsupported")
                .unwrap_err()
                .contains("not supported")
        );
    }

    #[test]
    fn unavailable_browser_speech_can_fall_back_to_provider() {
        assert!(should_retry_with_provider(
            "The speech service is unavailable"
        ));
        assert!(should_retry_with_provider(
            "Voice input failed: service-not-allowed"
        ));
        assert!(should_retry_with_provider(
            "Voice input is not supported on this device. Check the microphone permission."
        ));
        assert!(!should_retry_with_provider("No speech detected"));
        assert!(!should_retry_with_provider("Microphone permission denied"));
        assert!(!should_retry_with_provider("Dictation cancelled"));
    }

    #[test]
    fn recording_starts_script_contains_both_modes() {
        let script = start_script_for_test(VoiceMode::Auto);
        assert!(script.contains("SpeechRecognition"));
        assert!(script.contains("MediaRecorder"));
        assert!(script.contains("getUserMedia"));
        assert!(script.contains("60000"));
        assert!(script.contains("OfflineAudioContext"));
        assert!(script.contains("RecurrencySpeech"));
        assert!(!script.contains("MAX_RECORDING_MS_PLACEHOLDER"));
        assert!(!script.contains("__VOICE_MODE__"));
    }
}
