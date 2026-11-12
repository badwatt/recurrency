use crate::components::page_header::PageHeader;
use crate::pages::settings_page::APP_VERSION;
use dioxus::prelude::*;
use serde::Deserialize;

const REPO_URL: &str = "https://github.com/badwatt/recurrency";
const RELEASES_URL: &str = "https://api.github.com/repos/badwatt/recurrency/releases/latest";

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct ReleaseInfo {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    body: String,
}

#[cfg(target_arch = "wasm32")]
async fn fetch_release(url: &str) -> Option<ReleaseInfo> {
    crate::api::client::get_client()
        .get(url)
        .header("User-Agent", "recurrency")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json::<ReleaseInfo>()
        .await
        .ok()
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_release(url: &str) -> Option<ReleaseInfo> {
    let url = url.to_string();
    let (tx, rx) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let result = {
            let req = ureq::get(&url)
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
                response.body_mut().read_json::<ReleaseInfo>().ok()
            })
        };
        let _ = tx.send(result);
    });
    tokio::time::timeout(std::time::Duration::from_secs(12), rx)
        .await
        .ok()?
        .ok()?
}

async fn fetch_latest() -> Option<ReleaseInfo> {
    fetch_release(RELEASES_URL).await
}

async fn fetch_current() -> Option<ReleaseInfo> {
    fetch_release(&format!("{REPO_URL}/releases/tags/v{APP_VERSION}")).await
}

#[component]
pub fn AboutPage() -> Element {
    let mut latest = use_signal(|| None::<ReleaseInfo>);
    let mut current = use_signal(|| None::<ReleaseInfo>);
    let mut show_notes = use_signal(|| false);
    let mut show_latest = use_signal(|| false);

    use_future(move || async move {
        let latest_release = fetch_latest().await;
        let current_release = fetch_current().await;
        latest.set(latest_release.clone());
        current.set(current_release.or(latest_release));
    });

    let latest_release = latest();
    let current_release = current();
    let latest_version = latest_release
        .as_ref()
        .map(|release| release.tag_name.trim_start_matches('v').to_string())
        .unwrap_or_else(|| APP_VERSION.to_string());
    let displayed = if show_latest() { latest_release.as_ref() } else { current_release.as_ref() };
    let displayed_version = displayed
        .map(|release| release.tag_name.trim_start_matches('v').to_string())
        .unwrap_or_else(|| APP_VERSION.to_string());
    let notes = displayed
        .map(|release| release.body.clone())
        .unwrap_or_else(|| "Release notes are available on GitHub.".to_string());
    let release_href = displayed
        .map(|release| release.html_url.clone())
        .unwrap_or_else(|| format!("{REPO_URL}/releases"));

    rsx! {
        div { class: "flex flex-col gap-6 animate-in fade-in duration-300",
            PageHeader { title: "About", subtitle: "Recurrency, the people behind it, and what changed" }

            div { class: "relative overflow-hidden rounded-2xl border border-border/60 bg-gradient-to-br from-primary/15 via-card to-card p-6 md:p-8",
                div { class: "absolute -right-16 -top-16 h-48 w-48 rounded-full bg-primary/10 blur-3xl" }
                div { class: "relative flex flex-col gap-5 sm:flex-row sm:items-center sm:justify-between",
                    div {
                        p { class: "text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground", "Recurrency" }
                        h1 { class: "mt-2 text-3xl font-bold tracking-tight", "Every expense is a subscription." }
                        p { class: "mt-2 max-w-xl text-sm text-muted-foreground",
                            "A small, self-hosted tool for seeing what you keep paying for — and what you would subscribe to again today."
                        }
                    }
                    div { class: "rounded-xl border border-primary/20 bg-background/60 px-5 py-4 text-center",
                        p { class: "text-xs uppercase tracking-widest text-muted-foreground", "Installed version" }
                        p { class: "mt-1 font-mono text-2xl font-bold text-primary", "v{APP_VERSION}" }
                        if latest_version != APP_VERSION {
                            p { class: "mt-1 text-[11px] text-muted-foreground", "Latest: v{latest_version}" }
                        }
                    }
                }
            }

            div { class: "grid gap-4 md:grid-cols-2",
                div { class: "rounded-xl border border-border/50 bg-card p-5",
                    p { class: "text-sm font-semibold", "What's new" }
                    p { class: "mt-1 text-xs text-muted-foreground", "Release notes for v{displayed_version}" }
                    div { class: "mt-4 flex flex-wrap gap-2",
                        button {
                            r#type: "button",
                            class: "inline-flex h-9 items-center rounded-md border border-border px-3 text-sm font-medium hover:bg-accent",
                            onclick: move |_| show_notes.toggle(),
                            if show_notes() { "Hide notes" } else { "Show changelog" }
                        }
                        if latest_version != APP_VERSION {
                            button {
                                r#type: "button",
                                class: "inline-flex h-9 items-center rounded-md border border-primary/30 px-3 text-sm font-medium text-primary hover:bg-primary/10",
                                onclick: move |_| {
                                    show_latest.toggle();
                                    show_notes.set(true);
                                },
                                if show_latest() { "Back to v{APP_VERSION}" } else { "Latest: v{latest_version}" }
                            }
                        }
                    }
                    if show_notes() {
                        pre { class: "mt-4 max-h-72 overflow-auto whitespace-pre-wrap rounded-lg bg-muted/50 p-3 font-sans text-xs leading-relaxed text-muted-foreground",
                            {notes}
                        }
                        a {
                            href: release_href,
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "mt-3 inline-flex text-xs font-medium text-primary hover:underline",
                            "Open release on GitHub"
                        }
                    }
                }

                div { class: "rounded-xl border border-border/50 bg-card p-5",
                    p { class: "text-sm font-semibold", "Enjoying Recurrency?" }
                    p { class: "mt-1 text-xs text-muted-foreground", "A star helps other people find a quieter way to track their subscriptions." }
                    a {
                        href: REPO_URL,
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "mt-4 inline-flex h-9 items-center gap-2 rounded-md bg-primary px-3 text-sm font-medium text-primary-foreground hover:bg-primary/90",
                        "★ Star on GitHub"
                    }
                    div { class: "mt-5 flex flex-wrap gap-3 border-t border-border/50 pt-4 text-xs",
                        a { href: format!("{REPO_URL}/issues"), target: "_blank", rel: "noopener noreferrer", class: "text-muted-foreground hover:text-foreground", "Report an issue" }
                        a { href: format!("{REPO_URL}#readme"), target: "_blank", rel: "noopener noreferrer", class: "text-muted-foreground hover:text-foreground", "Documentation" }
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
    fn release_parser_keeps_tag_body_and_url() {
        let release: ReleaseInfo = serde_json::from_str(
            r#"{"tag_name":"v0.36.2","html_url":"https://example.test/r","body":"Notes"}"#,
        )
        .unwrap();
        assert_eq!(release.tag_name, "v0.36.2");
        assert_eq!(release.body, "Notes");
    }
}
