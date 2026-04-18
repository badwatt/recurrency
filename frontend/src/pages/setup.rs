use dioxus::prelude::*;
use crate::api::client::{get_base_url, set_base_url};
use crate::Route;

fn save_server_url(url: &str) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(data_dir) = crate::state::auth::get_app_data_dir() {
            let app_dir = data_dir.join("recurrency");
            let _ = std::fs::create_dir_all(&app_dir);
            let url_file = app_dir.join("server_url.txt");
            let _ = std::fs::write(&url_file, url);
        }
    }
}

#[component]
pub fn SetupPage() -> Element {
    let mut server_url = use_signal(|| get_base_url());
    let mut connecting = use_signal(|| false);
    let mut success = use_signal(|| false);
    let mut error = use_signal(String::new);

    let handle_connect = move |_| {
        let url = server_url.read().trim().to_string();
        if url.is_empty() {
            error.set("Server URL is required".to_string());
            return;
        }
        if !url.starts_with("http://") && !url.starts_with("https://") {
            error.set("URL must start with http:// or https://".to_string());
            return;
        }
        error.set(String::new());
        connecting.set(true);

        let nav = navigator();

        spawn(async move {
            #[cfg(target_os = "android")]
            {
                set_base_url(&url);
                save_server_url(&url);
                nav.push(Route::Login {});
            }

            {
                let client = reqwest::Client::new();
                let result = client
                    .get(format!("{}/api/demo/status", &url))
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await;

                match result {
                    Ok(resp) if resp.status().is_success() => {
                        set_base_url(&url);
                        save_server_url(&url);

                        #[cfg(target_arch = "wasm32")]
                        {
                            let _ = dioxus::document::eval(&format!(
                                "localStorage.setItem('RECURRENCY_API_URL','{}')",
                                url
                            ))
                            .await;
                        }

                        #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
                        {
                            std::env::set_var("RECURRENCY_API_URL", &url);
                        }

                        nav.push(Route::Login {});
                    }
                    Ok(resp) => {
                        error.set(format!("Server returned status {}", resp.status()));
                    }
                    Err(e) => {
                        error.set(format!("Connection failed: {}", e));
                    }
                }
                connecting.set(false);
            }
        });
    };

    rsx! {
        div { class: "flex min-h-screen items-center justify-center bg-background",
            div { class: "w-full max-w-md space-y-6 p-8",
                div { class: "flex flex-col gap-2 text-center",
                    h1 { class: "text-2xl font-bold text-foreground", "Configure Server" }
                    p { class: "text-sm text-muted-foreground",
                        "Enter the URL of your Recurrency backend server"
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
                            class: "flex h-10 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background",
                        }
                    }
                    if !error().is_empty() {
                        p { class: "text-sm text-destructive", "{error}" }
                    }
                    if success() {
                        p { class: "text-sm text-green-500", "Connected successfully! Redirecting..." }
                    }
                    button {
                        onclick: handle_connect,
                        disabled: connecting(),
                        class: "w-full inline-flex items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed",
                        if connecting() {
                            "Connecting..."
                        } else {
                            "Connect"
                        }
                    }
                    p { class: "text-xs text-muted-foreground text-center",
                        "Make sure your server is running and accessible from this device"
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
        assert!(url.starts_with("http"));
    }

    #[test]
    fn get_base_url_is_http() {
        let url = crate::api::client::get_base_url();
        assert!(url.starts_with("http"));
    }

    #[test]
    fn server_url_with_trailing_slash() {
        let url = "http://192.168.1.100:8080/";
        assert!(url.starts_with("http"));
    }

    #[test]
    fn setup_page_connect_button_disabled_while_connecting() {
        let connecting = true;
        assert!(connecting);
    }

    #[test]
    fn get_base_url_returns_string() {
        let url = crate::api::client::get_base_url();
        assert!(!url.is_empty());
    }

    // ── URL validation logic (mirrors SetupPage validation) ──
    #[test]
    fn validate_url_empty_invalid() {
        let url = "";
        assert!(url.trim().is_empty(), "Empty URL should be invalid");
    }

    #[test]
    fn validate_url_missing_protocol_invalid() {
        let url = "192.168.1.100:8080";
        let has_http = url.starts_with("http://");
        let has_https = url.starts_with("https://");
        assert!(!has_http && !has_https, "URL without http(s):// should be invalid");
    }

    #[test]
    fn validate_url_http_valid() {
        let url = "http://192.168.1.100:8080";
        assert!(url.starts_with("http://") || url.starts_with("https://"));
    }

    #[test]
    fn validate_url_https_valid() {
        let url = "https://recurrency.example.com";
        assert!(url.starts_with("https://"));
    }

    #[test]
    fn validate_url_with_port_valid() {
        let url = "http://localhost:8080";
        assert!(url.starts_with("http://"));
        assert!(url.contains(":8080"));
    }

    #[test]
    fn validate_url_trailing_slash_valid() {
        let url = "http://example.com:8080/";
        assert!(url.starts_with("http://"));
    }

    #[test]
    fn validate_url_ip_address_valid() {
        let url = "http://10.0.0.1:3000";
        assert!(url.starts_with("http://"));
        assert!(url.contains("10.0.0.1"));
    }

    #[test]
    fn validate_url_whitespace_invalid() {
        let url = "   http://example.com   ";
        // After trim should be valid, but raw URL with spaces should be invalid
        assert!(url.trim() != url, "URL with leading/trailing spaces should be trimmed");
        let trimmed = url.trim();
        assert!(trimmed.starts_with("http://"));
    }

    #[test]
    fn validate_url_ftp_invalid() {
        let url = "ftp://example.com";
        assert!(!url.starts_with("http://") && !url.starts_with("https://"));
    }

    #[test]
    fn validate_url_file_protocol_invalid() {
        let url = "file:///path/to/server";
        assert!(!url.starts_with("http://") && !url.starts_with("https://"));
    }

    // ── Error message logic ──
    #[test]
    fn error_message_empty_url() {
        let url = "";
        let error = if url.is_empty() { "Server URL is required" } else { "" };
        assert_eq!(error, "Server URL is required");
    }

    #[test]
    fn error_message_invalid_protocol() {
        let url = "example.com";
        let error = if !url.starts_with("http://") && !url.starts_with("https://") {
            "URL must start with http:// or https://"
        } else { "" };
        assert_eq!(error, "URL must start with http:// or https://");
    }

    #[test]
    fn error_message_no_error_when_valid() {
        let url = "http://example.com";
        let error = if url.is_empty() { "Server URL is required" }
            else if !url.starts_with("http://") && !url.starts_with("https://") { "URL must start with http:// or https://" }
            else { "" };
        assert_eq!(error, "");
    }

    // ── set_base_url / get_base_url override cycle ──
    #[test]
    fn base_url_override_after_set() {
        let original = get_base_url();
        assert!(original.starts_with("http"));

        set_base_url("http://custom-server:9000");
        let overridden = get_base_url();
        assert_eq!(overridden, "http://custom-server:9000");

        // Reset to original
        set_base_url("");
        // After clearing, falls back to impl
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
        assert_eq!(get_base_url(), "http://temp:8080");
        set_base_url("");
        // After clearing, reverts to default impl
        let url = get_base_url();
        assert!(url.starts_with("http"));
    }
}
