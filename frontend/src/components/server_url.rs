use crate::components::toast::TOAST;
use crate::state::app_mode::{normalize_server_url, APP_MODE, DEFAULT_NATIVE_SERVER};
use crate::state::auth::AUTH;
use dioxus::prelude::*;

/// Editable server URL for native (Android/desktop) builds.
///
/// Web builds are served from the same origin as the API and intentionally do
/// not expose this control. On native, changing servers invalidates the
/// current session, so the settings version logs out after saving.
#[component]
pub fn ServerUrlEditor(#[props(default = false)] logout_on_change: bool) -> Element {
    let mut value = use_signal(|| {
        APP_MODE()
            .mode
            .server_url()
            .unwrap_or(DEFAULT_NATIVE_SERVER)
            .to_string()
    });
    let mut error = use_signal(String::new);
    let mut saved = use_signal(String::new);

    let mut save = move |_| {
        let raw = value().trim().to_string();
        let url = match normalize_server_url(&raw) {
            Ok(url) => url,
            Err(message) => {
                saved.set(String::new());
                error.set(message);
                return;
            }
        };

        let previous = APP_MODE().mode.server_url().unwrap_or_default().to_string();
        let changed = previous != url;

        error.set(String::new());
        value.set(url.clone());
        APP_MODE.write().set_server(url.clone());
        crate::api::client::set_base_url(&url);

        if changed && logout_on_change && AUTH().is_authenticated {
            AUTH.write().logout();
            AUTH.write().persist_to_storage();
            TOAST.write().success("Server saved. Sign in again.");
            navigator().push(crate::Route::Login {});
            return;
        }

        saved.set("Server saved".to_string());
        TOAST.write().success("Server saved");
    };

    rsx! {
        div { class: "space-y-3",
            label {
                r#for: "settings-server-url",
                class: "text-sm font-medium",
                "Server URL"
            }
            div { class: "flex flex-col gap-2 sm:flex-row",
                input {
                    id: "settings-server-url",
                    r#type: "url",
                    inputmode: "url",
                    autocapitalize: "off",
                    autocorrect: "off",
                    spellcheck: "false",
                    placeholder: "https://recurrency.example.com",
                    value: value,
                    oninput: move |e| {
                        value.set(e.value());
                        error.set(String::new());
                        saved.set(String::new());
                    },
                    class: "flex h-10 min-w-0 flex-1 rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm focus:outline-none focus:ring-1 focus:ring-ring",
                }
                button {
                    r#type: "button",
                    class: "inline-flex h-10 shrink-0 items-center justify-center rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90",
                    onclick: move |_| save(()),
                    "Save server"
                }
            }
            if !error().is_empty() {
                p { class: "text-sm text-destructive", {error()} }
            } else if !saved().is_empty() {
                p { class: "text-sm text-muted-foreground", {saved()} }
            } else {
                p { class: "text-xs text-muted-foreground",
                    "Use the address of your Recurrency server, without /api."
                }
            }
        }
    }
}
