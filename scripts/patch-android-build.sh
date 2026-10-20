#!/usr/bin/env bash
# Patches a Dioxus-generated Android project so the APK uses the Recurrency
# app icon (not the default Dioxus icon) and is labelled "Recurrency"
# (not the auto-generated "RecurrencyFrontend" from the Cargo crate name).
#
# Shared between scripts/build-android.sh (local dev) and
# .github/workflows/release.yml (CI). Keeps both flows in sync.
#
# Usage: patch-android-build.sh [res_dir] [gradle_file] [webview_kt]
# The res_dir only seeds discovery: the script applies the launcher icons and
# strings.xml to EVERY `*/src/main/res` under the android project, because the
# dx layout has shifted across versions (inner app/app/... vs single app/...)
# and Gradle's :app sourceSet decides which one is actually packaged.

set -e

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

RES_HINT="${1:-target/dx/recurrency-frontend/release/android/app/app/src/main/res}"
GRADLE_FILE="${2:-target/dx/recurrency-frontend/release/android/app/build.gradle.kts}"
WEBVIEW_KT="${3:-target/dx/recurrency-frontend/release/android/app/app/src/main/kotlin/dev/dioxus/main/RustWebView.kt}"

ICON="${ICON:-frontend/assets/icons/clock-1024.png}"

# Resolve ICON path relative to ROOT (or ROOT/frontend) if relative
case "$ICON" in
    /*) ;;
    *)
        if [ -f "$ROOT/$ICON" ]; then
            ICON="$ROOT/$ICON"
        else
            ICON="$ROOT/frontend/$ICON"
        fi
        ;;
esac

if [ ! -f "$ICON" ]; then
    echo "ERROR: icon not found at $ICON"
    exit 1
fi

# Pick ImageMagick command (magick on modern, convert on legacy)
if command -v magick >/dev/null 2>&1; then
    IM_CMD="magick"
elif command -v convert >/dev/null 2>&1; then
    IM_CMD="convert"
else
    echo "ERROR: ImageMagick (magick or convert) not found"
    exit 1
fi

# Android project root (…/release/android), derived from the res hint.
ANDROID_ROOT="$(echo "$RES_HINT" | sed 's@/app/.*@@')"
if [ ! -d "$ANDROID_ROOT" ]; then
    echo "ERROR: android project not found at $ANDROID_ROOT"
    exit 1
fi

# Discover every res dir the project may package.
RES_DIRS=()
while IFS= read -r d; do
    RES_DIRS+=("$d")
done < <(find "$ANDROID_ROOT" -type d -path '*/src/main/res' 2>/dev/null | sort -u)

if [ "${#RES_DIRS[@]}" -eq 0 ]; then
    echo "ERROR: no */src/main/res directories under $ANDROID_ROOT"
    exit 1
fi

patch_res_dir() {
    local res="$1"
    echo "--- Patching res: $res ---"

    # 1. Launcher icons for every density (legacy + round).
    for size_px in 48:mdpi 72:hdpi 96:xhdpi 144:xxhdpi 192:xxxhdpi; do
        local size="${size_px%%:*}"
        local dpi="${size_px##*:}"
        mkdir -p "$res/mipmap-$dpi"
        $IM_CMD "$ICON" -resize "${size}x${size}" "$res/mipmap-$dpi/ic_launcher.webp"
        cp "$res/mipmap-$dpi/ic_launcher.webp" "$res/mipmap-$dpi/ic_launcher_round.webp"
    done
    echo "  launcher webp written (mdpi..xxxhdpi)"

    # 2. Drop every adaptive-icon / vector XML so the launcher falls back to
    #    our raster mipmaps. dx's adaptive XML references @drawable resources
    #    that AAPT does not merge from this res dir, which breaks resource
    #    linking (`resource drawable/ic_launcher_foreground not found`).
    find "$res" -name '*ic_launcher*.xml' -delete 2>/dev/null || true
    echo "  removed adaptive/vector launcher XMLs"

    # 3. App name: dx derives it from the Cargo crate name
    #    ("RecurrencyFrontend"). Force "Recurrency" here.
    mkdir -p "$res/values"
    local strings="$res/values/strings.xml"
    if [ -f "$strings" ]; then
        sed -i 's/RecurrencyFrontend/Recurrency/g' "$strings"
    else
        printf '<?xml version="1.0" encoding="utf-8"?>\n<resources>\n    <string name="app_name">Recurrency</string>\n</resources>\n' > "$strings"
    fi
    echo "  app_name → Recurrency"

    # 4. System bars follow the app theme. Mocha is the default; the web app
    #    updates the colors at runtime through the RecurrencyAndroid bridge.
    local styles="$res/values/styles.xml"
    if [ -f "$styles" ]; then
        if ! grep -q 'android:statusBarColor' "$styles"; then
            sed -i '/<style name="AppTheme"/a\        <item name="android:statusBarColor">#1e1e2e</item>\n        <item name="android:navigationBarColor">#1e1e2e</item>\n        <item name="android:windowLightStatusBar">false</item>\n        <item name="android:windowLightNavigationBar">false</item>\n        <item name="android:windowBackground">#1e1e2e</item>' "$styles"
        fi
    else
        cat > "$styles" <<'XMLEOF'
<?xml version="1.0" encoding="utf-8"?>
<resources>
    <style name="AppTheme" parent="@style/Theme.AppCompat.Light.NoActionBar">
        <item name="android:statusBarColor">#1e1e2e</item>
        <item name="android:navigationBarColor">#1e1e2e</item>
        <item name="android:windowLightStatusBar">false</item>
        <item name="android:windowLightNavigationBar">false</item>
        <item name="android:windowBackground">#1e1e2e</item>
    </style>
</resources>
XMLEOF
    fi
    echo "  system bars → mocha default"
}

for res in "${RES_DIRS[@]}"; do
    patch_res_dir "$res"
done

# 5. Some layouts hardcode the label on the manifest instead of @string/app_name.
while IFS= read -r manifest; do
    sed -i 's/android:label="RecurrencyFrontend"/android:label="Recurrency"/g' "$manifest"
done < <(find "$ANDROID_ROOT" -name AndroidManifest.xml -type f 2>/dev/null)
echo "--- Patched AndroidManifest label ---"

# 6. Patch Gradle: extractNativeLibs=true, jvmTarget=17
if [ -f "$GRADLE_FILE" ]; then
    sed -i 's/extractNativeLibs="false"/extractNativeLibs="true"/g' "$GRADLE_FILE"
    sed -i 's/jvmTarget = "1\.8"/jvmTarget = "17"/g' "$GRADLE_FILE"
    echo "--- Patched Gradle: extractNativeLibs=true, jvmTarget=17 ---"
else
    echo "WARNING: gradle file not found at $GRADLE_FILE"
fi

# 7. Patch WebView (FOUC + native loader)
if [ -f "$WEBVIEW_KT" ] && [ -f "$ROOT/scripts/patch_webview.py" ]; then
    python3 "$ROOT/scripts/patch_webview.py" "$WEBVIEW_KT"
    echo "--- Patched RustWebView.kt ---"
fi

echo "--- Android patches complete ---"
