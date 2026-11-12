use dioxus::prelude::*;
use serde::Deserialize;

const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/badwatt/recurrency/releases/latest";
const DISMISSED_FILE: &str = "update_dismissed.txt";
const DISMISSED_KEY: &str = "recurrency-update-dismissed";

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct Release {
    tag_name: String,
    html_url: String,
}

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
    std::fs::read_to_string(path).ok().map(|value| value.trim().to_string())
}

#[cfg(target_arch = "wasm32")]
fn store_dismissed(version: &str) {
    if let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
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

#[cfg(target_arch = "wasm32")]
async fn fetch_latest() -> Option<Release> {
    let response = crate::api::client::get_client()
        .get(LATEST_RELEASE_URL)
        .header("User-Agent", "recurrency")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    response.json::<Release>().await.ok()
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_latest() -> Option<Release> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let result = {
            let req = ureq::get(LATEST_RELEASE_URL)
                .header("User-Agent", "recurrency")
                .header("Accept", "application/vnd.github+json")
                .config()
                .timeout_global(Some(std::time::Duration::from_secs(10)))
                .http_status_as_error(false)
                .build();
            req.call().ok().and_then(|mut response| {
                if !response.status().is_success() {
                    return None;
                }
                response.body_mut().read_json::<Release>().ok()
            })
        };
        let _ = tx.send(result);
    });
    tokio::time::timeout(std::time::Duration::from_secs(12), rx)
        .await
        .ok()?
        .ok()?
}

#[component]
pub fn UpdateBanner() -> Element {
    let mut release = use_signal(|| None::<Release>);
    let mut dismissed = use_signal(|| false);

    use_future(move || async move {
        if !crate::state::preferences::PREFERENCES().show_updates() {
            return;
        }
        let dismissed_version = read_dismissed();
        if let Some(info) = fetch_latest().await
            && is_newer_version(current_version(), &info.tag_name)
        {
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
            a {
                href,
                target: "_blank",
                rel: "noopener noreferrer",
                class: "inline-flex h-8 shrink-0 items-center rounded-md border border-primary/30 px-3 text-xs font-medium text-primary hover:bg-primary/10",
                "View release"
            }
            button {
                r#type: "button",
                aria_label: "Dismiss update notice",
                class: "inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-muted hover:text-foreground",
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
    fn release_parses_tag_and_url() {
        let release: Release = serde_json::from_str(
            r#"{"tag_name":"v0.31.0","html_url":"https://example.test/release"}"#,
        )
        .unwrap();
        assert_eq!(release.tag_name, "v0.31.0");
        assert_eq!(release.html_url, "https://example.test/release");
    }
}
