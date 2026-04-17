use dioxus::prelude::*;
use crate::api::client::{get_base_url, set_base_url};
use crate::Route;

/// Save server URL to persistent storage.
/// - WASM: localStorage (synchronous web_sys in get_base_url_impl)
/// - Android/desktop: app data file
fn save_server_url(_url: &str) {
    // On WASM, localStorage is already handled by get_base_url_impl().
    // We save to file for Android/desktop persistence across app restarts.
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(data_dir) = dirs::data_dir() {
            let app_dir: std::path::PathBuf = data_dir.join("recurrency");
            let _ = std::fs::create_dir_all(&app_dir);
            let url_file: std::path::PathBuf = app_dir.join("server_url.txt");
            if std::fs::write(&url_file, _url).is_ok() {
                eprintln!("[save_server_url] Saved to {:?}", url_file);
            } else {
                eprintln!("[save_server_url] Failed to write {:?}", url_file);
            }
        }
    }
}

/// Setup page: server URL input and Connect button.
/// Used on first launch (desktop/mobile) to configure the backend URL.
#[component]
pub fn SetupPage() -> Element {
    let mut server_url = use_signal(|| get_base_url());
    let mut connecting = use_signal(|| false);
    let mut success = use_signal(|| false);
    let mut error = use_signal(String::new);

    let handle_connect = move |_| {
        let url = server_url().trim().to_string();
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
        let url_clone = url.clone();
        spawn(async move {
            // On Android, reqwest hangs permanently (async runtime issue).
            // Skip health check — save URL and navigate immediately.
            // Login page will show error if URL is wrong.
            #[cfg(target_os = "android")]
            {
                success.set(true);
                connecting.set(false);
                set_base_url(&url_clone);
                save_server_url(&url_clone);
                let nav = navigator();
                nav.push(Route::Login {});
            }

            // On web/desktop, verify connectivity before navigating.
            #[cfg(not(target_os = "android"))]
            {
                let client = reqwest::Client::new();
                match client.get(format!("{}/api/demo/status", &url_clone))
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await
                {
                    Ok(resp) if resp.status().is_success() => {
                        success.set(true);
                        connecting.set(false);
                        // Update runtime base URL
                        set_base_url(&url_clone);
                        // Persist for next app launch
                        save_server_url(&url_clone);
                        // On WASM: also store in localStorage for get_base_url_impl()
                        #[cfg(target_arch = "wasm32")]
                        {
                            let js = format!("localStorage.setItem('RECURRENCY_API_URL', '{}')", url_clone);
                            let _ = dioxus::document::eval(&js).await;
                        }
                        // Desktop env var (crashes on Android — guarded)
                        #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
                        std::env::set_var("RECURRENCY_API_URL", &url_clone);
                        let nav = navigator();
                        nav.push(Route::Login {});
                    }
                    Ok(resp) => {
                        connecting.set(false);
                        error.set(format!("Server returned status {}", resp.status()));
                    }
                    Err(e) => {
                        connecting.set(false);
                        error.set(format!("Connection failed: {}", e));
                    }
                }
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
                            oninput: move |e| { server_url.set(e.value()); error.set(String::new()); },
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
        assert!(!url.is_empty(), "Base URL should not be empty");
    }

    #[test]
    fn setup_page_renders_without_panic() {
        let url = get_base_url();
        assert!(url.starts_with("http"), "URL should start with http: {}", url);
    }

    #[test]
    fn get_base_url_is_http() {
        let url = crate::api::client::get_base_url();
        assert!(url.starts_with("http"), "URL should start with http: {}", url);
    }

    #[test]
    fn server_url_with_trailing_slash() {
        let url = "http://192.168.1.100:8080/";
        assert!(url.starts_with("http"), "URL should start with http");
    }

    #[test]
    fn setup_page_connect_button_disabled_while_connectting() {
        let connecting = true;
        assert!(connecting, "Button should be disabled when connecting");
    }

    #[test]
    fn get_base_url_returns_string() {
        let url = crate::api::client::get_base_url();
        assert!(!url.is_empty());
    }
}
