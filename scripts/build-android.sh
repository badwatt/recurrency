#!/usr/bin/env bash
set -e

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Parse --out-dir argument
OUT_DIR=""
for arg in "$@"; do
  case $arg in
    --out-dir=*) OUT_DIR="${arg#--out-dir=}" ;;
    --out-dir) OUT_DIR="$2" ; shift ;;
  esac
done

cd "$ROOT/frontend"

# Default output dir when called without --out-dir (e.g. directly)
if [ -z "$OUT_DIR" ]; then
    OUT_DIR="$ROOT/bin/android"
fi

# Resolve Java from nix-shell (filter out shell banner)
JAVA_HOME="$(nix-shell -p jdk17 --run 'echo $JAVA_HOME' 2>/dev/null | grep -E '^/' | head -1)"
if [ -z "$JAVA_HOME" ] || [ ! -d "$JAVA_HOME" ]; then
  echo "ERROR: Could not resolve JAVA_HOME from nix-shell"
  echo "Trying fallback: searching /nix/store for openjdk..."
  JAVA_HOME="$(find /nix/store -maxdepth 2 -name 'openjdk' -type d 2>/dev/null | head -1)"
fi
if [ -z "$JAVA_HOME" ] || [ ! -d "$JAVA_HOME" ]; then
  echo "ERROR: JAVA_HOME not found. Install JDK 17."
  exit 1
fi
export JAVA_HOME
export PATH="$JAVA_HOME/bin:$PATH"

# Resolve dx from rustup nightly - must set CARGO/RUSTC to force rustup toolchain
export PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$HOME/.local/share/.dx/tools/wasm-bindgen-v0.2.100/bin:$HOME/.local/cargo/bin:$PATH"
export CARGO="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo"
export RUSTC="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc"
export RUSTUP_TOOLCHAIN=nightly-x86_64-unknown-linux-gnu
export ANDROID_HOME="$HOME/.android-sdk"
export ANDROID_NDK_HOME="$HOME/.android-sdk/ndk/r27d"

# Use NDK's clang as C/C++ compiler for native crates (aws-lc-sys)
export AR="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
export CC="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android28-clang"
export CXX="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android28-clang++"
export CFLAGS="-target aarch64-linux-android28 -fPIC"
export CXXFLAGS="-target aarch64-linux-android28 -fPIC"

# Force linker to NDK linker for Android target
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android28-clang"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_RUNNER="echo"

ICON="assets/icons/icon-1024.png"
# Resolve version from workspace Cargo.toml (no VERSION file exists)
VERSION=$(grep '^version' "$ROOT/Cargo.toml" | head -1 | sed 's/.*= *"\(.*\)"/\1/')
APK_NAME="recurrency-${VERSION}-aarch64.apk"
KEYSTORE="debug.keystore"
KEY_ALIAS="debug"
KEY_PASS="android"

# Generate debug keystore if missing
if [ ! -f "$KEYSTORE" ]; then
    echo "--- Generating debug keystore ---"
    keytool -genkey -v -keystore "$KEYSTORE" -alias "$KEY_ALIAS" -keyalg RSA -keysize 2048 -validity 10000 -storepass "$KEY_PASS" -keypass "$KEY_PASS" -dname "CN=Debug,O=Recurrency,C=US"
fi

echo "--- Building Android APK (aarch64) ---"
# Clean previous Android build to avoid duplicate resource conflicts.
# Dioxus generates ic_launcher_background.xml, previous builds leave .png residue.
rm -rf target/dx/recurrency-frontend/release/android 2>/dev/null || true
dx bundle --platform android --target aarch64-linux-android --release

# Replace icons
RES_DIR="target/dx/recurrency-frontend/release/android/app/app/src/main/res"
echo "--- Replacing Android icons ---"
for dir in mipmap-mdpi mipmap-hdpi mipmap-xhdpi mipmap-xxhdpi mipmap-xxxhdpi; do
    case $dir in
        mipmap-mdpi) SIZE=48;;
        mipmap-hdpi) SIZE=72;;
        mipmap-xhdpi) SIZE=96;;
        mipmap-xxhdpi) SIZE=144;;
        mipmap-xxxhdpi) SIZE=192;;
    esac
    TARGET="$RES_DIR/$dir/ic_launcher.webp"
    if [ -f "$TARGET" ]; then
        magick convert "$ICON" -resize ${SIZE}x${SIZE} "$TARGET"
        echo "  ${SIZE}×${SIZE} icon replaced"
    fi
