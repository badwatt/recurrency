use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const ANDROID_PROJECT: &str = "frontend/target/dx/recurrency_frontend/release/android";
const ANDROID_APP: &str = "frontend/target/dx/recurrency_frontend/release/android/app";
const ICON: &str = "frontend/assets/icons/clock-1024.png";
const NDK_VERSION: &str = "r27d";
const CMDLINE_TOOLS_VERSION: &str = "11076708";
const BUILD_TOOLS_VERSION: &str = "34.0.0";

fn run(cwd: &Path, program: &str, args: &[&str], envs: &[(String, String)]) -> Result<(), String> {
    let mut cmd = Command::new(program);
    cmd.args(args).current_dir(cwd);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    let status = cmd
        .status()
        .map_err(|err| format!("failed to run {program}: {err}"))?;
    if !status.success() {
        return Err(format!("{program} exited with {status}"));
    }
    Ok(())
}

fn command_exists(name: &str) -> bool {
    env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).any(|dir| dir.join(name).is_file()))
        .unwrap_or(false)
}

fn resolve_java() -> Result<String, String> {
    if let Ok(java_home) = env::var("JAVA_HOME")
        && Path::new(&java_home).is_dir()
    {
        return Ok(java_home);
    }

    if let Ok(output) = Command::new("nix-shell")
        .args(["-p", "jdk17", "--run", "echo $JAVA_HOME"])
        .output()
    {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if line.starts_with('/') && Path::new(line).is_dir() {
                return Ok(line.to_string());
            }
        }
    }

    if let Ok(output) = Command::new("find")
        .args([
            "/nix/store",
            "-maxdepth",
            "2",
            "-name",
            "openjdk",
            "-type",
            "d",
        ])
        .output()
    {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if Path::new(line).is_dir() {
                return Ok(line.to_string());
            }
        }
    }

    Err("JAVA_HOME not found. Install JDK 17.".into())
}

fn android_env() -> Result<Vec<(String, String)>, String> {
    let java_home = resolve_java()?;
    let home = env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    let ndk = format!("{home}/.android-sdk/ndk/{NDK_VERSION}");
    let home_toolchain = format!("{home}/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu");
    let temp_toolchain =
        "/tmp/recurrency-rustup/toolchains/nightly-x86_64-unknown-linux-gnu".to_string();
    let (toolchain, cargo_home, rustup_home) =
        if Path::new(&home_toolchain).join("bin/cargo").is_file() {
            (
                home_toolchain,
                format!("{home}/.local/cargo"),
                format!("{home}/.rustup"),
            )
        } else if Path::new(&temp_toolchain).join("bin/cargo").is_file() {
            (
                temp_toolchain,
                "/tmp/recurrency-cargo".to_string(),
                "/tmp/recurrency-rustup".to_string(),
            )
        } else {
            (String::new(), String::new(), String::new())
        };
    let has_toolchain = !toolchain.is_empty();

    let mut path = vec![format!("{java_home}/bin")];
    if has_toolchain {
        path.push(format!("{cargo_home}/bin"));
        path.push(format!("{toolchain}/bin"));
    }
    path.push(format!(
        "{home}/.local/share/.dx/tools/wasm-bindgen-v0.2.100/bin"
    ));
    path.push(format!("{home}/.local/cargo/bin"));
    path.push(format!("{home}/.cargo/bin"));
    if let Ok(existing) = env::var("PATH") {
        path.push(existing);
    }

    let mut envs = vec![
        ("JAVA_HOME".into(), java_home),
        ("PATH".into(), path.join(":")),
        ("ANDROID_HOME".into(), format!("{home}/.android-sdk")),
        ("ANDROID_NDK_HOME".into(), ndk.clone()),
        (
            "AR".into(),
            format!("{ndk}/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"),
        ),
        (
            "CC".into(),
            format!(
                "{ndk}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android28-clang"
            ),
        ),
        (
            "CXX".into(),
            format!(
                "{ndk}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android28-clang++"
            ),
        ),
        (
            "CFLAGS".into(),
            "-target aarch64-linux-android28 -fPIC".into(),
        ),
        (
            "CXXFLAGS".into(),
            "-target aarch64-linux-android28 -fPIC".into(),
        ),
        (
            "CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER".into(),
            format!(
                "{ndk}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android28-clang"
            ),
        ),
        (
            "CARGO_TARGET_AARCH64_LINUX_ANDROID_RUNNER".into(),
            "echo".into(),
        ),
    ];
    if has_toolchain {
        envs.push(("CARGO".into(), format!("{toolchain}/bin/cargo")));
        envs.push(("RUSTC".into(), format!("{toolchain}/bin/rustc")));
        envs.push((
            "RUSTUP_TOOLCHAIN".into(),
            "nightly-x86_64-unknown-linux-gnu".into(),
        ));
        envs.push(("RUSTUP_HOME".into(), rustup_home));
        envs.push(("CARGO_HOME".into(), cargo_home));
    }
    Ok(envs)
}

