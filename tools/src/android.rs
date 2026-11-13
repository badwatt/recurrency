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
    if !out.contains("android.permission.CAMERA") {
        if let Some(pos) = out.find("<application") {
            out.insert_str(
                pos,
                "    <uses-permission android:name=\"android.permission.CAMERA\" />\n    <uses-permission android:name=\"android.permission.REQUEST_INSTALL_PACKAGES\" />\n",
            );
        }
    }
    if !out.contains("androidx.core.content.FileProvider") {
        let provider = r#"        <provider
            android:name="androidx.core.content.FileProvider"
            android:authorities="${applicationId}.fileprovider"
            android:exported="false"
            android:grantUriPermissions="true">
            <meta-data
                android:name="android.support.FILE_PROVIDER_PATHS"
                android:resource="@xml/recurrency_file_paths" />
        </provider>
"#;
        if let Some(pos) = out.rfind("</application>") {
            out.insert_str(pos, provider);
        }
    }
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
    if !out.contains("androidx.documentfile:documentfile")
        && out.contains("androidx.webkit:webkit")
        && out.contains("dependencies {")
    {
        out = out.replace("dependencies {", "dependencies {\n    implementation(\"androidx.documentfile:documentfile:1.0.1\")");
    }
    if !out.contains("com.journeyapps:zxing-android-embedded")
        && out.contains("com.android.application")
        && out.contains("dependencies {")
    {
        out = out.replace(
            "dependencies {",
            "dependencies {\n    implementation(\"androidx.appcompat:appcompat:1.7.0\")\n    implementation(\"com.journeyapps:zxing-android-embedded:4.3.0\")",
        );
    }

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
            "        addJavascriptInterface(ImagePickerBridge(this), \"RecurrencyImagePicker\")\n",
            "        addJavascriptInterface(FilePickerBridge(this), \"RecurrencyFilePicker\")\n",
            "        addJavascriptInterface(SyncFolderBridge(this), \"RecurrencySyncFolder\")\n",
            "        addJavascriptInterface(DownloadBridge(this), \"RecurrencyDownload\")\n",
            "        addJavascriptInterface(QrScannerBridge(this), \"RecurrencyQr\")\n",
            "        addJavascriptInterface(ApkUpdaterBridge(this), \"RecurrencyUpdater\")\n",
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

private class ImagePickerBridge(private val view: android.webkit.WebView) {
    @android.webkit.JavascriptInterface
    fun pickImage() {
        val activity = view.context as? WryActivity ?: return
        activity.runOnUiThread { activity.pickImageForWebView(view) }
    }
}

private class SyncFolderBridge(private val view: android.webkit.WebView) {
    private val prefs get() = view.context.getSharedPreferences("recurrency_sync", 0)

    private fun tree(): androidx.documentfile.provider.DocumentFile? {
        val raw = prefs.getString("tree_uri", null) ?: return null
        return try {
            androidx.documentfile.provider.DocumentFile.fromTreeUri(view.context, android.net.Uri.parse(raw))
        } catch (_: Exception) {
            null
        }
    }

    @android.webkit.JavascriptInterface
    fun hasFolder(): Boolean = tree() != null

    @android.webkit.JavascriptInterface
    fun folderName(): String = tree()?.name ?: ""

    @android.webkit.JavascriptInterface
    fun disconnect() {
        prefs.edit().remove("tree_uri").apply()
    }

    @android.webkit.JavascriptInterface
    fun chooseFolder() {
        val activity = view.context as? WryActivity ?: return
        activity.runOnUiThread { activity.chooseSyncFolder(view) }
    }

    @android.webkit.JavascriptInterface
    fun readAll(): String {
        val tree = tree() ?: return "[]"
        val result = org.json.JSONArray()
        for (file in tree.listFiles()) {
            if (file.isFile && file.name?.startsWith("recurrency-sync-") == true && file.name?.endsWith(".json") == true) {
                try {
                    val text = view.context.contentResolver.openInputStream(file.uri)?.bufferedReader()?.use { it.readText() } ?: ""
                    if (text.isNotEmpty()) result.put(text)
                } catch (_: Exception) {}
            }
        }
        return result.toString()
    }

