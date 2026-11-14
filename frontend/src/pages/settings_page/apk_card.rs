use crate::pages::settings_page::APP_VERSION;
use dioxus::prelude::*;

const LATEST_APK_URL: &str =
    "https://github.com/badwatt/recurrency/releases/latest/download/recurrency-latest-aarch64.apk";

#[component]
pub fn ApkCard() -> Element {
    // The QR is meant to be scanned from a phone, not viewed on that phone.
    if cfg!(target_os = "android") {
        return rsx! {};
    }

    let qr = crate::utils::qr::svg(LATEST_APK_URL).ok();

    rsx! {
        div { class: "rounded-xl border border-border/50 bg-card/80 p-6",
            div { class: "flex items-start gap-4",
                div { class: "min-w-0 flex-1",
                    h2 { class: "text-lg font-semibold", "Android app" }
                    p { class: "mt-1 text-sm text-muted-foreground",
                        "Scan the code with an Android device to download the latest APK."
                    }
                }
                span { class: "rounded-full border border-border/60 bg-background px-2.5 py-1 text-[11px] font-medium text-muted-foreground",
                    "APK"
                }
            }
            div { class: "mt-5 flex flex-col gap-5 sm:flex-row sm:items-center",
                div {
                    class: "flex h-44 w-44 shrink-0 items-center justify-center self-center rounded-xl bg-white p-2 sm:self-start",
                    if let Some(svg) = qr {
                        div { class: "h-full w-full [&>svg]:h-full [&>svg]:w-full", dangerous_inner_html: svg }
                    } else {
                        p { class: "text-center text-xs text-muted-foreground", "QR unavailable" }
                    }
                }
                div { class: "min-w-0 flex-1 space-y-3",
                    p { class: "text-sm font-medium", "recurrency-latest-aarch64.apk" }
                    p { class: "text-xs text-muted-foreground",
                        "This QR always points to the latest release. Installed version: v{APP_VERSION}."
                    }
                }
            }
        }
    }
}
