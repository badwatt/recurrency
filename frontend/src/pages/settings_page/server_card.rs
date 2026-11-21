use crate::components::server_url::ServerUrlEditor;
use crate::components::web_server_qr::WebServerQr;
use dioxus::prelude::*;

/// Server pairing: QR on web, editable URL on native/desktop.
use crate::components::card::SectionCard;
#[component]
pub fn ServerCard() -> Element {
    if cfg!(target_arch = "wasm32") {
        return rsx! { WebServerQr {} };
    }

    rsx! {
        SectionCard {
            title: "Server connection",
            description: "Connect this device to your Recurrency server. Changing it signs you out.",
            content_class: "p-0",
            div { class: "p-6",
                ServerUrlEditor { logout_on_change: true }
            }
        }
    }
}