pub fn workspace_version(root: &Path) -> Result<String, String> {
    let cargo = fs::read_to_string(root.join("Cargo.toml"))
        .map_err(|err| format!("cannot read workspace Cargo.toml: {err}"))?;
    for line in cargo.lines() {
        if let Some(rest) = line.strip_prefix("version = \"")
            && let Some(version) = rest.strip_suffix('"')
        {
            return Ok(version.to_string());
        }
    }
    Err("workspace Cargo.toml has no version".into())
}

pub fn version_code(version: &str) -> Option<u32> {
    let core = version.split('-').next()?;
    let mut parts = core.split('.');
    let major: u32 = parts.next()?.parse().ok()?;
    let minor: u32 = parts.next()?.parse().ok()?;
    let patch: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(major * 10_000 + minor * 100 + patch)
}

pub fn patch_strings(content: &str) -> String {
    content.replace("RecurrencyFrontend", "Recurrency")
}

pub fn patch_manifest(content: &str) -> String {
    let mut out = content.replace(
        "android:label=\"RecurrencyFrontend\"",
        "android:label=\"Recurrency\"",
    );
    if !out.contains("android:allowBackup=") {
        if out.contains("<application ") {
            out = out.replacen(
                "<application ",
                "<application android:allowBackup=\"false\" ",
                1,
            );
        } else {
            out = out.replacen(
                "<application>",
                "<application android:allowBackup=\"false\">",
                1,
            );
        }
    }
    out
}

pub fn patch_styles(content: &str) -> String {
    if content.contains("android:statusBarColor") {
        return content.to_string();
    }
    if let Some(start) = content.find("<style name=\"AppTheme\"")
        && let Some(line_end) = content[start..].find('\n')
    {
        let insert_at = start + line_end + 1;
        let mut out = content.to_string();
        out.insert_str(insert_at, &style_items());
        return out;
    }
    if content.is_empty() {
        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<resources>\n    <style name=\"AppTheme\" parent=\"@style/Theme.AppCompat.Light.NoActionBar\">\n{}    </style>\n</resources>\n",
            style_items()
        )
    } else {
        content.to_string()
    }
}

fn style_items() -> String {
    concat!(
        "        <item name=\"android:statusBarColor\">#1e1e2e</item>\n",
        "        <item name=\"android:navigationBarColor\">#1e1e2e</item>\n",
        "        <item name=\"android:windowLightStatusBar\">false</item>\n",
        "        <item name=\"android:windowLightNavigationBar\">false</item>\n",
        "        <item name=\"android:windowBackground\">#1e1e2e</item>\n",
    )
    .to_string()
}

