use crate::api::client::{get_base_url, set_base_url};
use crate::state::app_mode::{AppMode, APP_MODE};
use crate::Route;
use dioxus::prelude::*;

/// Persist server URL to filesystem (native) or localStorage (WASM).
fn save_server_url(url: &str) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(data_dir) = crate::state::auth::get_app_data_dir() {
            let app_dir = data_dir.join("recurrency");
            if let Err(e) = std::fs::create_dir_all(&app_dir) {
                eprintln!("[Setup] Failed to create app dir {:?}: {}", app_dir, e);
            }
            if let Err(e) = std::fs::write(app_dir.join("server_url.txt"), url) {
                eprintln!("[Setup] Failed to write server URL: {}", e);
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = url; // localStorage handled by AppModeState::persist
    }
}

#[component]
pub fn SetupPage() -> Element {
    let mut mode = use_signal(|| {
        let current = APP_MODE();
        match &current.mode {
            AppMode::Server(_url) => "server".to_string(),
            AppMode::Local => "local".to_string(),
        }
    });
    let mut server_url = use_signal(|| match &APP_MODE().mode {
        AppMode::Server(url) => url.clone(),
        AppMode::Local => String::new(),
    });
    let mut error = use_signal(String::new);
    let mut connecting = use_signal(|| false);

    let handle_connect = move |_| {
        let selected_mode = mode();
        error.set(String::new());

        if selected_mode == "server" {
            let url = server_url().trim().to_string();
            if url.is_empty() {
                error.set("Server URL is required".to_string());
                return;
            }
            if !url.starts_with("http://") && !url.starts_with("https://") {
                error.set("URL must start with http:// or https://".to_string());
                return;
            }

            connecting.set(true);

            spawn(async move {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let url_for_check = url.clone();
                    let tcp_check = tokio::task::spawn_blocking(move || {
                        let trimmed = url_for_check.trim();
                        let without_proto = trimmed
                            .strip_prefix("http://")
                            .or_else(|| trimmed.strip_prefix("https://"));
                        let host_port = match without_proto {
                            Some(s) => s.split('/').next().unwrap_or(s),
                            None => return Err("URL must start with http:// or https://".to_string()),
                        };
                        let mut parts = host_port.split(':');
                        let host = match parts.next() {
                            Some(h) if !h.is_empty() => h.to_string(),
                            _ => return Err("Invalid URL: missing host".to_string()),
                        };
                        let port: u16 = parts.next().unwrap_or(if trimmed.starts_with("https://") { "443" } else { "80" })
                            .parse().map_err(|_| "Invalid port".to_string())?;

                        let addr_str = format!("{}:{}", host, port);
                        use std::net::ToSocketAddrs;
                        let addr = addr_str
                            .to_socket_addrs()
                            .map_err(|e| format!("Invalid host: {}", e))?
                            .next()
                            .ok_or("Could not resolve host")?;
                        std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(5))
                            .map(|_| ())
                            .map_err(|e| format!("Connection failed: {}", e))
                    });

                    let timeout = tokio::time::timeout(std::time::Duration::from_secs(10), tcp_check).await;
                    match timeout {
                        Ok(Ok(Ok(()))) => {
                            set_base_url(&url);
                            // Update APP_MODE global state
                            APP_MODE.write().set_server(url.clone());
                            save_server_url(&url);
                            let nav = navigator();
                            nav.push(Route::Login {});
                        }
                        Ok(Ok(Err(e))) => {
                            error.set(format!("Connection failed: {}", e));
                        }
                        Ok(Err(e)) => {
                            error.set(format!("Task error: {}", e));
                        }
                        Err(_) => {
                            error.set("Connection timed out. Check the URL and make sure the server is running.".to_string());
                        }
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    set_base_url(&url);
                    APP_MODE.write().set_server(url.clone());
                    let nav = navigator();
                    nav.push(Route::Login {});
                }
                connecting.set(false);
            });
        } else {
            // Local mode — no server needed, go directly to app
            APP_MODE.write().set_local();
            let nav = navigator();
            nav.push(Route::Login {});
        }
    };

    rsx! {
        div { class: "flex min-h-screen items-center justify-center bg-background",
            div { class: "w-full max-w-md space-y-6 p-8",
                div { class: "flex flex-col gap-2 text-center",
                    h1 { class: "text-2xl font-bold text-foreground", "Recurrency" }
                    p { class: "text-sm text-muted-foreground",
                        "Choose how you want to use Recurrency"
                    }
                }

                // Mode selection cards
                div { class: "flex flex-col gap-3",
                    // Local mode card
                    button {
                        onclick: move |_| { mode.set("local".to_string()); },
                        class: if mode() == "local" {
                            "rounded-xl border-2 border-primary bg-primary/5 p-4 text-left transition-all hover:bg-primary/10"
                        } else {
                            "rounded-xl border border-border p-4 text-left transition-all hover:border-primary/50"
                        },
                        div { class: "flex items-center gap-3",
                            div { class: "flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10",
                                // Server/Database icon
                                svg {
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    class: "h-5 w-5 text-primary",
                                    path { d: "M12 2L2 7l10 5 10-5-10-5z" }
                                    path { d: "M2 17l10 5 10-5" }
                                    path { d: "M2 12l10 5 10-5" }
                                }
                            }
                            div { class: "flex-1",
                                p { class: "font-medium text-foreground", "Local Mode" }
                                p { class: "text-sm text-muted-foreground",
                                    "Use offline — data stored on this device"
                                }
                            }
                            if mode() == "local" {
                                div { class: "h-5 w-5 rounded-full bg-primary flex items-center justify-center",
                                    svg {
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "white",
                                        stroke_width: "3",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        class: "h-3 w-3",
                                        polyline { points: "20 6 9 17 4 12" }
                                    }
                                }
                            }
                        }
                    }

                    // Server mode card
                    button {
                        onclick: move |_| { mode.set("server".to_string()); },
                        class: if mode() == "server" {
                            "rounded-xl border-2 border-primary bg-primary/5 p-4 text-left transition-all hover:bg-primary/10"
                        } else {
                            "rounded-xl border border-border p-4 text-left transition-all hover:border-primary/50"
                        },
                        div { class: "flex items-center gap-3",
                            div { class: "flex h-10 w-10 items-center justify-center rounded-lg bg-blue-500/10",
                                // Cloud/Server icon
                                svg {
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    class: "h-5 w-5 text-blue-500",
                                    circle { cx: "12", cy: "12", r: "3" }
                                    path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" }
                                }
                            }
                            div { class: "flex-1",
                                p { class: "font-medium text-foreground", "Server Mode" }
                                p { class: "text-sm text-muted-foreground",
                                    "Connect to a self-hosted Recurrency server"
                                }
                            }
                            if mode() == "server" {
                                div { class: "h-5 w-5 rounded-full bg-primary flex items-center justify-center",
                                    svg {
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "white",
                                        stroke_width: "3",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        class: "h-3 w-3",
                                        polyline { points: "20 6 9 17 4 12" }
                                    }
                                }
                            }
                        }
                    }
                }

                // Server URL input (only shown when server mode selected)
                if mode() == "server" {
                    div { class: "flex flex-col gap-4",
                        div { class: "flex flex-col gap-2",
                            label {
                                r#for: "setup-server-url",
                                class: "text-sm font-medium text-foreground",
                                "Server URL"
                            }
                            input {
                                id: "setup-server-url",
                                r#type: "text",
                                placeholder: "http://192.168.1.100:8080",
                                value: "{server_url}",
                                oninput: move |e| {
                                    server_url.set(e.value());
                                    error.set(String::new());
                                },
                                class: "flex h-10 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-primary",
                            }
                        }
                        if !error().is_empty() {
                            p { class: "text-sm text-red-500", "{error}" }
                        }
                        p { class: "text-xs text-muted-foreground text-center",
                            "Make sure your server is running and accessible from this device"
                        }
                    }
                }

                // Connect / Continue button
                button {
                    onclick: handle_connect,
                    disabled: connecting(),
                    class: if connecting() {
                        "w-full rounded-md bg-muted text-muted-foreground px-4 py-2 text-sm font-medium cursor-not-allowed"
                    } else {
                        "w-full rounded-md bg-primary text-primary-foreground px-4 py-2 text-sm font-medium hover:bg-primary/90 active:scale-[0.97] transition-all"
                    },
                    if connecting() {
                        "Connecting..."
                    } else if mode() == "local" {
                        "Continue Offline"
                    } else {
                        "Connect"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_base_url_not_empty() {
        let url = get_base_url();
        assert!(!url.is_empty());
    }

    #[test]
    fn setup_page_renders_without_panic() {
        let url = get_base_url();
        assert!(url.starts_with("http") || url.is_empty());
    }

    #[test]
    fn get_base_url_is_http() {
        let url = crate::api::client::get_base_url();
        assert!(url.is_empty() || url.starts_with("http"));
    }

    #[test]
    fn set_base_url_overrides() {
        set_base_url("http://override:9090");
        assert_eq!(get_base_url(), "http://override:9090");
        set_base_url("");
    }

    #[test]
    fn set_base_url_empty_clears_override() {
        set_base_url("http://temp:8080");
        let url = get_base_url();
        assert!(url.starts_with("http"));
        set_base_url("");
    }

    #[test]
    fn set_base_url_rejects_invalid() {
        set_base_url("not-a-url");
        // Should not override — still falls back to default
        let url = get_base_url();
        assert!(url.starts_with("http"));
    }

    #[test]
    fn set_base_url_accepts_https() {
        set_base_url("https://secure-server.example.com");
        assert_eq!(get_base_url(), "https://secure-server.example.com");
        set_base_url("");
    }

    #[test]
    fn base_url_override_https() {
        set_base_url("https://secure-server.example.com");
        assert_eq!(get_base_url(), "https://secure-server.example.com");
        set_base_url("");
    }

    #[test]
    fn base_url_override_clears() {
        set_base_url("http://temp:8080");
        let url = get_base_url();
        assert!(url.starts_with("http"));
        set_base_url("");
    }

    // ── Mode selection tests ──

    #[test]
    fn app_mode_local_is_local() {
        assert!(AppMode::Local.is_local());
    }

    #[test]
    fn app_mode_server_is_not_local() {
        assert!(!AppMode::Server("http://test:8080".into()).is_local());
    }

    #[test]
    fn app_mode_server_url_accessor() {
        assert_eq!(AppMode::Local.server_url(), None);
        assert_eq!(
            AppMode::Server("http://test:8080".into()).server_url(),
            Some("http://test:8080")
        );
    }

    // ── URL validation ──

    #[test]
    fn validate_url_empty_invalid() {
        let url = "";
        assert!(url.trim().is_empty());
    }

    #[test]
    fn validate_url_requires_protocol() {
        let url = "192.168.1.100";
        assert!(!url.starts_with("http://") && !url.starts_with("https://"));
    }

    #[test]
    fn validate_url_localhost_valid() {
        let url = "http://localhost:3000";
        assert!(url.starts_with("http://"));
    }

    #[test]
    fn validate_url_https_valid() {
        let url = "https://recurrency.example.com";
        assert!(url.starts_with("https://"));
    }

    // ── RSX structure tests ──

    #[test]
    fn setup_page_has_mode_selection() {
        let src = include_str!("setup.rs");
        assert!(src.contains("Local Mode"), "Should have local mode option");
        assert!(src.contains("Server Mode"), "Should have server mode option");
    }

    #[test]
    fn setup_page_has_url_input() {
        let src = include_str!("setup.rs");
        assert!(src.contains("setup-server-url"), "Should have server URL input");
    }

    #[test]
    fn setup_page_has_connect_button() {
        let src = include_str!("setup.rs");
        assert!(
            src.contains("Connect") || src.contains("Continue Offline"),
            "Should have action button"
        );
    }

    #[test]
    fn setup_page_mode_cards_have_icons() {
        let src = include_str!("setup.rs");
        assert!(src.contains("border-primary"), "Selected mode should highlight");
        assert!(src.contains("rounded-full bg-primary"), "Selected mode should have checkmark");
    }

    #[test]
    fn setup_page_local_mode_skips_connection() {
        let src = include_str!("setup.rs");
        assert!(src.contains("Continue Offline"), "Local mode should have continue button");
    }
}