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

/// Mobile browsers overlay their scrollbar; keep the same full-width cards as
/// the native app by hiding the reserved gutter below the compact breakpoint.
pub fn install_layout_css() {
    let Ok(document) = document() else { return };
    let Ok(style) = document.create_element("style") else {
        return;
    };
    style.set_text_content(Some(
        "@media (max-width: 900px){*{scrollbar-width:none}*::-webkit-scrollbar{display:none}}",
    ));
    if let Some(body) = document.body() {
        let _ = body.append_child(&style);
    }
}

/// Clear the page search field when Escape is pressed. The native input owns
/// its keys, so the browser listener edits the field and replays the input
/// event the app already listens to.
pub fn install_search_escape() {
    let Ok(document) = document() else { return };
    let on_key =
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
            if event.key() != "Escape" {
                return;
            }
            let Some(target) = document.active_element() else {
                return;
            };
            let Ok(input) = target.dyn_into::<HtmlInputElement>() else {
                return;
            };
            // Every search field in the app: the lists' "Search", "Search
            // money", "Search logo"…
            if !input.placeholder().starts_with("Search") {
                return;
            }
            if input.value().is_empty() {
                return;
            }
            input.set_value("");
            if let Ok(replayed) = web_sys::Event::new("input") {
                let _ = input.dispatch_event(&replayed);
            }
        });
    if let Some(document) = web_sys::window().and_then(|window| window.document()) {
        let _ =
            document.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref());
        on_key.forget();
    }
}

/// Remember the scroll offset of a named dialog so a remount (for example
/// while a nested logo picker opens) does not send the form back to the top.
pub fn install_dialog_scroll_memory() {
    let Ok(document) = document() else { return };
    let on_scroll = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
        let Some(target) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        else {
            return;
        };
        let Ok(Some(scroller)) = target.closest("[data-testid^='dialog-scroll:']") else {
            return;
        };
        let Some(id) = scroller.get_attribute("data-testid") else {
            return;
        };
        if !id.starts_with("dialog-scroll:") || id == "dialog-scroll:none" {
            return;
        }
        if let Some(storage) =
            web_sys::window().and_then(|window| window.local_storage().ok().flatten())
        {
            let key = format!("recurrency.dialog.{id}");
            let _ = storage.set_item(&key, &scroller.scroll_top().to_string());
        }
    });
    let _ = document.add_event_listener_with_callback_and_bool(
        "scroll",
        on_scroll.as_ref().unchecked_ref(),
        true,
    );
    on_scroll.forget();
}

fn apply_saved_dialog_scrolls() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Some(storage) = web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    else {
        return;
    };
    let Ok(nodes) = document.query_selector_all("[data-testid^='dialog-scroll:']") else {
        return;
    };
    for index in 0..nodes.length() {
        let Some(node) = nodes.get(index) else {
            continue;
        };
        let Ok(element) = node.dyn_into::<web_sys::Element>() else {
            continue;
        };
        let Some(id) = element.get_attribute("data-testid") else {
            continue;
        };
        if id == "dialog-scroll:none" {
            continue;
        }
        let Ok(Some(value)) = storage.get_item(&format!("recurrency.dialog.{id}")) else {
            continue;
        };
        if let (Ok(offset), Ok(scroller)) = (
            value.parse::<f64>(),
            element.dyn_into::<web_sys::HtmlElement>(),
        ) {
            scroller.set_scroll_top(offset as i32);
        }
    }
}

/// Reapply saved offsets after a nested dialog closes. The subscription form
/// can be rebuilt while the logo picker is open; this puts it back where the
/// user left it once the new DOM exists.
pub fn restore_dialog_scrolls() {
    apply_saved_dialog_scrolls();
    let closure = Closure::<dyn FnMut()>::new(apply_saved_dialog_scrolls);
    if let Some(window) = web_sys::window() {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            60,
        );
        closure.forget();
    }
}

pub fn clear_dialog_scroll(id: &str) {
    let Some(storage) = web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    else {
        return;
    };
    let _ = storage.remove_item(&format!("recurrency.dialog.dialog-scroll:{id}"));
}