done

# Generate foreground with padding + solid background for adaptive icons
# Android adaptive-icon: 108dp canvas, 72dp safe zone.
# Foreground: 432x432 canvas, 216x216 centered icon
mkdir -p "$RES_DIR/drawable" "$RES_DIR/drawable-nodpi"
# Remove Dioxus-generated XMLs to avoid duplicate resource conflict and shadowing.
# drawable-v24/ic_launcher_foreground.xml (white vector) shadows our custom PNG on API 24+.
rm -f "$RES_DIR/drawable/ic_launcher_background.xml" 2>/dev/null || true
rm -f "$RES_DIR/drawable-v24/ic_launcher_foreground.xml" 2>/dev/null || true
magick convert "$ICON" -resize 216x216 -gravity center -extent 432x432 "$RES_DIR/drawable/ic_launcher_foreground.png"
echo "--- Generated ic_launcher_foreground.png (432x432 with padding) ---"

magick convert -size 432x432 xc:'#0ea5e9' "$RES_DIR/drawable/ic_launcher_background.png"
echo "--- Generated ic_launcher_background.png (432x432 solid color) ---"

# Patch adaptive-icon XMLs: solid background + padded foreground
for dir in mipmap-mdpi mipmap-hdpi mipmap-xhdpi mipmap-xxhdpi mipmap-xxxhdpi; do
    LAUNCHER_XML="$RES_DIR/$dir/ic_launcher.xml"
    if [ -f "$LAUNCHER_XML" ]; then
        cat > "$LAUNCHER_XML" << 'XMLEOF'
<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
  <background android:drawable="@drawable/ic_launcher_background"/>
  <foreground android:drawable="@drawable/ic_launcher_foreground"/>
</adaptive-icon>
XMLEOF
        echo "--- Patched $LAUNCHER_XML ---"
    fi
done
GRADLE_FILE="target/dx/recurrency-frontend/release/android/app/app/build.gradle.kts"
if [ -f "$GRADLE_FILE" ]; then
    sed -i 's/extractNativeLibs="false"/extractNativeLibs="true"/g' "$GRADLE_FILE"
    sed -i 's/jvmTarget = "1\.8"/jvmTarget = "17"/g' "$GRADLE_FILE"
    echo "--- Patched Gradle: extractNativeLibs=true, jvmTarget=17 ---"
fi

# Reassemble + sign APK
echo "--- Building signed APK ---"
cd target/dx/recurrency-frontend/release/android/app
./gradlew clean 2>&1 | tail -1
if ! ./gradlew assembleRelease -x lintVitalRelease -x lint 2>&1; then
    echo "ERROR: Gradle build failed. Full output above."
    cd "$ROOT"
    exit 1
fi
cd -

UNSIGNED=$(find target/dx/recurrency-frontend/release/android/app/app/build/outputs/apk/release -name "*-unsigned.apk" | head -1)
if [ -z "$UNSIGNED" ]; then
    UNSIGNED=$(find target/dx/recurrency-frontend/release/android/app/app/build/outputs/apk -name "*.apk" | head -1)
fi
echo "Unsigned APK: $UNSIGNED"

# Create output directory
mkdir -p "$OUT_DIR"

# Sign with apksigner (sign in place, then move to output)
APKSIGNER_JAR="$ANDROID_HOME/build-tools/34.0.0/lib/apksigner.jar"
java -jar "$APKSIGNER_JAR" sign \
    --ks "$KEYSTORE" --ks-key-alias "$KEY_ALIAS" \
    --ks-pass "pass:$KEY_PASS" --key-pass "pass:$KEY_PASS" \
    "$UNSIGNED"

# Verify original
java -jar "$APKSIGNER_JAR" verify "$UNSIGNED" && echo "--- APK signed and verified ---" || echo "--- WARNING: APK verification failed ---"

# Move to output
mv "$UNSIGNED" "$OUT_DIR/$APK_NAME"

ls -la "$OUT_DIR/$APK_NAME"
echo "--- Done: $OUT_DIR/$APK_NAME ---"
