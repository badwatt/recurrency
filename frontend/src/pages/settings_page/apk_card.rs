use crate::api::update::{self, Release};
use crate::components::update_banner::is_newer_version;
use crate::pages::settings_page::APP_VERSION;
use dioxus::prelude::*;

const LATEST_APK_URL: &str =
    "https://github.com/badwatt/recurrency/releases/latest/download/recurrency-latest-aarch64.apk";

#[component]
pub fn ApkCard() -> Element {
    let mut latest = use_signal(|| None::<Release>);
    let mut status = use_signal(String::new);
    let mut installing = use_signal(|| false);
    let qr = crate::utils::qr::svg(LATEST_APK_URL).ok();

    use_future(move || async move {
        latest.set(update::latest().await);
    });

    let release = latest();
    let newer = release
        .as_ref()
        .is_some_and(|release| is_newer_version(APP_VERSION, &release.tag_name));
    let install = move |_| {
        let Some(release) = latest() else {
            return;
        };
        let Some(url) = release.apk_url().map(str::to_string) else {
            status.set("No Android APK is attached to the latest release.".to_string());
            return;
        };
        installing.set(true);
        status.set("Starting the update…".to_string());
        spawn(async move {
            match update::install_apk(&url).await {
                Ok(()) => status.set("Downloading update. Follow the Android installer.".to_string()),
                Err(message) => status.set(message),
            }
            installing.set(false);
        });
    };

    rsx! {
        div { class: "rounded-xl border border-border/50 bg-card/80 p-6",
            div { class: "flex items-start gap-4",
                div { class: "min-w-0 flex-1",
                    h2 { class: "text-lg font-semibold", "Android app" }
                    p { class: "mt-1 text-sm text-muted-foreground",
                        "Scan the code to download the latest APK, or update from here."
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
                        "Install it on an Android device. The APK is always the latest release."
                    }
                    a {
                        href: LATEST_APK_URL,
                        class: "inline-flex h-9 items-center rounded-md border border-border bg-background px-3 text-sm font-medium hover:bg-accent",
                        "Download APK"
                    }
                    if cfg!(target_os = "android") && newer {
                        button {
                            r#type: "button",
                            disabled: installing(),
                            onclick: install,
                            class: "inline-flex h-9 items-center rounded-md bg-primary px-3 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50",
                            if installing() { "Starting…" } else { "Update Recurrency" }
                        }
                    } else if newer {
                        a {
                            href: release.map(|release| release.html_url).unwrap_or_default(),
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "inline-flex h-9 items-center rounded-md bg-primary px-3 text-sm font-medium text-primary-foreground hover:bg-primary/90",
                            "View update"
                        }
                    }
                    if !status().is_empty() {
                        p { class: "text-xs text-muted-foreground", {status()} }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn apk_card_has_update_and_qr_paths() {
        let source = include_str!("apk_card.rs");
        assert!(source.contains("LATEST_APK_URL"));
        assert!(source.contains("install_apk"));
        assert!(source.contains("dangerous_inner_html"));
    }
}