    @android.webkit.JavascriptInterface
    fun writeFile(content: String): Boolean {
        val tree = tree() ?: return false
        val device = prefs.getString("device_id", null) ?: java.util.UUID.randomUUID().toString().also {
            prefs.edit().putString("device_id", it).apply()
        }
        val filename = "recurrency-sync-" + device + ".json"
        return try {
            val file = tree.findFile(filename) ?: tree.createFile("application/json", filename) ?: return false
            view.context.contentResolver.openOutputStream(file.uri, "wt")?.use { it.write(content.toByteArray(Charsets.UTF_8)) }
            true
        } catch (_: Exception) {
            false
        }
    }
}

private class FilePickerBridge(private val view: android.webkit.WebView) {
    @android.webkit.JavascriptInterface
    fun pickJson() {
        val activity = view.context as? WryActivity ?: return
        activity.runOnUiThread { activity.pickJsonForWebView(view) }
    }
}

private class QrScannerBridge(private val view: android.webkit.WebView) {
    @android.webkit.JavascriptInterface
    fun scan() {
        val activity = view.context as? WryActivity ?: return
        activity.runOnUiThread { activity.scanQrForWebView(view) }
    }
}

private class ApkUpdaterBridge(private val view: android.webkit.WebView) {
    @android.webkit.JavascriptInterface
    fun install(url: String) {
        val activity = view.context as? WryActivity ?: return
        activity.runOnUiThread { activity.downloadAndInstallApk(view, url) }
    }
}

private class DownloadBridge(private val view: android.webkit.WebView) {
    @android.webkit.JavascriptInterface
    fun saveFile(filename: String, mime: String, text: String) {
        val activity = view.context as? android.app.Activity ?: return
        Thread {
            try {
                val bytes = text.toByteArray(Charsets.UTF_8)
                if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.Q) {
                    val values = android.content.ContentValues().apply {
                        put(android.provider.MediaStore.MediaColumns.DISPLAY_NAME, filename)
                        put(android.provider.MediaStore.MediaColumns.MIME_TYPE, mime)
                        put(android.provider.MediaStore.MediaColumns.RELATIVE_PATH, android.os.Environment.DIRECTORY_DOWNLOADS)
                    }
                    val uri = activity.contentResolver.insert(
                        android.provider.MediaStore.Downloads.EXTERNAL_CONTENT_URI,
                        values,
                    )
                    if (uri != null) {
                        activity.contentResolver.openOutputStream(uri)?.use { it.write(bytes) }
                    }
                } else {
                    val dir = activity.getExternalFilesDir(
                        android.os.Environment.DIRECTORY_DOWNLOADS,
                    )
                    if (dir != null) {
                        java.io.File(dir, filename).writeBytes(bytes)
                        android.media.MediaScannerConnection.scanFile(
                            activity,
                            arrayOf(java.io.File(dir, filename).absolutePath),
                            arrayOf(mime),
                            null,
                        )
                    }
                }
                activity.runOnUiThread {
                    android.widget.Toast.makeText(activity, "Saved to Downloads", android.widget.Toast.LENGTH_SHORT).show()
                }
            } catch (_: Exception) {
                activity.runOnUiThread {
                    android.widget.Toast.makeText(activity, "Could not save file", android.widget.Toast.LENGTH_SHORT).show()
                }
            }
        }.start()
    }

    @android.webkit.JavascriptInterface
    fun printHtml(html: String) {
        val activity = view.context as? android.app.Activity ?: return
        activity.runOnUiThread {
            val printView = android.webkit.WebView(activity)
            printView.webViewClient = object : android.webkit.WebViewClient() {
                override fun onPageFinished(view: android.webkit.WebView, url: String?) {
                    val manager = activity.getSystemService(android.content.Context.PRINT_SERVICE)
                        as android.print.PrintManager
                    manager.print(
                        "Recurrency",
                        view.createPrintDocumentAdapter("Recurrency"),
                        android.print.PrintAttributes.Builder().build(),
                    )
                }
            }
            printView.loadDataWithBaseURL(null, html, "text/html", "UTF-8", null)
        }
    }
}
"##;