pub fn patch_gradle(content: &str, code: Option<u32>) -> String {
    let mut out = content
        .replace("extractNativeLibs=\"false\"", "extractNativeLibs=\"true\"")
        .replace("jvmTarget = \"1.8\"", "jvmTarget = \"17\"");

    if let Some(code) = code {
        let mut lines = Vec::new();
        for line in out.lines() {
            if line.trim_start().starts_with("versionCode = ") {
                let indent = line.strip_suffix(line.trim_start()).unwrap_or("");
                lines.push(format!("{indent}versionCode = {code}"));
                continue;
            }
            lines.push(line.to_string());
        }
        out = lines.join("\n");
        if content.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

const FOUC_START: &str = "        // Prevent FOUC:";
const BRIDGE_START: &str = "        // Expose the system-bars bridge";
const INJECT_END: &str = "            \"\"\", setOf(\"*\"))\n        }\n";
const SETTINGS: &str = "settings.javaScriptCanOpenWindowsAutomatically = true";
const BRIDGE_MARKER: &str = "// --- Recurrency system bars bridge ---";

const LOADER_SCRIPT: &str = r##"(function(){
var S=document.createElement('style');
S.textContent='body{visibility:hidden}html{background:#1e1e2e;color:#cdd6f4}html.light{background:#FAF9F2;color:#212121}html.dark{background:#0d0d11;color:#ededed}html.mocha{background:#1e1e2e;color:#cdd6f4}#dxloader{visibility:visible!important;display:flex;flex-direction:column;align-items:center;justify-content:center;position:fixed;inset:0;z-index:99999;font-family:system-ui,sans-serif}@keyframes dxspin{to{transform:rotate(360deg)}}@keyframes dxpulse{0%,100%{opacity:1}50%{opacity:.5}}@keyframes dxdot{0%,60%,100%{opacity:0}30%{opacity:1}}#dxloader .spin-outer{position:relative;width:48px;height:48px}#dxloader .spin-ring{position:absolute;inset:0;border-radius:50%;border:4px solid #3b4261}#dxloader .spin-ring.top{border-top-color:#6366f1;animation:dxspin .9s linear infinite}#dxloader .spin-ring.inner{inset:6px;border-bottom-color:#818cf8;animation:dxspin 1.4s linear infinite reverse}#dxloader .brand{margin-top:24px;font-size:20px;font-weight:700;letter-spacing:-0.02em}#dxloader .tagline{margin-top:6px;font-size:13px;animation:dxpulse 2s ease-in-out infinite}#dxloader .tagline .dot{animation:dxdot 1.2s infinite;opacity:0}#dxloader .tagline .dot:nth-child(2){animation-delay:.2s}#dxloader .tagline .dot:nth-child(3){animation-delay:.4s}#dxloader.light{background:#FAF9F2;color:#212121}#dxloader.light .spin-ring{border-color:#d4d4d8}#dxloader.light .spin-ring.top{border-top-color:#6366f1}#dxloader.light .spin-ring.inner{border-bottom-color:#818cf8}#dxloader.light .brand{color:#212121}#dxloader.light .tagline{color:#71717a}#dxloader.dark{background:#0d0d11;color:#ededed}#dxloader.dark .spin-ring{border-color:#27272a}#dxloader.dark .spin-ring.top{border-top-color:#a78bfa}#dxloader.dark .spin-ring.inner{border-bottom-color:#7c3aed}#dxloader.dark .brand{color:#ededed}#dxloader.dark .tagline{color:#a1a1aa}';
document.documentElement.appendChild(S);
var t='mocha';
try{var s=localStorage.getItem('recurrency-theme');if(s&&['light','dark','mocha'].indexOf(s)>=0)t=s;}catch(e){}
if(t==='system'){try{if(window.matchMedia('(prefers-color-scheme: dark)').matches)t='mocha';else t='light';}catch(e){t='mocha';}}
var D=document.createElement('div');
D.id='dxloader';
D.innerHTML=`<div class="spin-outer"><div class="spin-ring top"></div><div class="spin-ring inner"></div></div><div class="brand">Recurrency</div><div class="tagline">Every expense is a subscription<span class="dot">.</span><span class="dot">.</span><span class="dot">.</span></div>`;
D.className=t;
document.documentElement.appendChild(D);
var h=document.documentElement;
if(t==='light')h.className='light';
else h.className=t+' dark';
})();"##;

