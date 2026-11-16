//! Native desktop microphone capture.
//!
//! WebKitGTK does not expose `getUserMedia` reliably, so desktop builds record
//! through the operating system audio API and reuse the same transcription
//! paths as the web client.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use base64::Engine;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample, StreamConfig};

use crate::state::voice::VoiceMode;
use crate::utils::speech::SpeechInput;

const SAMPLE_RATE_FOR_WHISPER: u32 = 16_000;
const MAX_RECORDING_SECONDS: u64 = 60;
const WAV_HEADER_BYTES: usize = 44;
const BITS_PER_SAMPLE: u16 = 16;
const MONO_CHANNELS: u16 = 1;
const PCM_FORMAT: u16 = 1;

static STOP: AtomicBool = AtomicBool::new(false);
static ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn stop() {
    STOP.store(true, Ordering::Relaxed);
}

pub async fn listen(mode: VoiceMode) -> Result<SpeechInput, String> {
    if mode == VoiceMode::Native {
        return Err("Android speech is not available on desktop".to_string());
    }
    if ACTIVE.swap(true, Ordering::AcqRel) {
        return Err("The microphone is already recording".to_string());
    }
    STOP.store(false, Ordering::Relaxed);
    let recording = tokio::task::spawn_blocking(record).await;
    ACTIVE.store(false, Ordering::Release);
    let (samples, sample_rate) = recording.map_err(|error| error.to_string())??;

    let use_local = mode == VoiceMode::Local
        || (mode == VoiceMode::Auto
            && crate::utils::local_speech::MODELS
                .iter()
                .any(crate::utils::local_speech::installed));
    if use_local {
        let samples = resample(&samples, sample_rate, SAMPLE_RATE_FOR_WHISPER);
        let mut bytes = Vec::with_capacity(samples.len() * 4);
        for sample in samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        return Ok(SpeechInput::Pcm(
            base64::engine::general_purpose::STANDARD.encode(bytes),
        ));
    }

    Ok(SpeechInput::Audio {
        mime_type: "audio/wav".to_string(),
        base64: encode_wav(&samples, sample_rate),
    })
}

fn record() -> Result<(Vec<f32>, u32), String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "No microphone was found".to_string())?;
    let supported = device
        .default_input_config()
        .map_err(|error| format!("Could not open the microphone: {error}"))?;
    let sample_rate = supported.sample_rate();
    let channels = usize::from(supported.channels());
    let format = supported.sample_format();
    let config: StreamConfig = supported.into();
    let samples = Arc::new(Mutex::new(Vec::<f32>::new()));

    let stream = match format {
        SampleFormat::I8 => build_stream::<i8>(&device, config, samples.clone(), channels),
        SampleFormat::I16 => build_stream::<i16>(&device, config, samples.clone(), channels),
        SampleFormat::I32 => build_stream::<i32>(&device, config, samples.clone(), channels),
        SampleFormat::F32 => build_stream::<f32>(&device, config, samples.clone(), channels),
        other => Err(format!("Unsupported microphone sample format: {other}")),
    }?;
    stream
        .play()
        .map_err(|error| format!("Could not start the microphone: {error}"))?;
    let started = Instant::now();
    while !STOP.load(Ordering::Relaxed)
        && started.elapsed() < Duration::from_secs(MAX_RECORDING_SECONDS)
    {
        std::thread::sleep(Duration::from_millis(25));
    }
    let _ = stream.pause();
    drop(stream);
    let samples = samples
        .lock()
        .map_err(|_| "Could not read the recording".to_string())?
        .clone();
    if samples.is_empty() {
        return Err("No speech detected".to_string());
    }
    Ok((samples, sample_rate))
}

fn build_stream<T>(
    device: &cpal::Device,
    config: StreamConfig,
    samples: Arc<Mutex<Vec<f32>>>,
    channels: usize,
) -> Result<cpal::Stream, String>
where
    T: SizedSample + Sample,
    f32: FromSample<T>,
{
    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                if let Ok(mut output) = samples.lock() {
                    for frame in data.chunks(channels) {
                        let value = frame
                            .iter()
                            .map(|sample| f32::from_sample(*sample))
                            .sum::<f32>()
                            / channels as f32;
                        output.push(value);
                    }
                }
            },
            |error| eprintln!("[audio] microphone stream error: {error}"),
            None,
        )
        .map_err(|error| format!("Could not read the microphone: {error}"))
}

fn resample(samples: &[f32], input_rate: u32, output_rate: u32) -> Vec<f32> {
    if input_rate == output_rate {
        return samples.to_vec();
    }
    let ratio = f64::from(input_rate) / f64::from(output_rate);
    let output_len = ((samples.len() as f64) / ratio).round() as usize;
    (0..output_len)
        .map(|index| {
            let source = index as f64 * ratio;
            let left = source.floor() as usize;
            let right = (left + 1).min(samples.len().saturating_sub(1));
            let fraction = (source - left as f64) as f32;
            samples[left.min(samples.len().saturating_sub(1))] * (1.0 - fraction)
                + samples[right] * fraction
        })
        .collect()
}

fn encode_wav(samples: &[f32], sample_rate: u32) -> String {
    let data_len = samples.len() * 2;
    let mut bytes = Vec::with_capacity(WAV_HEADER_BYTES + data_len);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&((36 + data_len) as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&PCM_FORMAT.to_le_bytes());
    bytes.extend_from_slice(&MONO_CHANNELS.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * u32::from(MONO_CHANNELS) * u32::from(BITS_PER_SAMPLE) / 8;
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&(MONO_CHANNELS * BITS_PER_SAMPLE / 8).to_le_bytes());
    bytes.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(data_len as u32).to_le_bytes());
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16;
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resampling_keeps_the_requested_duration() {
        let samples = vec![0.0; 48_000];
        let resampled = resample(&samples, 48_000, 16_000);
        assert_eq!(resampled.len(), 16_000);
    }

    #[test]
    fn wav_has_a_valid_header_and_payload() {
        let encoded = encode_wav(&[0.0, 1.0, -1.0], 16_000);
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        assert_eq!(&bytes[..4], b"RIFF");
        assert_eq!(bytes.len(), WAV_HEADER_BYTES + 3 * 2);
        assert_eq!(&bytes[8..12], b"WAVE");
    }
}
