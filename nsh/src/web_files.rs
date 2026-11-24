//! Browser file downloads and pickers. Native builds use the OS dialogs.

use js_sys::{Array, Promise, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, FileReader, HtmlAnchorElement, HtmlInputElement, Url};

fn document() -> Result<web_sys::Document, String> {
    web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| "The browser document is unavailable.".to_string())
}

fn blob_parts(bytes: &[u8]) -> Array {
    let parts = Array::new();
    parts.push(&Uint8Array::from(bytes));
    parts
}

fn blob_bag(mime: &str) -> BlobPropertyBag {
    let bag = BlobPropertyBag::new();
    bag.set_type(mime);
    bag
}

/// Download bytes through a temporary object URL and anchor.
pub fn download_bytes(name: &str, mime: &str, bytes: &[u8]) -> Result<(), String> {
    let document = document()?;
    let parts = blob_parts(bytes);
    let blob = Blob::new_with_u8_array_sequence_and_options(parts.as_ref(), &blob_bag(mime))
        .map_err(|_| "Could not create the download.".to_string())?;
    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|_| "Could not create the download URL.".to_string())?;
    let anchor = document
        .create_element("a")
        .map_err(|_| "Could not create the download link.".to_string())?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|_| "Could not create the download link.".to_string())?;
    anchor.set_href(&url);
    anchor.set_download(name);
    if let Some(body) = document.body() {
        let _ = body.append_child(&anchor);
    }
    anchor.click();
    anchor.remove();
    let _ = Url::revoke_object_url(&url);
    Ok(())
}

pub fn download_text(name: &str, mime: &str, text: &str) -> Result<(), String> {
    download_bytes(name, mime, text.as_bytes())
}

#[derive(Clone, Copy)]
enum ReadMode {
    Text,
    DataUrl,
}

async fn pick_file(accept: &str, mode: ReadMode) -> Result<Option<String>, String> {
    let document = document()?;
    let input = document
        .create_element("input")
        .map_err(|_| "Could not create the file picker.".to_string())?
        .dyn_into::<HtmlInputElement>()
        .map_err(|_| "Could not create the file picker.".to_string())?;
    input.set_type("file");
    input.set_accept(accept);
    let promise = Promise::new(&mut |resolve, _reject| {
        let input_for_change = input.clone();
        let resolve_for_change = resolve.clone();
        let on_change = Closure::<dyn FnMut()>::new(move || {
            let input_after = input_for_change.clone();
            let resolve_after = resolve_for_change.clone();
            let Some(files) = input_after.files() else {
                let _ = resolve_after.call1(&JsValue::NULL, &JsValue::from_str(""));
                return;
            };
            let Some(file) = files.get(0) else {
                let _ = resolve_after.call1(&JsValue::NULL, &JsValue::from_str(""));
                return;
            };
            let Ok(reader) = FileReader::new() else {
                let _ = resolve_after.call1(&JsValue::NULL, &JsValue::from_str(""));
                return;
            };
            let reader_for_load = reader.clone();
            let resolve_for_load = resolve_after.clone();
            let on_load = Closure::<dyn FnMut()>::new(move || {
                let value = reader_for_load
                    .result()
                    .unwrap_or_else(|_| JsValue::from_str(""));
                let _ = resolve_for_load.call1(&JsValue::NULL, &value);
            });
            reader.set_onload(Some(on_load.as_ref().unchecked_ref()));
            on_load.forget();
            let _ = match mode {
                ReadMode::Text => reader.read_as_text(&file),
                ReadMode::DataUrl => reader.read_as_data_url(&file),
            };
            input_after.remove();
        });
        input.set_onchange(Some(on_change.as_ref().unchecked_ref()));
        on_change.forget();
        let resolve_for_cancel = resolve.clone();
        let input_for_cancel = input.clone();
        let on_cancel = Closure::<dyn FnMut()>::new(move || {
            input_for_cancel.remove();
            let _ = resolve_for_cancel.call1(&JsValue::NULL, &JsValue::from_str(""));
        });
        input.set_oncancel(Some(on_cancel.as_ref().unchecked_ref()));
        on_cancel.forget();
        if let Some(body) = document.body() {
            let _ = body.append_child(&input);
        }
        input.click();
    });
    let value = JsFuture::from(promise)
        .await
        .map_err(|_| "Could not read the selected file.".to_string())?;
    let text = value.as_string().unwrap_or_default();
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

pub async fn pick_text_file(accept: &str) -> Result<Option<String>, String> {
    pick_file(accept, ReadMode::Text).await
}

pub async fn pick_data_url(accept: &str) -> Result<Option<String>, String> {
    pick_file(accept, ReadMode::DataUrl).await
}
