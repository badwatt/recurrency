//! Dismissible banner for a newer GitHub release.

use nsh_base::{button, theme};
use nsh_core::prelude::*;
use nsh_core::storage::Storage;

fn preferred_asset<'a>(release: &'a crate::api::Release) -> Option<&'a crate::api::ReleaseAsset> {
    let suffix = if cfg!(target_os = "android") {
        [".apk"].as_slice()
    } else if cfg!(target_os = "windows") {
        [".exe", ".msi"].as_slice()
    } else if cfg!(target_os = "macos") {
        [".dmg"].as_slice()
    } else {
        [".appimage", ".deb", ".tar.gz"].as_slice()
    };
    release.assets.iter().find(|asset| {
        let name = asset.name.to_ascii_lowercase();
        suffix.iter().any(|suffix| name.ends_with(suffix))
    })
}

const SHOW_UPDATES_KEY: &str = "recurrency.show-updates";
const DISMISSED_KEY: &str = "recurrency.update-dismissed";

pub fn show_updates() -> bool {
    Storage::get_item(SHOW_UPDATES_KEY)
        .as_deref()
        .map(str::trim)
        .map_or(true, |value| value != "false")
}

pub fn set_show_updates(value: bool) {
    let _ = Storage::set_item(SHOW_UPDATES_KEY, if value { "true" } else { "false" });
}

fn dismissed_version(version: &str) -> bool {
    Storage::get_item(DISMISSED_KEY).as_deref() == Some(version)
}

fn dismiss_version(version: &str) {
    let _ = Storage::set_item(DISMISSED_KEY, version);
}

fn newer(current: &str, latest: &str) -> bool {
    let parse = |value: &str| {
        value
            .trim()
            .trim_start_matches('v')
            .split(['.', '-'])
            .take(3)
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    parse(latest) > parse(current)
}

pub fn update_banner() -> AnyElement {
    let release = resource(
        || Some(()),
        |(), _| async { Ok::<_, String>(crate::api::latest_release().await) },
    );
    let dismissed = signal(false);
    let updating = signal(false);
    let visible = move || {
        show_updates()
            && !dismissed.get()
            && release
                .get()
                .flatten()
                .is_some_and(|info: crate::api::Release| {
                    newer(env!("CARGO_PKG_VERSION"), &info.tag_name)
                        && !dismissed_version(&info.tag_name)
                })
    };
    when(visible, move || {
        let Some(info) = release.get().flatten() else {
            return view().h(px(0.0)).into_element();
        };
        let version = info.tag_name.trim_start_matches('v').to_string();
        let url = info.html_url.clone();
        let download = preferred_asset(&info).map(|asset| asset.browser_download_url.clone());
        view()
            .w_full()
            .items_center()
            .justify_center()
            .border_b(px(1.0))
            .border_color(move || theme().primary.opacity(0.3))
            .bg(move || theme().primary.opacity(0.1))
            .child(
                view()
                    .w_full()
                    .max_w(px(1280.0))
                    .px(px(24.0))
                    .py(px(8.0))
                    .flex_row()
                    .items_center()
                    .gap(px(10.0))
                    .child(
                        view()
                            .h(px(36.0))
                            .w(px(36.0))
                            .flex_shrink(0.0)
                            .items_center()
                            .justify_center()
                            .rounded(px(8.0))
                            .bg(move || theme().primary.opacity(0.15))
                            .child(
                                image(asset!("icons/download.svg"))
                                    .size(px(18.0))
                                    .tint_color(move || theme().primary),
                            ),
                    )
                    .child(
                        view()
                            .flex_col()
                            .flex_grow(1.0)
                            .min_w(px(0.0))
                            .child(
                                text(format!("Recurrency {version} is available"))
                                    .text_size(px(12.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(move || theme().foreground),
                            )
                            .child(
                                text(format!("You are on {}.", env!("CARGO_PKG_VERSION")))
                                    .text_size(px(10.0))
                                    .text_color(move || theme().muted_foreground),
                            ),
                    )
                    .when(cfg!(target_arch = "wasm32"), |view| {
                        view.child(crate::ui::action_button("View release", false, move || {
                            let url = url.clone();
                            spawn(async move {
                                let _ = open_url(url).await;
                            });
                        }))
                    })
                    .when(
                        !cfg!(target_arch = "wasm32") && download.is_some(),
                        |view| {
                            let download = download.clone();
                            view.child(crate::ui::action_button(
                                if updating.get() {
                                    "Starting…"
                                } else {
                                    "Update"
                                },
                                false,
                                move || {
                                    let Some(url) = download.clone() else { return };
                                    if updating.get() {
                                        return;
                                    }
                                    updating.set(true);
                                    spawn(async move {
                                        match crate::updater::install(&url).await {
                                            Ok(()) => crate::components::toast::success(
                                                "Downloading update. Follow the installer."
                                                    .to_string(),
                                            ),
                                            Err(error) => {
                                                crate::components::toast::error(format!(
                                                    "Could not start the update: {error}"
                                                ));
                                                let _ = open_url(url).await;
                                            }
                                        }
                                        updating.set(false);
                                    });
                                },
                            ))
                        },
                    )
                    .child(
                        button()
                            .cursor_pointer()
                            .items_center()
                            .justify_center()
                            .h(px(26.0))
                            .w(px(26.0))
                            .rounded(px(5.0))
                            .on_press(move || {
                                if let Some(info) = release.get().flatten() {
                                    dismiss_version(&info.tag_name);
                                }
                                dismissed.set(true);
                            })
                            .child(
                                image(asset!("icons/x.svg"))
                                    .size(px(13.0))
                                    .tint_color(move || theme().muted_foreground),
                            ),
                    ),
            )
            .into_element()
    })
    .otherwise(|| view().h(px(0.0)).into_element())
    .into_element()
}

#[cfg(test)]
mod tests {
    use super::preferred_asset;
    use crate::api::{Release, ReleaseAsset};

    #[test]
    fn the_platform_asset_is_preferred() {
        let release = Release {
            tag_name: "v9.0.0".to_string(),
            html_url: "https://example.com".to_string(),
            body: String::new(),
            assets: vec![
                ReleaseAsset {
                    name: "Recurrency-x86_64.AppImage".to_string(),
                    browser_download_url: "a".to_string(),
                },
                ReleaseAsset {
                    name: "Recurrency-x86_64.deb".to_string(),
                    browser_download_url: "b".to_string(),
                },
            ],
        };
        let asset = preferred_asset(&release).expect("a linux asset");
        assert!(
            asset
                .name
                .to_ascii_lowercase()
                .ends_with(if cfg!(target_os = "linux") {
                    ".appimage"
                } else {
                    ".apk"
                })
                || cfg!(not(target_os = "linux"))
        );
    }

    #[test]
    fn a_release_without_assets_has_no_download() {
        let release = Release {
            tag_name: "v9.0.0".to_string(),
            html_url: "https://example.com".to_string(),
            body: String::new(),
            assets: Vec::new(),
        };
        assert!(preferred_asset(&release).is_none());
    }
}

#[cfg(test)]
mod preference_tests {
    use super::*;

    #[test]
    fn newer_versions_are_detected_and_prereleases_do_not_regress() {
        assert!(newer("0.56.0", "v0.57.0"));
        assert!(!newer("0.56.0", "v0.56.0"));
        assert!(!newer("0.56.0", "0.56.0-beta.1"));
    }
}
