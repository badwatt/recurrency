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
    pub fn apk_url(&self) -> Option<&str> {
        self.assets
            .iter()
            .find(|asset| asset.name.ends_with("-aarch64.apk"))
            .map(|asset| asset.browser_download_url.as_str())
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

/// Start the native Android download + package installer.
pub async fn install_apk(url: &str) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        let mut eval = document::eval(
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

#[cfg(test)]
mod tests {
    use super::*;

    fn release() -> Release {
        Release {
            tag_name: "v0.48.0".into(),
            html_url: "https://example.test".into(),
            assets: vec![
                ReleaseAsset {
                    name: "recurrency-0.48.0-aarch64.apk".into(),
                    browser_download_url: "https://example.test/app.apk".into(),
                },
                ReleaseAsset {
                    name: "checksums.txt".into(),
                    browser_download_url: "https://example.test/sums".into(),
                },
            ],
        }
    }

    #[test]
    fn finds_the_android_apk_asset() {
        assert_eq!(release().apk_url(), Some("https://example.test/app.apk"));
    }

    #[test]
    fn release_parses_assets() {
        let parsed: Release = serde_json::from_str(
            r#"{"tag_name":"v1","html_url":"u","assets":[{"name":"a-aarch64.apk","browser_download_url":"d"}]}"#,
        )
        .unwrap();
        assert_eq!(parsed.apk_url(), Some("d"));
    }
}