pub fn patch_wry_activity(content: &str) -> String {
    let mut out = content.to_string();
    if out.contains("// --- Recurrency image picker ---") {
        return patch_file_and_back(out);
    }
    if !out.contains("import android.content.Intent") {
        out = out.replace(
            "import android.os.Build",
            "import android.content.Intent\nimport android.os.Build",
        );
    }
    let field = "    private lateinit var mWebView: RustWebView";
    if out.contains(field) {
        out = out.replace(
            field,
            &format!("{field}\n    private var recurrencyImageWebView: android.webkit.WebView? = null\n    private var recurrencyFileWebView: android.webkit.WebView? = null\n    private var recurrencySyncWebView: android.webkit.WebView? = null\n    private var recurrencyLastBackPress: Long = 0"),
        );
    }
    let methods = r##"    // --- Recurrency image picker ---
    fun pickImageForWebView(view: android.webkit.WebView) {
        recurrencyImageWebView = view
        val intent = Intent(Intent.ACTION_GET_CONTENT).apply {
            type = "image/*"
            addCategory(Intent.CATEGORY_OPENABLE)
        }
        try {
            startActivityForResult(intent, RECURRENCY_IMAGE_PICK)
        } catch (_: Exception) {
            recurrencyImageWebView = null
        }
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode != RECURRENCY_IMAGE_PICK) return
        val view = recurrencyImageWebView ?: return
        recurrencyImageWebView = null
        if (resultCode != RESULT_OK) {
            view.post { view.evaluateJavascript("window.__recurrencyAvatarPicked && window.__recurrencyAvatarPicked('')", null) }
            return
        }
        val uri = data?.data
        if (uri == null) {
            view.post { view.evaluateJavascript("window.__recurrencyAvatarPicked && window.__recurrencyAvatarPicked('')", null) }
            return
        }
        Thread {
            try {
                val resolver = contentResolver
                val bounds = android.graphics.BitmapFactory.Options().apply { inJustDecodeBounds = true }
                resolver.openInputStream(uri)?.use {
                    android.graphics.BitmapFactory.decodeStream(it, null, bounds)
                }
                var sample = 1
                if (bounds.outWidth > 0 && bounds.outHeight > 0) {
                    while (bounds.outWidth / sample > 512 || bounds.outHeight / sample > 512) sample *= 2
                }
                val options = android.graphics.BitmapFactory.Options().apply { inSampleSize = sample }
                val bitmap = resolver.openInputStream(uri)?.use {
                    android.graphics.BitmapFactory.decodeStream(it, null, options)
                } ?: run {
                    view.post { view.evaluateJavascript("window.__recurrencyAvatarPicked && window.__recurrencyAvatarPicked('')", null) }
                    return@Thread
                }
                val max = 256
                val scale = minOf(1f, max.toFloat() / bitmap.width, max.toFloat() / bitmap.height)
                val width = (bitmap.width * scale).toInt().coerceAtLeast(1)
                val height = (bitmap.height * scale).toInt().coerceAtLeast(1)
                val scaled = android.graphics.Bitmap.createScaledBitmap(bitmap, width, height, true)
                val output = java.io.ByteArrayOutputStream()
                scaled.compress(android.graphics.Bitmap.CompressFormat.JPEG, 86, output)
                val encoded = android.util.Base64.encodeToString(output.toByteArray(), android.util.Base64.NO_WRAP)
                val js = "window.__recurrencyAvatarPicked && window.__recurrencyAvatarPicked('data:image/jpeg;base64," + encoded + "')"
                view.post { view.evaluateJavascript(js, null) }
            } catch (_: Exception) {
                view.post { view.evaluateJavascript("window.__recurrencyAvatarPicked && window.__recurrencyAvatarPicked('')", null) }
            }
        }.start()
    }