fn inject_block() -> String {
    format!(
        concat!(
            "        // Expose the system-bars bridge so the theme can paint status/nav bars.\n",
            "        addJavascriptInterface(SystemBarsBridge(this), \"RecurrencyAndroid\")\n",
            "        // Prevent FOUC: mocha background + themed native loader.\n",
            "        // setBackgroundColor paints mocha behind WebView — eliminates white flash.\n",
            "        // addDocumentStartJavaScript injects CSS + loader HTML before WASM loads.\n",
            "        // Reads localStorage for theme preference to match loader colors to theme.\n",
            "        setBackgroundColor(android.graphics.Color.parseColor(\"#1e1e2e\"))\n",
            "        if (WebViewFeature.isFeatureSupported(WebViewFeature.DOCUMENT_START_SCRIPT)) {{\n",
            "            WebViewCompat.addDocumentStartJavaScript(this, \"\"\"\n",
            "{}\n",
            "            \"\"\", setOf(\"*\"))\n",
            "        }}\n",
        ),
        LOADER_SCRIPT
    )
}

const BRIDGE_CLASS: &str = r##"
// --- Recurrency system bars bridge ---
private class SystemBarsBridge(private val view: android.webkit.WebView) {
    @android.webkit.JavascriptInterface
    fun setSystemBars(color: String, light: Boolean) {
        val activity = view.context as? android.app.Activity ?: return
        activity.runOnUiThread {
            val window = activity.window
            val parsed = try {
                android.graphics.Color.parseColor(color)
            } catch (e: IllegalArgumentException) {
                return@runOnUiThread
            }
            window.statusBarColor = parsed
            window.navigationBarColor = parsed
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.Q) {
                window.isStatusBarContrastEnforced = false
                window.isNavigationBarContrastEnforced = false
            }
            androidx.core.view.WindowInsetsControllerCompat(window, window.decorView).apply {
                isAppearanceLightStatusBars = light
                isAppearanceLightNavigationBars = light
            }
            view.setBackgroundColor(parsed)
        }
    }
}
"##;

pub fn patch_webview(content: &str) -> String {
    let mut out = content.to_string();

    if let Some(start) = out.find(BRIDGE_START).or_else(|| out.find(FOUC_START))
        && let Some(rel) = out[start..].find(INJECT_END)
    {
        let end = start + rel + INJECT_END.len();
        let start = if start > 0 && out.as_bytes()[start - 1] == b'\n' {
            start - 1
        } else {
            start
        };
        out.replace_range(start..end, "");
    }

    if let Some(pos) = out.find(&format!("\n{BRIDGE_MARKER}")) {
        out.truncate(pos);
    }

    let mut lines = Vec::new();
    for line in out.lines() {
        if line
            .trim_start()
            .starts_with("addJavascriptInterface(SystemBarsBridge(this), \"RecurrencyAndroid\")")
        {
            continue;
        }
        lines.push(line.to_string());
    }
    out = lines.join("\n");

    if let Some(position) = out.find(SETTINGS) {
        let insert_at = position + SETTINGS.len();
        out.insert_str(insert_at, &format!("\n{}", inject_block()));
    }

    out = out.trim_end().to_string();
    out.push('\n');
    out.push_str(BRIDGE_CLASS);
    out.push('\n');
    out
}

fn collect_dirs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.push(path.clone());
            collect_dirs(&path, out);
        }
    }
}

fn collect_files(dir: &Path, name: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, name, out);
        } else if path.file_name().and_then(|value| value.to_str()) == Some(name) {
            out.push(path);
        }
    }
}

