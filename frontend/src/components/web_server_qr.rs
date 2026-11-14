use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
fn current_server_url() -> String {
    web_sys::window()
        .and_then(|window| window.location().origin().ok())
        .unwrap_or_else(crate::api::client::get_base_url)
}

#[cfg(not(target_arch = "wasm32"))]
fn current_server_url() -> String {
    String::new()
}

#[component]
pub fn WebServerQr() -> Element {
    if !cfg!(target_arch = "wasm32") {
        return rsx! {};
    }
    let url = current_server_url();
    let payload = format!(r#"{{"type":"recurrency-server","url":"{url}"}}"#);
    let qr = crate::utils::qr::svg(&payload).ok();
    rsx! {
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                h2 { class: "text-lg font-semibold", "Connect a device" }
                p { class: "text-sm text-muted-foreground",
                    "Scan this code from the Android app to use this server."
                }
            }
            div { class: "flex flex-col items-center gap-4 p-6 sm:flex-row",
                div { class: "flex h-40 w-40 shrink-0 items-center justify-center rounded-xl bg-white p-2",
                    if let Some(svg) = qr {
                        div { class: "h-full w-full [&>svg]:h-full [&>svg]:w-full", dangerous_inner_html: svg }
                    } else {
                        p { class: "text-center text-xs text-muted-foreground", "QR unavailable" }
                    }
                }
                div { class: "min-w-0 text-center sm:text-left",
                    p { class: "text-sm font-medium", "Server" }
                    p { class: "mt-1 break-all font-mono text-xs text-muted-foreground", {url} }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn web_qr_is_web_only() {
        let source = include_str!("web_server_qr.rs");
        assert!(source.contains("target_arch = \"wasm32\""));
        assert!(source.contains("recurrency-server"));
    }
}