"##;
    if let Some(pos) = out.rfind("    companion object {") {
        out.insert_str(pos, methods);
    }
    let file_methods = r##"    // --- Recurrency QR scanner and APK updater ---
    fun scanQrForWebView(view: android.webkit.WebView) {
        recurrencyQrWebView = view
        try {
            com.journeyapps.barcodescanner.IntentIntegrator(this)
                .setPrompt("Scan a Recurrency server QR")
                .setBeepEnabled(false)
                .setOrientationLocked(false)
                .initiateScan()
        } catch (_: Exception) {
            recurrencyQrWebView = null
            view.post { view.evaluateJavascript("window.__recurrencyQrScanned && window.__recurrencyQrScanned('')", null) }
        }
    }

    fun downloadAndInstallApk(view: android.webkit.WebView, url: String) {
        recurrencyUpdateWebView = view
        if (!url.startsWith("https://github.com/badwatt/recurrency/") || !url.endsWith(".apk")) {
            view.post { view.evaluateJavascript("window.__recurrencyApkStatus && window.__recurrencyApkStatus('error')", null) }
            return
        }
        Thread {
            try {
                val connection = java.net.URL(url).openConnection() as java.net.HttpURLConnection
                connection.connectTimeout = 15000
                connection.readTimeout = 30000
                connection.instanceFollowRedirects = true
                connection.connect()
                if (connection.responseCode !in 200..299) throw IllegalStateException("HTTP " + connection.responseCode)
                val dir = java.io.File(cacheDir, "updates").apply { mkdirs() }
                val file = java.io.File(dir, "recurrency-latest.apk")
                connection.inputStream.use { input -> file.outputStream().use { output -> input.copyTo(output) } }
                val uri = androidx.core.content.FileProvider.getUriForFile(
                    this,
                    packageName + ".fileprovider",
                    file,
                )
                val intent = Intent(Intent.ACTION_VIEW).apply {
                    setDataAndType(uri, "application/vnd.android.package-archive")
                    addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                }
                runOnUiThread {
                    startActivity(intent)
                    view.post { view.evaluateJavascript("window.__recurrencyApkStatus && window.__recurrencyApkStatus('installing')", null) }
                }
            } catch (_: Exception) {
                view.post { view.evaluateJavascript("window.__recurrencyApkStatus && window.__recurrencyApkStatus('error')", null) }
            }
        }.start()
    }

    // --- Recurrency file picker and back handling ---
    fun chooseSyncFolder(view: android.webkit.WebView) {
        recurrencySyncWebView = view
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
            addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
            addFlags(Intent.FLAG_GRANT_PREFIX_URI_PERMISSION)
        }
        try {
            startActivityForResult(intent, RECURRENCY_SYNC_FOLDER)
        } catch (_: Exception) {
            recurrencySyncWebView = null
        }
    }

    fun pickJsonForWebView(view: android.webkit.WebView) {
        recurrencyFileWebView = view
        val intent = Intent(Intent.ACTION_GET_CONTENT).apply {
            type = "application/json"
            addCategory(Intent.CATEGORY_OPENABLE)
        }
        try {
            startActivityForResult(intent, RECURRENCY_FILE_PICK)
        } catch (_: Exception) {
            recurrencyFileWebView = null
        }
    }

    @Suppress("DEPRECATION")
    override fun onBackPressed() {
        if (mWebView.canGoBack()) {
            mWebView.goBack()
            return
        }
        val now = System.currentTimeMillis()
        if (now - recurrencyLastBackPress < 2000) {
            super.onBackPressed()
            return
        }
        recurrencyLastBackPress = now
        android.widget.Toast.makeText(
            this,
            "Press back again to exit",
            android.widget.Toast.LENGTH_SHORT,
        ).show()
    }

"##;
    if !out.contains("// --- Recurrency file picker and back handling ---")
        && let Some(pos) = out.rfind("    companion object {")
    {
        out.insert_str(pos, file_methods);
    }
    // Handle the JSON result before the image-picker branch.
    if !out.contains("requestCode == RECURRENCY_FILE_PICK") {
        out = out.replace(
            "        super.onActivityResult(requestCode, resultCode, data)\n        if (requestCode != RECURRENCY_IMAGE_PICK) return",
            "        super.onActivityResult(requestCode, resultCode, data)\n                if (requestCode == RECURRENCY_SYNC_FOLDER) {\n            val view = recurrencySyncWebView ?: return\n            recurrencySyncWebView = null\n            val uri = data?.data\n            if (resultCode == RESULT_OK && uri != null) {\n                try {\n                    contentResolver.takePersistableUriPermission(\n                        uri,\n                        Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION,\n                    )\n                    view.context.getSharedPreferences(\"recurrency_sync\", 0)\n                        .edit().putString(\"tree_uri\", uri.toString()).apply()\n                    view.post { view.evaluateJavascript(\"window.__recurrencySyncFolderChosen && window.__recurrencySyncFolderChosen('ok')\", null) }\n                } catch (_: Exception) {\n                    view.post { view.evaluateJavascript(\"window.__recurrencySyncFolderChosen && window.__recurrencySyncFolderChosen('')\", null) }\n                }\n            } else {\n                view.post { view.evaluateJavascript(\"window.__recurrencySyncFolderChosen && window.__recurrencySyncFolderChosen('')\", null) }\n            }\n            return\n        }\nif (requestCode == RECURRENCY_FILE_PICK) {\n            val view = recurrencyFileWebView ?: return\n            recurrencyFileWebView = null\n            val uri = data?.data\n            if (resultCode != RESULT_OK || uri == null) {\n                view.post { view.evaluateJavascript(\"window.__recurrencyBackupPicked && window.__recurrencyBackupPicked('')\", null) }\n                return\n            }\n            Thread {\n                try {\n                    val text = contentResolver.openInputStream(uri)?.bufferedReader()?.use { it.readText() } ?: \"\"\n                    val quoted = org.json.JSONObject.quote(text)\n                    view.post { view.evaluateJavascript(\"window.__recurrencyBackupPicked && window.__recurrencyBackupPicked(\" + quoted + \")\", null) }\n                } catch (_: Exception) {\n                    view.post { view.evaluateJavascript(\"window.__recurrencyBackupPicked && window.__recurrencyBackupPicked('')\", null) }\n                }\n            }.start()\n            return\n        }\n        if (requestCode != RECURRENCY_IMAGE_PICK) return",
        );
    }
    out = out.replace(
        "    companion object {\n",
        "    companion object {\n        private const val RECURRENCY_IMAGE_PICK = 4711\n        private const val RECURRENCY_FILE_PICK = 4712\n        private const val RECURRENCY_SYNC_FOLDER = 4713\n",
    );
    patch_file_and_back(out)
}

