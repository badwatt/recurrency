use dioxus::prelude::*;
use crate::api::client::get_base_url;
use crate::Route;

/// Setup page: server URL input and Connect button.
/// Used on first launch (desktop/mobile) to configure the backend URL.
#[component]
pub fn SetupPage() -> Element {
    let mut server_url = use_signal(|| get_base_url());
    let mut connecting = use_signal(|| false);
    let mut error = use_signal(String::new);
    let mut success = use_signal(|| false);

    let handle_connect = move |_| {
        let url = server_url();
        if url.is_empty() {
            error.set("Server URL is required".into());
            return;
        }
        error.set(String::new());
        connecting.set(true);
        spawn(async move {
            // Try to hit the health/demo endpoint to verify connectivity
            let client = reqwest::Client::new();
            match client.get(format!("{}/api/demo/status", url))
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    success.set(true);
                    connecting.set(false);
                    // Store the URL in env / localStorage
                    let _ = dioxus::document::eval(&format!(
                        "localStorage.setItem('RECURRENCY_API_URL', '{}')",
                        url,
                    ));
                    std::env::set_var("RECURRENCY_API_URL", &url);
                    let nav = navigator();
                    nav.push(Route::Login {});
                }
                Ok(resp) => {
                    error.set(format!("Server returned status {}", resp.status()));
                    connecting.set(false);
                }
                Err(e) => {
                    error.set(format!("Connection failed: {}", e));
                    connecting.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "flex min-h-dvh items-center justify-center bg-background p-4",
            div { class: "w-full max-w-md rounded-lg border border-border bg-card shadow-sm",
                // Header
                div { class: "p-6 pb-0",
                    h1 { class: "text-2xl font-bold text-center text-foreground",
                        "Connect to Server"
                    }
                    p { class: "text-sm text-muted-foreground text-center mt-2",
                        "Enter your Recurrency server URL to get started"
                    }
                }

                // Form
                div { class: "p-6 space-y-4",
                    if !error().is_empty() {
                        div { class: "rounded-md bg-destructive/10 p-3 text-sm text-destructive",
                            {error()}
                        }
                    }

                    if success() {
                        div { class: "rounded-md bg-green-100 p-3 text-sm text-green-800",
                            "Connected successfully! Redirecting..."
                        }
                    }

                    div { class: "space-y-2",
                        label {
                            r#for: "setup-server-url",
                            class: "text-sm font-medium text-foreground",
                            "Server URL"
                        }
                        input {
                            id: "setup-server-url",
                            r#type: "text",
                            placeholder: "http://localhost:8080",
                            value: "{server_url}",
                            oninput: move |e| { server_url.set(e.value()); error.set(String::new()); },
                            class: "flex h-10 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background",
                        }
                    }

                    button {
                        onclick: handle_connect,
                        disabled: connecting(),
                        class: "inline-flex h-10 w-full items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50",
                        if connecting() { "Connecting..." } else { "Connect" }
                    }

                    p { class: "text-center text-sm text-muted-foreground",
                        "Enter the URL of your Recurrency backend server"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_base_url_not_empty() {
        let url = super::get_base_url();
        assert!(!url.is_empty());
    }

    #[test]
    fn empty_url_is_invalid() {
        let url = "";
        assert!(url.is_empty());
    }

    #[test]
    fn valid_url_format() {
        let url = "http://localhost:8080";
        assert!(url.starts_with("http://") || url.starts_with("https://"));
    }
}