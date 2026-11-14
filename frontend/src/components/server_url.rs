use crate::components::toast::TOAST;
use crate::state::app_mode::{APP_MODE, NATIVE_SERVER_OVERRIDE, normalize_server_url};
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
            .unwrap_or(NATIVE_SERVER_OVERRIDE)
            .to_string()
    });
    let mut error = use_signal(String::new);
    let mut saved = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut scanning = use_signal(|| false);

    let save = use_callback(move |_: ()| {
        if loading() {
            return;
        }
        let raw = value().trim().to_string();
        if let Err(message) = normalize_server_url(&raw) {
            saved.set(String::new());
            error.set(message);
            return;
        }

        loading.set(true);
        error.set(String::new());
        saved.set(String::new());
        spawn(async move {
            match crate::api::server::validate_server(&raw).await {
                Ok(url) => {
                    let previous = APP_MODE().mode.server_url().unwrap_or_default().to_string();
                    let changed = previous != url;
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
                }
                Err(message) => error.set(message),
            }
            loading.set(false);
        });
    });

    let scan = move |_| {
        if scanning() {
            return;
        }
        scanning.set(true);
        error.set(String::new());
        saved.set(String::new());
        spawn(async move {
            match crate::utils::qr::scan().await {
                Ok(raw) => match crate::api::server::parse_qr_payload(&raw) {
                    Ok(url) => {
                        value.set(url);
                        saved.set("QR read. Press Save server to verify it.".to_string());
                    }
                    Err(message) => error.set(message),
                },
                Err(message) => error.set(message),
            }
            scanning.set(false);
        });
    };

    let use_local = move |_| {
        APP_MODE.write().set_local();
        if AUTH().is_authenticated {
            AUTH.write().logout();
            AUTH.write().persist_to_storage();
        }
        TOAST
            .write()
            .success("Local mode enabled. Your data stays on this device.");
        navigator().push(crate::Route::Login {});
    };

    let local_mode = APP_MODE().mode.is_offline();
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
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Enter {
                            save.call(());
                        }
                    },
                    oninput: move |e| {
                        value.set(e.value());
                        error.set(String::new());
                        saved.set(String::new());
                    },
                    class: "flex h-10 min-w-0 flex-1 rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm focus:outline-none focus:ring-1 focus:ring-ring",
                }
                button {
                    r#type: "button",
                    disabled: loading(),
                    class: "inline-flex h-10 w-full shrink-0 items-center justify-center rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50 sm:w-auto",
                    onclick: move |_| save.call(()),
                    if loading() { "Checking…" } else { "Save server" }
                }
            }
            if crate::utils::qr::scanner_available() {
                button {
                    r#type: "button",
                    disabled: scanning(),
                    class: "inline-flex h-10 w-full items-center justify-center rounded-md border border-border bg-background px-4 text-sm font-medium hover:bg-accent disabled:opacity-50",
                    onclick: scan,
                    if scanning() { "Scanning…" } else { "Scan QR" }
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
            if cfg!(not(target_arch = "wasm32")) {
                div { class: "flex flex-col gap-2 rounded-lg border border-border/60 bg-muted/20 p-3 sm:flex-row sm:items-center sm:justify-between",
                    div {
                        p { class: "text-sm font-medium", if local_mode { "Using this device" } else { "No server needed?" } }
                        p { class: "text-xs text-muted-foreground",
                            if local_mode {
                                "Your subscriptions and transactions are stored locally."
                            } else {
                                "Use Recurrency entirely offline. You can switch back later."
                            }
                        }
                    }
                    if !local_mode {
                        button {
                            r#type: "button",
                            class: "inline-flex h-9 shrink-0 items-center justify-center rounded-md border border-border bg-background px-3 text-sm font-medium hover:bg-accent",
                            onclick: use_local,
                            "Use this device"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod local_mode_tests {
    #[test]
    fn native_server_editor_offers_local_mode() {
        let src = include_str!("server_url.rs");
        assert!(src.contains("Use this device"));
        assert!(src.contains("set_local"));
        assert!(src.contains("Scan QR"));
        assert!(src.contains("parse_qr_payload"));
        assert!(src.contains("onkeydown"));
        assert!(src.contains("Key::Enter"));
    }
}
