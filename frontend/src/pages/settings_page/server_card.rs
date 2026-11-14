use crate::components::server_url::ServerUrlEditor;
use crate::components::web_server_qr::WebServerQr;
use dioxus::prelude::*;

/// Server pairing: QR on web, editable URL on native/desktop.
#[component]
pub fn ServerCard() -> Element {
    if cfg!(target_arch = "wasm32") {
        return rsx! { WebServerQr {} };
    }

    rsx! {
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                h2 { class: "text-lg font-semibold", "Server connection" }
                p { class: "text-sm text-muted-foreground",
                    "Connect this device to your Recurrency server. Changing it signs you out."
                }
            }
            div { class: "p-6",
                ServerUrlEditor { logout_on_change: true }
            }
        }
    }
}
