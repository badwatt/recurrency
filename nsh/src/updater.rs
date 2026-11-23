//! Start the platform installer for a newer release. The GitHub release is
//! still the source of truth; this module downloads the asset and hands it to
//! the operating system instead of replacing application files in place.

pub async fn install(url: &str) -> Result<(), String> {
    if !url.starts_with("https://github.com/badwatt/recurrency/") {
        return Err("The update URL is not trusted".to_string());
    }
    let response = nsh_core::element::fetch(nsh_core::element::HttpRequest::get(url))
        .await
        .map_err(|error| error.to_string())?;
    if !response.ok() {
        return Err(format!("The download answered {}", response.status()));
    }
    let bytes = response.body;
    install_bytes(url, &bytes).await
}

#[cfg(target_os = "android")]
async fn install_bytes(url: &str, bytes: &[u8]) -> Result<(), String> {
    // Android can only install an APK after the user confirms the package
    // installer; open the download so that prompt is available.
    let _ = (url, bytes);
    Err("The Android installer is not available in this build".to_string())
}

#[cfg(any(target_os = "ios", target_arch = "wasm32"))]
async fn install_bytes(url: &str, bytes: &[u8]) -> Result<(), String> {
    let _ = (url, bytes);
    Err("Native updates are not available on this platform".to_string())
}

#[cfg(not(any(target_os = "android", target_os = "ios", target_arch = "wasm32")))]
async fn install_bytes(url: &str, bytes: &[u8]) -> Result<(), String> {
    let file_name = url
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .ok_or_else(|| "Invalid update filename".to_string())?;
    let directory = std::env::temp_dir().join("recurrency-updates");
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let path = directory.join(file_name);
    std::fs::write(&path, bytes).map_err(|error| error.to_string())?;

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
    let command = "open";
    #[cfg(target_os = "linux")]
    let command = "xdg-open";
    #[cfg(target_os = "windows")]
    let command = "cmd";

    #[cfg(target_os = "windows")]
    let mut process = {
        let mut process = std::process::Command::new(command);
        process.args(["/C", "start", "", path.to_string_lossy().as_ref()]);
        process
    };
    #[cfg(not(target_os = "windows"))]
    let mut process = std::process::Command::new(command);
    process.arg(&path);
    process
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}
