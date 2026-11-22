//! Microphone capture. The web runtime records through the browser's
//! MediaRecorder; native builds keep the file-based voice-note path.

#[derive(Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub enum Capture {
    Started,
    Stopped { base64: String, mime: String },
}

pub fn supported() -> bool {
    cfg!(target_arch = "wasm32")
}

#[cfg(target_arch = "wasm32")]
pub async fn toggle() -> Result<Capture, String> {
    let script = r#"
(async () => {
  if (window.__recurrencyRecorder) {
    const recorder = window.__recurrencyRecorder;
    return await new Promise((resolve, reject) => {
      recorder.onstop = async () => {
        try {
          const blob = new Blob(recorder.chunks, { type: recorder.mimeType || 'audio/webm' });
          const buffer = await blob.arrayBuffer();
          const bytes = new Uint8Array(buffer);
          let binary = '';
          const chunk = 0x8000;
          for (let index = 0; index < bytes.length; index += chunk) {
            binary += String.fromCharCode.apply(null, bytes.subarray(index, index + chunk));
          }
          const base64 = btoa(binary);
          recorder.stream.getTracks().forEach(track => track.stop());
          window.__recurrencyRecorder = null;
          resolve({ base64, mime: blob.type || 'audio/webm' });
        } catch (error) {
          reject(String(error));
        }
      };
      recorder.stop();
    });
  }
  const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
  const recorder = new MediaRecorder(stream);
  recorder.chunks = [];
  recorder.stream = stream;
  recorder.ondataavailable = event => {
    if (event.data.size) recorder.chunks.push(event.data);
  };
  recorder.start();
  window.__recurrencyRecorder = recorder;
  return { started: true };
})()
"#;
    let promise = js_sys::eval(script).map_err(|error| {
        error
            .as_string()
            .unwrap_or_else(|| "Could not start the microphone.".to_string())
    })?;
    let promise = js_sys::Promise::from(promise);
    let value = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|error| {
            error
                .as_string()
                .unwrap_or_else(|| "Could not use the microphone.".to_string())
        })?;
    if js_sys::Reflect::get(&value, &wasm_bindgen::JsValue::from_str("started"))
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        return Ok(Capture::Started);
    }
    let field = |key: &str| {
        js_sys::Reflect::get(&value, &wasm_bindgen::JsValue::from_str(key))
            .ok()
            .and_then(|value| value.as_string())
            .unwrap_or_default()
    };
    let base64 = field("base64");
    if base64.is_empty() {
        return Err("The microphone returned no audio.".to_string());
    }
    Ok(Capture::Stopped {
        base64,
        mime: field("mime"),
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn toggle() -> Result<Capture, String> {
    Err(
        "Microphone recording is available in the web build; choose a voice-note file instead."
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_web_build_advertises_microphone_support() {
        assert_eq!(supported(), cfg!(target_arch = "wasm32"));
    }
}
