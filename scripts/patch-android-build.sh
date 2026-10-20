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

ICON="${ICON:-assets/icons/clock-1024.png}"

# Resolve ICON path relative to ROOT if relative
case "$ICON" in
    /*) ;;
    *)  ICON="$ROOT/$ICON" ;;
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

write_adaptive_xml() {
    cat << 'XMLEOF'
<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
  <background android:drawable="@drawable/ic_launcher_background"/>
  <foreground android:drawable="@drawable/ic_launcher_foreground"/>
</adaptive-icon>
XMLEOF
}

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

    # 2. Adaptive icon foreground/background (108dp canvas, icon in safe zone).
    mkdir -p "$res/drawable"
    $IM_CMD "$ICON" -resize 216x216 -gravity center -extent 432x432 \
        "$res/drawable/ic_launcher_foreground.png"
    $IM_CMD -size 432x432 xc:'#6366f1' "$res/drawable/ic_launcher_background.png"

    # 3. Drop dx-generated launcher XMLs (they shadow our drawables) and write
    #    our own adaptive icon for API 26+.
    find "$res" -name 'ic_launcher*.xml' -delete 2>/dev/null || true
    mkdir -p "$res/mipmap-anydpi-v26"
    write_adaptive_xml > "$res/mipmap-anydpi-v26/ic_launcher.xml"
    write_adaptive_xml > "$res/mipmap-anydpi-v26/ic_launcher_round.xml"
    echo "  adaptive icon XML written"

    # 4. App name: dx derives it from the Cargo crate name
    #    ("RecurrencyFrontend"). Force "Recurrency" here.
    mkdir -p "$res/values"
    local strings="$res/values/strings.xml"
    if [ -f "$strings" ]; then
        sed -i 's/RecurrencyFrontend/Recurrency/g' "$strings"
    else
        printf '<?xml version="1.0" encoding="utf-8"?>\n<resources>\n    <string name="app_name">Recurrency</string>\n</resources>\n' > "$strings"
    fi
    echo "  app_name → Recurrency"
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