fn collect_all_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_all_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn image_magick() -> Option<&'static str> {
    if command_exists("magick") {
        Some("magick")
    } else if command_exists("convert") {
        Some("convert")
    } else {
        None
    }
}

fn patch_res_dir(res: &Path, icon: &Path, image_magick: &str) -> Result<(), String> {
    println!("--- Patching res: {} ---", res.display());
    for (size, dpi) in [
        (48, "mdpi"),
        (72, "hdpi"),
        (96, "xhdpi"),
        (144, "xxhdpi"),
        (192, "xxxhdpi"),
    ] {
        let dir = res.join(format!("mipmap-{dpi}"));
        fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
        let target = dir.join("ic_launcher.webp");
        run(
            res,
            image_magick,
            &[
                icon.to_string_lossy().as_ref(),
                "-resize",
                &format!("{size}x{size}"),
                target.to_string_lossy().as_ref(),
            ],
            &[],
        )?;
        fs::copy(&target, dir.join("ic_launcher_round.webp")).map_err(|err| err.to_string())?;
    }

    let mut files = Vec::new();
    collect_all_files(res, &mut files);
    for file in files {
        if file
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.contains("ic_launcher") && name.ends_with(".xml"))
        {
            let _ = fs::remove_file(file);
        }
    }

    let values = res.join("values");
    fs::create_dir_all(&values).map_err(|err| err.to_string())?;
    let strings = values.join("strings.xml");
    if strings.is_file() {
        let content = fs::read_to_string(&strings).map_err(|err| err.to_string())?;
        fs::write(&strings, patch_strings(&content)).map_err(|err| err.to_string())?;
    } else {
        fs::write(
            &strings,
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<resources>\n    <string name=\"app_name\">Recurrency</string>\n</resources>\n",
        )
        .map_err(|err| err.to_string())?;
    }

    let styles = values.join("styles.xml");
    if styles.is_file() {
        let content = fs::read_to_string(&styles).map_err(|err| err.to_string())?;
        fs::write(&styles, patch_styles(&content)).map_err(|err| err.to_string())?;
    } else {
        fs::write(&styles, patch_styles("")).map_err(|err| err.to_string())?;
    }
    println!("  launcher icons, app_name and system bars patched");
    Ok(())
}

fn resolve_icon(root: &Path) -> PathBuf {
    let icon = env::var("ICON")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(ICON));
    if icon.is_absolute() {
        return icon;
    }
    if root.join(&icon).is_file() {
        return root.join(icon);
    }
    root.join("frontend").join(icon)
}

fn patch_project(root: &Path, icon: &Path, version_code: u32) -> Result<(), String> {
    let android_root = root.join(ANDROID_PROJECT);
    if !android_root.is_dir() {
        return Err(format!(
            "Android project not found at {}. Run dx bundle first.",
            android_root.display()
        ));
    }
    if !icon.is_file() {
        return Err(format!("icon not found at {}", icon.display()));
    }
    let Some(image_magick) = image_magick() else {
        return Err("ImageMagick (magick or convert) not found".into());
    };

    let mut dirs = Vec::new();
    collect_dirs(&android_root, &mut dirs);
    let res_dirs: Vec<_> = dirs
        .into_iter()
        .filter(|path| path.ends_with("src/main/res"))
        .collect();
    if res_dirs.is_empty() {
        return Err(format!(
            "no */src/main/res directories under {}",
            android_root.display()
        ));
    }
    for res in &res_dirs {
        patch_res_dir(res, icon, image_magick)?;
    }

    let mut manifests = Vec::new();
    collect_files(&android_root, "AndroidManifest.xml", &mut manifests);
    for manifest in manifests {
        let content = fs::read_to_string(&manifest).map_err(|err| err.to_string())?;
        fs::write(&manifest, patch_manifest(&content)).map_err(|err| err.to_string())?;
    }

    let mut gradle_files = Vec::new();
    collect_files(&android_root, "build.gradle.kts", &mut gradle_files);
    for gradle in gradle_files {
        let content = fs::read_to_string(&gradle).map_err(|err| err.to_string())?;
        if content.contains("com.android.application") {
            fs::write(&gradle, patch_gradle(&content, Some(version_code)))
                .map_err(|err| err.to_string())?;
            println!(
                "--- Patched versionCode={version_code} in {} ---",
                gradle.display()
            );
        }
    }

    let gradle = root.join(ANDROID_APP).join("build.gradle.kts");
    if gradle.is_file() {
        let content = fs::read_to_string(&gradle).map_err(|err| err.to_string())?;
        fs::write(&gradle, patch_gradle(&content, None)).map_err(|err| err.to_string())?;
    }

    let mut webviews = Vec::new();
    collect_files(&android_root, "RustWebView.kt", &mut webviews);
    for webview in webviews {
        let content = fs::read_to_string(&webview).map_err(|err| err.to_string())?;
        fs::write(&webview, patch_webview(&content)).map_err(|err| err.to_string())?;
        println!("--- Patched {} ---", webview.display());
    }

    println!("--- Android patches complete ---");
    Ok(())
}