/// Close an inline select when the pointer goes down outside its panel and
/// field. The document listener presses the panel's own trigger, so the toggle
/// runs through NSH's event loop instead of writing signals from JavaScript.
pub fn install_inline_select_dismiss() {
    let Ok(document) = document() else { return };
    let document_for_pointer = document.clone();
    let on_pointer =
        Closure::<dyn FnMut(web_sys::PointerEvent)>::new(move |event: web_sys::PointerEvent| {
            let target = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok());
            let Some(target) = target else { return };
            if target
                .closest("[data-testid='nsh-select-panel']")
                .ok()
                .flatten()
                .is_some()
                || target
                    .closest("[data-testid='nsh-select-field']")
                    .ok()
                    .flatten()
                    .is_some()
            {
                return;
            }
            let Ok(Some(panel)) =
                document_for_pointer.query_selector("[data-testid='nsh-select-panel']")
            else {
                return;
            };
            let Ok(Some(field)) = panel.closest("[data-testid='nsh-select-field']") else {
                return;
            };
            let Ok(Some(trigger)) = field.query_selector(".nsh-press") else {
                return;
            };
            let init = web_sys::PointerEventInit::new();
            init.set_bubbles(true);
            init.set_cancelable(true);
            init.set_composed(true);
            init.set_client_x(event.client_x());
            init.set_client_y(event.client_y());
            init.set_pointer_id(1);
            init.set_pointer_type("mouse");
            init.set_is_primary(true);
            for kind in ["pointerdown", "pointerup"] {
                if let Ok(synthetic) = web_sys::PointerEvent::new_with_event_init_dict(kind, &init)
                {
                    let _ = trigger.dispatch_event(synthetic.as_ref());
                }
            }
        });
    let _ = document
        .add_event_listener_with_callback("pointerdown", on_pointer.as_ref().unchecked_ref());
    on_pointer.forget();
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

thread_local! {
    static DROP_CALLBACKS: std::cell::RefCell<Option<(
        Closure<dyn FnMut(web_sys::DragEvent)>,
        Closure<dyn FnMut(web_sys::DragEvent)>,
        std::rc::Rc<dyn Fn(String)>,
    )>> = const { std::cell::RefCell::new(None) };
}

/// Reads a JSON backup dropped anywhere in the page. NSH's canvas cannot
/// expose a per-view DOM drop target on the web, so the document receives it.
pub fn install_backup_drop(callback: impl Fn(String) + 'static) {
    let callback: std::rc::Rc<dyn Fn(String)> = std::rc::Rc::new(callback);
    let already = DROP_CALLBACKS.with(|slot| slot.borrow().is_some());
    if already {
        DROP_CALLBACKS.with(|slot| {
            if let Some((_, _, existing)) = slot.borrow_mut().as_mut() {
                *existing = callback;
            }
        });
        return;
    }
    let Ok(document) = document() else { return };
    let over = Closure::<dyn FnMut(web_sys::DragEvent)>::new(|event: web_sys::DragEvent| {
        event.prevent_default()
    });
    let drop_callback = callback.clone();
    let drop = Closure::<dyn FnMut(web_sys::DragEvent)>::new(move |event: web_sys::DragEvent| {
        event.prevent_default();
        let Some(data) = event.data_transfer() else {
            return;
        };
        let Some(files) = data.files() else { return };
        let Some(file) = files.get(0) else { return };
        let Ok(reader) = FileReader::new() else {
            return;
        };
        let reader_for_load = reader.clone();
        let callback = drop_callback.clone();
        let on_load = Closure::<dyn FnMut()>::new(move || {
            if let Ok(value) = reader_for_load.result()
                && let Some(text) = value.as_string()
            {
                callback(text);
            }
        });
        reader.set_onload(Some(on_load.as_ref().unchecked_ref()));
        on_load.forget();
        let _ = reader.read_as_text(&file);
    });
    let _ = document.add_event_listener_with_callback("dragover", over.as_ref().unchecked_ref());
    let _ = document.add_event_listener_with_callback("drop", drop.as_ref().unchecked_ref());
    DROP_CALLBACKS.with(|slot| {
        *slot.borrow_mut() = Some((over, drop, callback));
    });
}

/// The hostname the page is served from, without its port.
pub fn page_host() -> Option<String> {
    let location = web_sys::window()?.location();
    let host = location.hostname().ok()?;
    (!host.is_empty()).then_some(host)
}

/// Route fragment of the current page (`#subscriptions` → `subscriptions`).
pub fn page_hash() -> Option<String> {
    let hash = web_sys::window()?.location().hash().ok()?;
    let value = hash.trim_start_matches('#').trim();
    (!value.is_empty()).then(|| value.to_string())
}

pub fn set_page_hash(value: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_hash(value);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn escape_clears_the_search_field_through_the_browser_listener() {
        let source = include_str!("web_files.rs");
        assert!(source.contains("install_search_escape"));
        assert!(source.contains("input.set_value(\"\")"));
    }

    #[test]
    fn outside_presses_dismiss_inline_selects() {
        let source = include_str!("web_files.rs");
        assert!(source.contains("install_inline_select_dismiss"));
        assert!(source.contains("nsh-select-panel"));
    }
}
