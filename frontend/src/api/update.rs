//! Latest GitHub release and in-app Android APK update.

use serde::Deserialize;

const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/badwatt/recurrency/releases/latest";

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Release {
    pub tag_name: String,
    pub html_url: String,
    #[serde(default)]
    pub assets: Vec<ReleaseAsset>,
}

impl Release {
    /// Pick the installer for the current native platform.
    pub fn native_asset(&self) -> Option<&ReleaseAsset> {
        #[cfg(not(any(target_os = "android", target_os = "linux", target_os = "macos", target_os = "windows")))]
        return None;

        #[cfg(any(target_os = "android", target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            #[cfg(target_os = "android")]
            let suffix = "-aarch64.apk";
            #[cfg(target_os = "linux")]
            let suffix = "_x86_64.AppImage";
            #[cfg(target_os = "macos")]
            let suffix = if std::env::consts::ARCH == "aarch64" {
                "_aarch64.dmg"
            } else {
                "_x86_64.dmg"
            };
            #[cfg(target_os = "windows")]
            let suffix = "_x64-setup.exe";

            self.assets
                .iter()
                .find(|asset| asset.name.ends_with(suffix))
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn latest() -> Option<Release> {
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
pub async fn latest() -> Option<Release> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let request = ureq::get(LATEST_RELEASE_URL)
            .header("User-Agent", "recurrency")
            .header("Accept", "application/vnd.github+json")
            .config()
            .timeout_global(Some(std::time::Duration::from_secs(10)))
            .http_status_as_error(false)
            .build();
        let result = request.call().ok().and_then(|mut response| {
            if !response.status().is_success() {
                return None;
            }
            response.body_mut().read_json::<Release>().ok()
        });
        let _ = tx.send(result);
    });
    tokio::time::timeout(std::time::Duration::from_secs(12), rx)
        .await
        .ok()?
        .ok()?
}

/// Download and start the native update flow.
pub async fn install(url: &str, filename: &str) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        let _ = filename;
        return install_apk(url).await;
    }
    #[cfg(not(target_os = "android"))]
    {
        install_native(url, filename).await
    }
}

/// Start the native Android download + package installer.
pub async fn install_apk(url: &str) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        let mut eval = dioxus::document::eval(
            r#"
            if (!window.RecurrencyUpdater || !window.RecurrencyUpdater.install) {
                dioxus.send('unavailable');
                return;
            }
            window.RecurrencyUpdater.install(await dioxus.recv());
            dioxus.send('started');
            "#,
        );
        let _ = eval.send(url.to_string());
        let result = eval.recv::<String>().await.unwrap_or_default();
        if result == "started" {
            Ok(())
        } else {
            Err("The Android installer is not available".to_string())
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = url;
        Err("In-app updates are available in the Android app".to_string())
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
async fn install_native(url: &str, filename: &str) -> Result<(), String> {
    if !url.starts_with("https://github.com/badwatt/recurrency/") {
        return Err("The update URL is not trusted".to_string());
    }
    let name = std::path::Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| "Invalid update filename".to_string())?
        .to_string();
    let base = crate::state::auth::get_app_data_dir()
        .ok_or_else(|| "No app data directory".to_string())?;
    let target = base.join("recurrency").join("updates").join(name);
    let Some(parent) = target.parent() else {
        return Err("Could not prepare the update directory".to_string());
    };
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;

    let url = url.to_string();
    let path = target.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let result = (|| {
            let response = ureq::get(&url)
                .header("User-Agent", "recurrency")
                .config()
                .timeout_global(Some(std::time::Duration::from_secs(120)))
                .build()
                .call()
                .map_err(|error| error.to_string())?;
            let mut file = std::fs::File::create(&path).map_err(|error| error.to_string())?;
            std::io::copy(&mut response.into_body().into_reader(), &mut file)
                .map_err(|error| error.to_string())?;
            Ok::<_, String>(path)
        })();
        let _ = tx.send(result);
    });
    let path = tokio::time::timeout(std::time::Duration::from_secs(125), rx)
        .await
        .map_err(|_| "Timed out downloading the update".to_string())?
        .map_err(|_| "The update download was cancelled".to_string())?
        ?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&path)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&path, permissions).map_err(|error| error.to_string())?;
    }

    #[cfg(target_os = "macos")]
    let status = std::process::Command::new("open").arg(&path).spawn();
    #[cfg(not(target_os = "macos"))]
    let status = std::process::Command::new(&path).spawn();
    status.map(|_| ()).map_err(|error| error.to_string())
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
async fn install_native(_url: &str, _filename: &str) -> Result<(), String> {
    Err("Native desktop updates are not available on this platform".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_asset_matches_the_current_platform() {
        let release = Release {
            tag_name: "v1".into(),
            html_url: "u".into(),
            assets: vec![
                ReleaseAsset {
                    name: "recurrency-desktop_1_x86_64.AppImage".into(),
                    browser_download_url: "linux".into(),
                },
                ReleaseAsset {
                    name: "recurrency-desktop_1_x86_64.dmg".into(),
                    browser_download_url: "mac-intel".into(),
                },
                ReleaseAsset {
                    name: "recurrency-desktop_1_aarch64.dmg".into(),
                    browser_download_url: "mac-arm".into(),
                },
                ReleaseAsset {
                    name: "recurrency-desktop_1_x64-setup.exe".into(),
                    browser_download_url: "windows".into(),
                },
                ReleaseAsset {
                    name: "recurrency-latest-aarch64.apk".into(),
                    browser_download_url: "android".into(),
                },
            ],
        };
        #[cfg(target_os = "linux")]
        assert_eq!(release.native_asset().unwrap().browser_download_url, "linux");
        #[cfg(target_os = "windows")]
        assert_eq!(release.native_asset().unwrap().browser_download_url, "windows");
        #[cfg(target_os = "android")]
        assert_eq!(release.native_asset().unwrap().browser_download_url, "android");
    }

    #[test]
    fn release_parses_assets() {
        let parsed: Release = serde_json::from_str(
            r#"{"tag_name":"v1","html_url":"u","assets":[{"name":"a-aarch64.apk","browser_download_url":"d"}]}"#,
        )
        .unwrap();
        assert_eq!(parsed.assets[0].name, "a-aarch64.apk");
    }
}