pub fn patch(root: &Path) -> Result<(), String> {
    let version = workspace_version(root)?;
    let code = version_code(&version)
        .ok_or_else(|| format!("cannot derive versionCode from {version}"))?;
    let icon = resolve_icon(root);
    patch_project(root, &icon, code)
}

pub fn setup(_root: &Path) -> Result<(), String> {
    let home = env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    let sdk = PathBuf::from(&home).join(".android-sdk");
    fs::create_dir_all(sdk.join("ndk")).map_err(|err| err.to_string())?;

    let ndk = sdk.join("ndk").join(NDK_VERSION);
    if !ndk.is_dir() {
        println!("Downloading Android NDK {NDK_VERSION}...");
        let url =
            format!("https://dl.google.com/android/repository/android-ndk-{NDK_VERSION}-linux.zip");
        run(&sdk, "curl", &["-sL", &url, "-o", "ndk.zip"], &[])?;
        run(&sdk, "unzip", &["-q", "ndk.zip"], &[])?;
        fs::rename(sdk.join(format!("android-ndk-{NDK_VERSION}")), &ndk)
            .map_err(|err| err.to_string())?;
        let _ = fs::remove_file(sdk.join("ndk.zip"));
    }

    let latest = sdk.join("cmdline-tools/latest");
    if !latest.join("bin").is_dir() {
        println!("Downloading command-line tools...");
        let url = format!(
            "https://dl.google.com/android/repository/commandlinetools-linux-{CMDLINE_TOOLS_VERSION}_latest.zip"
        );
        run(&sdk, "curl", &["-sL", &url, "-o", "cmdline.zip"], &[])?;
        let _ = fs::remove_dir_all(sdk.join("cmdline-tools"));
        run(&sdk, "unzip", &["-q", "cmdline.zip"], &[])?;
        fs::create_dir_all(&latest).map_err(|err| err.to_string())?;
        let tools = sdk.join("cmdline-tools");
        if let Ok(entries) = fs::read_dir(&tools) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name == "bin"
                    || name == "lib"
                    || name.ends_with(".txt")
                    || name.ends_with(".properties")
                {
                    let _ = fs::rename(&path, latest.join(name.as_ref()));
                }
            }
        }
        let _ = fs::remove_file(sdk.join("cmdline.zip"));
    }

    let sdkmanager = latest.join("bin/sdkmanager");
    if !sdkmanager.is_file() {
        return Err(format!("sdkmanager not found at {}", sdkmanager.display()));
    }

    let mut licenses = Command::new(&sdkmanager)
        .arg("--licenses")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| format!("failed to run sdkmanager: {err}"))?;
    if let Some(stdin) = licenses.stdin.as_mut() {
        let _ = stdin.write_all("y\n".repeat(100).as_bytes());
    }
    let _ = licenses.wait();

    run(
        &sdk,
        sdkmanager.to_string_lossy().as_ref(),
        &[
            "platform-tools",
            "platforms;android-34",
            "build-tools;34.0.0",
        ],
        &[],
    )?;
    println!(
        "Done! ANDROID_HOME={} ANDROID_NDK_HOME={}",
        sdk.display(),
        ndk.display()
    );
    Ok(())
}