fn patch_file_and_back(mut out: String) -> String {
    let field = "    private lateinit var mWebView: RustWebView";
    if !out.contains("recurrencyFileWebView") && out.contains(field) {
        out = out.replace(
            field,
            &format!("{field}\n    private var recurrencyFileWebView: android.webkit.WebView? = null\n    private var recurrencySyncWebView: android.webkit.WebView? = null\n    private var recurrencyLastBackPress: Long = 0"),
        );
    }
    let methods = r##"    // --- Recurrency file picker and back handling ---
    fun pickJsonForWebView(view: android.webkit.WebView) {
        recurrencyFileWebView = view
        val intent = Intent(Intent.ACTION_GET_CONTENT).apply {
            type = "application/json"
            addCategory(Intent.CATEGORY_OPENABLE)
        }
        try {
            startActivityForResult(intent, RECURRENCY_FILE_PICK)
        } catch (_: Exception) {
            recurrencyFileWebView = null
        }
    }

    @Suppress("DEPRECATION")
    override fun onBackPressed() {
        if (mWebView.canGoBack()) {
            mWebView.goBack()
            return
        }
        val now = System.currentTimeMillis()
        if (now - recurrencyLastBackPress < 2000) {
            super.onBackPressed()
            return
        }
        recurrencyLastBackPress = now
        android.widget.Toast.makeText(
            this,
            "Press back again to exit",
            android.widget.Toast.LENGTH_SHORT,
        ).show()
    }

"##;
    if !out.contains("// --- Recurrency file picker and back handling ---")
        && let Some(pos) = out.rfind("    companion object {")
    {
        out.insert_str(pos, methods);
    }
    if !out.contains("private const val RECURRENCY_FILE_PICK")
        && out.contains("private const val RECURRENCY_IMAGE_PICK = 4711")
    {
        out = out.replace(
            "        private const val RECURRENCY_IMAGE_PICK = 4711\n",
            "        private const val RECURRENCY_IMAGE_PICK = 4711\n        private const val RECURRENCY_FILE_PICK = 4712\n        private const val RECURRENCY_SYNC_FOLDER = 4713\n",
        );
    }
    if !out.contains("requestCode == RECURRENCY_FILE_PICK") {
        out = out.replace(
            "        super.onActivityResult(requestCode, resultCode, data)\n        if (requestCode != RECURRENCY_IMAGE_PICK) return",
            "        super.onActivityResult(requestCode, resultCode, data)\n                if (requestCode == RECURRENCY_SYNC_FOLDER) {\n            val view = recurrencySyncWebView ?: return\n            recurrencySyncWebView = null\n            val uri = data?.data\n            if (resultCode == RESULT_OK && uri != null) {\n                try {\n                    contentResolver.takePersistableUriPermission(\n                        uri,\n                        Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION,\n                    )\n                    view.context.getSharedPreferences(\"recurrency_sync\", 0)\n                        .edit().putString(\"tree_uri\", uri.toString()).apply()\n                    view.post { view.evaluateJavascript(\"window.__recurrencySyncFolderChosen && window.__recurrencySyncFolderChosen('ok')\", null) }\n                } catch (_: Exception) {\n                    view.post { view.evaluateJavascript(\"window.__recurrencySyncFolderChosen && window.__recurrencySyncFolderChosen('')\", null) }\n                }\n            } else {\n                view.post { view.evaluateJavascript(\"window.__recurrencySyncFolderChosen && window.__recurrencySyncFolderChosen('')\", null) }\n            }\n            return\n        }\nif (requestCode == RECURRENCY_FILE_PICK) {\n            val view = recurrencyFileWebView ?: return\n            recurrencyFileWebView = null\n            val uri = data?.data\n            if (resultCode != RESULT_OK || uri == null) {\n                view.post { view.evaluateJavascript(\"window.__recurrencyBackupPicked && window.__recurrencyBackupPicked('')\", null) }\n                return\n            }\n            Thread {\n                try {\n                    val text = contentResolver.openInputStream(uri)?.bufferedReader()?.use { it.readText() } ?: \"\"\n                    val quoted = org.json.JSONObject.quote(text)\n                    view.post { view.evaluateJavascript(\"window.__recurrencyBackupPicked && window.__recurrencyBackupPicked(\" + quoted + \")\", null) }\n                } catch (_: Exception) {\n                    view.post { view.evaluateJavascript(\"window.__recurrencyBackupPicked && window.__recurrencyBackupPicked('')\", null) }\n                }\n            }.start()\n            return\n        }\n        if (requestCode != RECURRENCY_IMAGE_PICK) return",
        );
    }
    out
}

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
            || line
                .trim_start()
                .starts_with("addJavascriptInterface(ImagePickerBridge(this), \"RecurrencyImagePicker\")")
            || line
                .trim_start()
                .starts_with("addJavascriptInterface(FilePickerBridge(this), \"RecurrencyFilePicker\")")
            || line
                .trim_start()
                .starts_with("addJavascriptInterface(SyncFolderBridge(this), \"RecurrencySyncFolder\")")
            || line
                .trim_start()
                .starts_with("addJavascriptInterface(DownloadBridge(this), \"RecurrencyDownload\")")
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

    let xml_dir = res.join("xml");
    fs::create_dir_all(&xml_dir).map_err(|err| err.to_string())?;
    fs::write(
        xml_dir.join("recurrency_file_paths.xml"),
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<paths>\n    <cache-path name=\"updates\" path=\"updates/\" />\n</paths>\n",
    )
    .map_err(|err| err.to_string())?;

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

    let mut activities = Vec::new();
    collect_files(&android_root, "WryActivity.kt", &mut activities);
    for activity in activities {
        let content = fs::read_to_string(&activity).map_err(|err| err.to_string())?;
        fs::write(&activity, patch_wry_activity(&content)).map_err(|err| err.to_string())?;
        println!("--- Patched {} ---", activity.display());
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


#[cfg(test)]
mod image_picker_tests {
    use super::*;

    #[test]
    fn patch_wry_activity_installs_native_picker_once() {
        let source = "package dev.dioxus.main\nimport android.os.Build\nabstract class WryActivity {\n    private lateinit var mWebView: RustWebView\n    companion object {\n        init { }\n    }\n}\n";
        let once = patch_wry_activity(source);
        assert!(once.contains("Recurrency image picker"));
        assert!(once.contains("recurrencyImageWebView"));
        assert!(once.contains("RECURRENCY_IMAGE_PICK = 4711"));
        assert!(once.contains("onBackPressed"));
        assert!(once.contains("Press back again to exit"));
        assert!(once.contains("RECURRENCY_FILE_PICK"));
        assert!(once.contains("RECURRENCY_SYNC_FOLDER"));
        assert!(once.contains("chooseSyncFolder"));
        assert!(once.contains("recurrencySyncWebView"));
        assert!(once.contains("import android.content.Intent"));
        let twice = patch_wry_activity(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn patch_webview_installs_image_picker_bridge_once() {
        let source = "class RustWebView {\n    init {\n        settings.javaScriptCanOpenWindowsAutomatically = true\n    }\n}\n";
        let once = patch_webview(source);
        assert!(once.contains("ImagePickerBridge"));
        assert!(once.contains("SyncFolderBridge"));
        assert!(once.contains("RecurrencySyncFolder"));
        assert!(once.contains("RecurrencyImagePicker"));
        assert!(once.contains("DownloadBridge"));
        assert!(once.contains("RecurrencyDownload"));
        let twice = patch_webview(&once);
        assert_eq!(once.matches("addJavascriptInterface(ImagePickerBridge").count(), 1);
        assert_eq!(twice.matches("addJavascriptInterface(ImagePickerBridge").count(), 1);
    }
}
