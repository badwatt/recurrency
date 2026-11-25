use crate::api::server::{parse_qr_payload, validate_server};
use crate::components::button::{Button, ButtonVariant};
use crate::components::field::Field;
use crate::components::input::Input;
use crate::state::app_mode::{
    APP_MODE, NATIVE_SERVER_OVERRIDE, last_server_url, normalize_server_url,
};
use crate::state::auth::AUTH;
use dioxus::prelude::*;

#[component]
pub fn ServerSettingsPage() -> Element {
    let mut value = use_signal(|| {
        APP_MODE()
            .mode
            .server_url()
            .map(str::to_string)
            .or_else(last_server_url)
            .unwrap_or_else(|| NATIVE_SERVER_OVERRIDE.to_string())
    });
    let mut error = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut scanning = use_signal(|| false);

    let save = use_callback(move |_: ()| {
        if loading() {
            return;
        }
        let raw = value().trim().to_string();
        if let Err(message) = normalize_server_url(&raw) {
            error.set(message);
            return;
        }
        loading.set(true);
        error.set(String::new());
        status.set(String::new());
        spawn(async move {
            match validate_server(&raw).await {
                Ok(url) => {
                    APP_MODE.write().set_server(url.clone());
                    crate::api::client::set_base_url(&url);
                    if AUTH().is_authenticated {
                        AUTH.write().logout();
                        AUTH.write().persist_to_storage();
                    }
                    navigator().push(crate::Route::Login {});
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
        spawn(async move {
            match crate::utils::qr::scan().await {
                Ok(raw) => match parse_qr_payload(&raw) {
                    Ok(url) => {
                        value.set(url);
                        status.set("QR read. Press Save server to verify it.".to_string());
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
        navigator().push(crate::Route::Login {});
    };

    let local = APP_MODE().mode.is_offline();

    rsx! {
        div { class: "flex min-h-dvh items-center justify-center bg-background p-4",
            div { class: "w-full max-w-md rounded-xl border border-border bg-card p-6 shadow-sm",
                div { class: "flex items-start justify-between gap-3",
                    div {
                        h1 { class: "text-2xl font-bold", "Server" }
                        p { class: "mt-1 text-sm text-muted-foreground",
                            "Connect to your Recurrency server or keep using this device."
                        }
                    }
                    Link {
                        to: crate::Route::Login {},
                        class: "text-sm font-medium text-muted-foreground hover:text-foreground",
                        "Back"
                    }
                }

                div { class: "mt-5 rounded-lg border border-border/60 bg-muted/20 p-3",
                    p { class: "text-xs uppercase tracking-wider text-muted-foreground", "Current" }
                    p { class: "mt-1 text-sm font-medium",
                        if local { "This device (local mode)" } else { {value()} }
                    }
                }

                Field { label: "Server URL", html_for: "auth-server-url", hint: "Use the server root without /api. It is validated before saving.",
                    Input {
                        id: "auth-server-url",
                        r#type: "url",
                        inputmode: "url",
                        autocapitalize: "off",
                        autocorrect: "off",
                        spellcheck: "false",
                        placeholder: "https://recurrency.example.com",
                        value: value,
                        on_keydown: move |event: KeyboardEvent| {
                            if event.key() == Key::Enter {
                                save.call(());
                            }
                        },
                        on_input: move |next| {
                            value.set(next);
                            error.set(String::new());
                            status.set(String::new());
                        },
                    }
                }

                if !error().is_empty() {
                    p { class: "mt-3 text-sm text-destructive", {error()} }
                } else if !status().is_empty() {
                    p { class: "mt-3 text-sm text-muted-foreground", {status()} }
                }

                div { class: "mt-5 flex flex-col gap-2 sm:flex-row",
                    Button {
                        class: "sm:flex-1",
                        disabled: loading(),
                        onclick: move |_| save.call(()),
                        if loading() { "Checking server…" } else { "Save server" }
                    }
                    if crate::utils::qr::scanner_available() {
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: scanning(),
                            onclick: scan,
                            class: "sm:w-auto",
                            if scanning() { "Scanning…" } else { "Scan QR" }
                        }
                    }
                }

                Button {
                    variant: ButtonVariant::Outline,
                    full_width: true,
                    class: "mt-3",
                    onclick: use_local,
                    "Use this device"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api::server::parse_qr_payload;

    #[test]
    fn server_page_uses_validated_saves_and_qr() {
        let source = include_str!("server_settings.rs");
        assert!(source.contains("validate_server"));
        assert!(source.contains("parse_qr_payload"));
        assert!(source.contains("Use this device"));
        assert!(source.contains("w-full"));
        assert!(source.contains("sm:flex-1"));
        assert!(source.contains("onkeydown"));
        assert!(source.contains("Key::Enter"));
        assert!(source.contains("save.call"));
    }

    #[test]
    fn qr_payload_keeps_server_url() {
        assert_eq!(
            parse_qr_payload(r#"{"url":"https://recurrency.example.com"}"#).unwrap(),
            "https://recurrency.example.com"
        );
    }
}