fn ensure_debug_keystore(
    keystore: &Path,
    envs: &[(String, String)],
    java_home: &str,
) -> Result<(), String> {
    if keystore.is_file() {
        return Ok(());
    }
    let keytool = format!("{java_home}/bin/keytool");
    run(
        keystore.parent().unwrap_or(Path::new(".")),
        &keytool,
        &[
            "-genkey",
            "-v",
            "-keystore",
            keystore.to_string_lossy().as_ref(),
            "-alias",
            "debug",
            "-keyalg",
            "RSA",
            "-keysize",
            "2048",
            "-validity",
            "10000",
            "-storepass",
            "android",
            "-keypass",
            "android",
            "-dname",
            "CN=Debug,O=Recurrency,C=US",
        ],
        envs,
    )
}

fn find_first_apk(dir: &Path, suffix: &str) -> Option<PathBuf> {
    if !dir.is_dir() {
        return None;
    }
    let mut entries = fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in &entries {
        if path.is_dir() {
            if let Some(found) = find_first_apk(path, suffix) {
                return Some(found);
            }
        } else if path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.ends_with(suffix))
        {
            return Some(path.clone());
        }
    }
    None
}

pub fn build(root: &Path, out_dir: &Path) -> Result<(), String> {
    let home = env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    let sdk = PathBuf::from(&home).join(".android-sdk");
    if !sdk.join("platforms").is_dir() || !sdk.join("ndk").join(NDK_VERSION).is_dir() {
        setup(root)?;
    }

    let envs = android_env()?;
    if !envs.iter().any(|(key, _)| key == "CARGO") {
        return Err("No rustup nightly toolchain with aarch64-linux-android found. Install it with `rustup toolchain install nightly --target aarch64-linux-android`.".into());
    }
    let java_home = resolve_java()?;
    let version = workspace_version(root)?;
    let code = version_code(&version)
        .ok_or_else(|| format!("cannot derive versionCode from {version}"))?;
    let frontend = root.join("frontend");
    let android_root = frontend.join("target/dx/recurrency_frontend/release/android");
    let app_dir = android_root.join("app");
    let keystore = frontend.join("debug.keystore");

    ensure_debug_keystore(&keystore, &envs, &java_home)?;

    if android_root.is_dir() {
        fs::remove_dir_all(&android_root).map_err(|err| err.to_string())?;
    }

    println!("--- Building Android APK (aarch64) ---");
    run(
        &frontend,
        "dx",
        &[
            "bundle",
            "--platform",
            "android",
            "--target",
            "aarch64-linux-android",
            "--release",
        ],
        &envs,
    )?;

    let icon = frontend.join("assets/icons/clock-1024.png");
    patch_project(root, &icon, code)?;

    println!("--- Building signed APK ---");
    let gradlew = app_dir.join("gradlew");
    run(
        &app_dir,
        gradlew.to_string_lossy().as_ref(),
        &["clean"],
        &envs,
    )?;
    run(
        &app_dir,
        gradlew.to_string_lossy().as_ref(),
        &["assembleRelease", "-x", "lintVitalRelease", "-x", "lint"],
        &envs,
    )?;

    let outputs = app_dir.join("app/build/outputs/apk/release");
    let unsigned = find_first_apk(&outputs, "-unsigned.apk")
        .or_else(|| find_first_apk(&app_dir.join("app/build/outputs/apk"), ".apk"))
        .ok_or_else(|| "could not find the generated APK".to_string())?;

    let apksigner = format!(
        "{}/build-tools/{BUILD_TOOLS_VERSION}/lib/apksigner.jar",
        sdk.display()
    );
    let keystore_str = keystore.to_string_lossy().into_owned();
    let unsigned_str = unsigned.to_string_lossy().into_owned();
    run(
        &app_dir,
        &format!("{java_home}/bin/java"),
        &[
            "-jar",
            &apksigner,
            "sign",
            "--ks",
            &keystore_str,
            "--ks-key-alias",
            "debug",
            "--ks-pass",
            "pass:android",
            "--key-pass",
            "pass:android",
            &unsigned_str,
        ],
        &envs,
    )?;
    run(
        &app_dir,
        &format!("{java_home}/bin/java"),
        &["-jar", &apksigner, "verify", &unsigned_str],
        &envs,
    )?;

    fs::create_dir_all(out_dir).map_err(|err| err.to_string())?;
    let destination = out_dir.join(format!("recurrency-{version}-aarch64.apk"));
    fs::rename(&unsigned, &destination).map_err(|err| err.to_string())?;
    println!("--- Done: {} ---", destination.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_code_from_semver() {
        assert_eq!(version_code("0.25.0"), Some(2500));
        assert_eq!(version_code("1.2.3"), Some(10203));
        assert_eq!(version_code("0.25.0-beta.1"), Some(2500));
        assert_eq!(version_code("1.2"), None);
        assert_eq!(version_code("x.y.z"), None);
    }

    #[test]
    fn patch_gradle_sets_version_code_and_jvm_target() {
        let content = "android {\n    versionCode = 1\n    jvmTarget = \"1.8\"\n    extractNativeLibs=\"false\"\n}\n";
        let patched = patch_gradle(content, Some(2500));
        assert!(patched.contains("versionCode = 2500"));
        assert!(patched.contains("jvmTarget = \"17\""));
        assert!(patched.contains("extractNativeLibs=\"true\""));
    }

    #[test]
    fn patch_webview_is_idempotent() {
        let source = concat!(
            "        settings.javaScriptCanOpenWindowsAutomatically = true\n",
            "        if (WebViewFeature.isFeatureSupported(WebViewFeature.DOCUMENT_START_SCRIPT)) {\n",
            "            isDocumentStartScriptEnabled = true\n",
            "        }\n",
            "    }\n",
        );
        let once = patch_webview(source);
        let twice = patch_webview(&once);
        assert_eq!(once, twice);
        assert_eq!(once.matches("// Prevent FOUC:").count(), 1);
        assert_eq!(once.matches("RecurrencyAndroid").count(), 1);
        assert_eq!(
            once.matches("// --- Recurrency system bars bridge ---")
                .count(),
            1
        );
        assert!(once.contains("#1e1e2e"));
    }

    #[test]
    fn patch_strings_replaces_generated_name() {
        let content = "<string name=\"app_name\">RecurrencyFrontend</string>";
        assert_eq!(
            patch_strings(content),
            "<string name=\"app_name\">Recurrency</string>"
        );
    }

    #[test]
    fn patch_manifest_disables_backup_and_fixes_label() {
        let manifest =
            "<manifest><application android:label=\"RecurrencyFrontend\"></application></manifest>";
        let patched = patch_manifest(manifest);
        assert!(patched.contains("android:label=\"Recurrency\""));
        assert!(patched.contains("android:allowBackup=\"false\""));
        assert_eq!(patch_manifest(&patched), patched);
    }

    #[test]
    fn patch_styles_inserts_system_bar_colors() {
        let content = "<style name=\"AppTheme\" parent=\"@style/Theme.AppCompat.Light.NoActionBar\">\n</style>";
        let patched = patch_styles(content);
        assert!(patched.contains("android:statusBarColor"));
        assert_eq!(patch_styles(&patched), patched);
    }
}
