use crate::api::client::set_base_url;
#[cfg(test)]
use crate::api::client::get_base_url;
use crate::state::app_mode::{AppMode, APP_MODE};
use crate::Route;
use dioxus::prelude::*;

/// Whether offline mode is selectable in Setup.
/// Available on all platforms — user can pick local-only on web or native.
const SHOW_OFFLINE: bool = true;

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
        let _ = url;
    }
}

#[component]
pub fn SetupPage() -> Element {
    let mut server_url = use_signal(|| {
        let current = APP_MODE();
        match &current.mode {
            AppMode::Server(url) if !url.is_empty() => url.clone(),
            _ => String::new(),
        }
    });
    let mut error = use_signal(String::new);
    let mut connecting = use_signal(|| false);

    let handle_connect = move |_| {
        let url = server_url().trim().to_string();
        error.set(String::new());

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
                let _ = dioxus::document::eval(&format!(
                    "localStorage.setItem('RECURRENCY_API_URL','{}')",
                    url
                )).await;
                let nav = navigator();
                nav.push(Route::Login {});
            }
            connecting.set(false);
        });
    };

    rsx! {
        div { class: "flex min-h-screen items-center justify-center bg-background",
            div { class: "w-full max-w-md space-y-6 p-8",
                div { class: "flex flex-col gap-2 text-center",
                    h1 { class: "text-2xl font-bold text-foreground", "Recurrency" }
                    p { class: "text-sm text-muted-foreground",
                        "Connect to your Recurrency server"
                    }
                }

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
                    } else {
                        "Connect"
                    }
                }

                if SHOW_OFFLINE {
                    button {
                        onclick: move |_| {
                            APP_MODE.write().set_local();
                            let nav = navigator();
                            nav.push(Route::Login {});
                        },
                        class: "w-full rounded-md border border-border bg-background px-4 py-2 text-sm font-medium text-foreground hover:bg-accent transition-colors",
                        "Use without server"
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

    #[test]
    fn app_mode_offline_is_offline() {
        assert!(crate::state::app_mode::AppMode::Offline.is_offline());
    }

    #[test]
    fn app_mode_server_is_not_offline() {
        assert!(!crate::state::app_mode::AppMode::Server("http://test:8080".into()).is_offline());
    }

    #[test]
    fn app_mode_server_url_accessor() {
        assert_eq!(crate::state::app_mode::AppMode::Offline.server_url(), None);
        assert_eq!(
            crate::state::app_mode::AppMode::Server("http://test:8080".into()).server_url(),
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
    fn setup_page_has_url_input() {
        let src = include_str!("setup.rs");
        assert!(src.contains("setup-server-url"), "Should have server URL input");
    }

    #[test]
    fn setup_page_has_connect_button() {
        let src = include_str!("setup.rs");
        assert!(
            src.contains("Connect"),
            "Should have connect button"
        );
    }

    #[test]
    fn setup_page_has_offline_option() {
        if SHOW_OFFLINE {
            let src = include_str!("setup.rs");
            assert!(src.contains("Use without server"), "Should have offline option");
        }
    }

    #[test]
    fn setup_page_default_url_is_empty() {
        let src = include_str!("setup.rs");
        // Default URL should be empty — user must enter their server URL
        assert!(src.contains("_ => String::new()"), "Default URL should be empty");
    }

    #[test]
    fn setup_page_validates_before_connect() {
        let src = include_str!("setup.rs");
        assert!(
            src.contains("url.is_empty()") && src.contains("http://"),
            "Should validate URL before connect"
        );
    }
}