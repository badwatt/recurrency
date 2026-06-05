#!/usr/bin/env bash
# Patches a Dioxus-generated Android project so the APK uses the Recurrency
# app icon (not the default Dioxus icon) and is labelled "Recurrency"
# (not the auto-generated "RecurrencyFrontend" from the Cargo crate name).
#
# Shared between scripts/build-android.sh (local dev) and
# .github/workflows/release.yml (CI). Keeps both flows in sync.
#
# Usage: patch-android-build.sh <res_dir> <gradle_file> <webview_kt>
# All args optional — defaults assume dx 0.7.9 layout under
# target/dx/recurrency-frontend/release/android/app/ (gradlew + build.gradle.kts
# at single app; sources + bundle outputs under app/app/).

set -e

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

RES_DIR="${1:-target/dx/recurrency-frontend/release/android/app/app/src/main/res}"
GRADLE_FILE="${2:-target/dx/recurrency-frontend/release/android/app/build.gradle.kts}"
WEBVIEW_KT="${3:-target/dx/recurrency-frontend/release/android/app/app/src/main/kotlin/dev/dioxus/main/RustWebView.kt}"

ICON="${ICON:-assets/icons/icon-1024.png}"

# Resolve ICON path relative to ROOT if relative
case "$ICON" in
    /*) ;;
    *)  ICON="$ROOT/$ICON" ;;
esac

if [ ! -f "$ICON" ]; then
    echo "ERROR: icon not found at $ICON"
    exit 1
fi

if [ ! -d "$RES_DIR" ]; then
    echo "ERROR: res dir not found at $RES_DIR"
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

# ── 1. Replace launcher icons in each mipmap density ──
# dx 0.7.9 generates .webp launchers; we resize icon-1024.png into each.
echo "--- Replacing Android launcher icons ---"
for size_px in 48:mdpi 72:hdpi 96:xhdpi 144:xxhdpi 192:xxxhdpi; do
    size="${size_px%%:*}"
    dpi="${size_px##*:}"
    mkdir -p "$RES_DIR/mipmap-$dpi"
    TARGET="$RES_DIR/mipmap-$dpi/ic_launcher.webp"
    $IM_CMD "$ICON" -resize ${size}x${size} "$TARGET"
    echo "  ${size}x${size} → mipmap-$dpi/ic_launcher.webp"
done

# ── 2. Adaptive icon: solid background + padded foreground ──
# Android adaptive-icon: 108dp canvas, 72dp safe zone.
# Foreground: 432x432 canvas, 216x216 centered icon.
mkdir -p "$RES_DIR/drawable" "$RES_DIR/drawable-nodpi"
# Remove Dioxus-generated XMLs that shadow our custom drawables on API 24+.
find "$RES_DIR" -name 'ic_launcher_*.xml' -delete 2>/dev/null || true

$IM_CMD "$ICON" -resize 216x216 -gravity center -extent 432x432 \
    "$RES_DIR/drawable/ic_launcher_foreground.png"
echo "--- Generated ic_launcher_foreground.png (432x432 padded) ---"

$IM_CMD -size 432x432 xc:'#0ea5e9' "$RES_DIR/drawable/ic_launcher_background.png"
echo "--- Generated ic_launcher_background.png (432x432 solid) ---"

# Also write to single-app res (where Gradle's :app sourceSet points).
# RES_DIR = .../app/app/src/main/res. Single-app = .../app/src/main/res.
# Use sed to collapse the doubled 'app/app' to a single 'app/'.
SINGLE_APP_RES="$(echo "$RES_DIR" | sed 's@/app/app/@/app/@')"
if [ "$SINGLE_APP_RES" != "$RES_DIR" ]; then
    mkdir -p "$SINGLE_APP_RES/drawable"
    cp "$RES_DIR/drawable/ic_launcher_foreground.png" "$SINGLE_APP_RES/drawable/"
    cp "$RES_DIR/drawable/ic_launcher_background.png" "$SINGLE_APP_RES/drawable/"
    # Remove any dx-generated XML shadows in single-app res too.
    find "$SINGLE_APP_RES" -name 'ic_launcher_*.xml' -delete 2>/dev/null || true
    echo "--- Copied drawables to single-app res: $SINGLE_APP_RES/drawable ---"
fi

# Verify drawables exist — AAPT will fail silently otherwise.
if [ ! -f "$RES_DIR/drawable/ic_launcher_foreground.png" ]; then
    echo "ERROR: ic_launcher_foreground.png was not created at $RES_DIR/drawable/"
    exit 1
fi
if [ ! -f "$RES_DIR/drawable/ic_launcher_background.png" ]; then
    echo "ERROR: ic_launcher_background.png was not created at $RES_DIR/drawable/"
    exit 1
fi

# ── 3. Patch adaptive-icon XMLs: background + foreground ──
# mipmap-anydpi-v26 is the API 26+ adaptive icon. dx 0.7.9 may put it
# in the inner (app/app/src/main/res/) dir which AAPT reads with
# inconsistent sourceSet scoping. Delete the inner one and write
# only to the single-app res dir (where Gradle's :app sourceSet
# points by default). This makes AAPT look up @drawable refs from
# the single-app drawable/ dir we also write to.
SINGLE_APP_RES="$(echo "$RES_DIR" | sed 's@/app/app/@/app/@')"
# Remove dx-generated adaptive icon XMLs from the inner res dir.
rm -f "$RES_DIR/mipmap-anydpi-v26/ic_launcher.xml" 2>/dev/null || true
for dir in mipmap-mdpi mipmap-hdpi mipmap-xhdpi mipmap-xxhdpi mipmap-xxxhdpi; do
    rm -f "$RES_DIR/$dir/ic_launcher.xml" 2>/dev/null || true
done
# Write the adaptive icon XML to the single-app res dir.
if [ "$SINGLE_APP_RES" != "$RES_DIR" ]; then
    mkdir -p "$SINGLE_APP_RES/mipmap-anydpi-v26"
    cat > "$SINGLE_APP_RES/mipmap-anydpi-v26/ic_launcher.xml" << 'XMLEOF'
<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
  <background android:drawable="@drawable/ic_launcher_background"/>
  <foreground android:drawable="@drawable/ic_launcher_foreground"/>
</adaptive-icon>
XMLEOF
    echo "--- Wrote adaptive icon XML to single-app res ---"
fi

# ── 4. Patch Gradle: extractNativeLibs=true, jvmTarget=17 ──
if [ -f "$GRADLE_FILE" ]; then
    sed -i 's/extractNativeLibs="false"/extractNativeLibs="true"/g' "$GRADLE_FILE"
    sed -i 's/jvmTarget = "1\.8"/jvmTarget = "17"/g' "$GRADLE_FILE"
    echo "--- Patched Gradle: extractNativeLibs=true, jvmTarget=17 ---"
fi

# ── 5. Fix app name: Dioxus derives app name from Cargo crate name. ──
# Override to match Dioxus.toml [android] name = "Recurrency".
STRINGS_XML="$RES_DIR/values/strings.xml"
if [ -f "$STRINGS_XML" ]; then
    sed -i 's/RecurrencyFrontend/Recurrency/g' "$STRINGS_XML"
    echo "--- Patched strings.xml: app_name → Recurrency ---"
else
    echo "WARNING: strings.xml not found at $STRINGS_XML"
fi

# ── 6. Patch WebView (FOUC + native loader) ──
if [ -f "$WEBVIEW_KT" ] && [ -f "$ROOT/scripts/patch_webview.py" ]; then
    python3 "$ROOT/scripts/patch_webview.py" "$WEBVIEW_KT"
    echo "--- Patched RustWebView.kt ---"
fi

echo "--- Android patches complete ---"
