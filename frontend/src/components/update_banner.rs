use crate::api::update::{self, Release};
use crate::components::button::{Button, ButtonSize, ButtonVariant, IconButton};
use crate::components::toast::TOAST;
use dioxus::prelude::*;

const DISMISSED_FILE: &str = "update_dismissed.txt";
const DISMISSED_KEY: &str = "recurrency-update-dismissed";

fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn version_parts(value: &str) -> Vec<u64> {
    value
        .trim()
        .trim_start_matches('v')
        .split(['.', '-'])
        .take(3)
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect()
}

/// True when `latest` is a stable semantic version newer than `current`.
pub fn is_newer_version(current: &str, latest: &str) -> bool {
    let current = version_parts(current);
    let latest = version_parts(latest);
    latest > current
}

#[cfg(target_arch = "wasm32")]
fn read_dismissed() -> Option<String> {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(DISMISSED_KEY).ok().flatten())
}

#[cfg(not(target_arch = "wasm32"))]
fn read_dismissed() -> Option<String> {
    let path = crate::state::auth::get_app_data_dir()?
        .join("recurrency")
        .join(DISMISSED_FILE);
    std::fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
}

#[cfg(target_arch = "wasm32")]
fn store_dismissed(version: &str) {
    if let Some(storage) =
        web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    {
        let _ = storage.set_item(DISMISSED_KEY, version);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn store_dismissed(version: &str) {
    if let Some(dir) = crate::state::auth::get_app_data_dir() {
        let dir = dir.join("recurrency");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join(DISMISSED_FILE), version);
    }
}

#[component]
pub fn UpdateBanner() -> Element {
    let mut release = use_signal(|| None::<Release>);
    let mut dismissed = use_signal(|| false);
    let mut updating = use_signal(|| false);

    use_future(move || async move {
        if !crate::state::preferences::PREFERENCES().show_updates() {
            return;
        }
        let dismissed_version = read_dismissed();
        if let Some(info) = update::latest().await
            && is_newer_version(current_version(), &info.tag_name)
        {
            let ready = if cfg!(target_arch = "wasm32") {
                update::web_image_available(&info.tag_name).await
            } else {
                info.has_platform_asset()
            };
            if !ready {
                return;
            }
            if dismissed_version.as_deref() == Some(info.tag_name.as_str()) {
                dismissed.set(true);
            } else {
                release.set(Some(info));
            }
        }
    });

    if !crate::state::preferences::PREFERENCES().show_updates() || dismissed() {
        return rsx! {};
    }
    let Some(info) = release() else {
        return rsx! {};
    };
    let version = info.tag_name.trim_start_matches('v').to_string();
    let href = info.html_url.clone();
    let native = info
        .native_asset()
        .map(|asset| (asset.browser_download_url.clone(), asset.name.clone()));
    let update_asset = native.clone();
    let update = move |_| {
        let Some((url, filename)) = update_asset.clone() else {
            return;
        };
        updating.set(true);
        spawn(async move {
            match update::install(&url, &filename).await {
                Ok(()) => TOAST
                    .write()
                    .success("Downloading update. Follow the installer."),
                Err(error) => TOAST
                    .write()
                    .error(format!("Could not start the update: {error}")),
            }
            updating.set(false);
        });
    };

    rsx! {
        div {
            class: "mx-4 mt-4 flex items-center gap-3 rounded-xl border border-primary/20 bg-gradient-to-r from-primary/10 via-card/95 to-card/95 p-3 shadow-sm backdrop-blur",
            role: "status",
            div { class: "flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-primary/15 text-primary",
                svg {
                    view_box: "0 0 24 24",
                    width: "18",
                    height: "18",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
                    polyline { points: "7 10 12 15 17 10" }
                    line { x1: "12", y1: "15", x2: "12", y2: "3" }
                }
            }
            div { class: "min-w-0 flex-1",
                p { class: "text-sm font-semibold", "Recurrency {version} is available" }
                p { class: "text-xs text-muted-foreground",
                    "You are on {current_version()}. The update is ready when you are."
                }
            }
            if !cfg!(target_arch = "wasm32") && native.is_some() {
                Button {
                    size: ButtonSize::Sm,
                    class: "shrink-0",
                    disabled: updating(),
                    onclick: update,
                    if updating() { "Starting…" } else { "Update" }
                }
            }
            if cfg!(target_arch = "wasm32") {
                a {
                    href,
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "inline-flex h-8 shrink-0 items-center rounded-md border border-primary/30 px-3 text-xs font-medium text-primary hover:bg-primary/10",
                    "View release"
                }
            }
            IconButton {
                label: "Dismiss update notice".to_string(),
                variant: ButtonVariant::Ghost,
                size: ButtonSize::IconSm,
                class: "shrink-0 text-muted-foreground".to_string(),
                onclick: move |_| {
                    store_dismissed(&info.tag_name);
                    dismissed.set(true);
                },
                svg {
                    view_box: "0 0 24 24",
                    width: "16",
                    height: "16",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    line { x1: "18", y1: "6", x2: "6", y2: "18" }
                    line { x1: "6", y1: "6", x2: "18", y2: "18" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_patch_is_detected() {
        assert!(is_newer_version("0.30.0", "v0.30.1"));
        assert!(is_newer_version("0.29.0", "0.30.0"));
        assert!(!is_newer_version("0.30.0", "v0.30.0"));
        assert!(!is_newer_version("0.30.0", "v0.29.9"));
    }

    #[test]
    fn version_comparison_handles_partial_and_invalid_values() {
        assert!(is_newer_version("0.30", "0.30.1"));
        assert!(!is_newer_version("0.30.1", "0.30"));
        assert!(!is_newer_version("", ""));
        assert!(is_newer_version("not-a-version", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "1.0.0-beta"));
    }

    #[test]
    fn release_parses_tag_and_url() {
        let release: Release = serde_json::from_str(
            r#"{"tag_name":"v0.31.0","html_url":"https://example.test/release"}"#,
        )
        .unwrap();
        assert_eq!(release.tag_name, "v0.31.0");
        assert_eq!(release.html_url, "https://example.test/release");
    }
}
