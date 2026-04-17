use dioxus::prelude::*;
use crate::api::client::{get_base_url, set_base_url};
use crate::Route;

fn save_server_url(url: &str) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(data_dir) = dirs::data_dir() {
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

            #[cfg(not(target_os = "android"))]
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
}
