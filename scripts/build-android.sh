#!/usr/bin/env bash
set -e
cd "$(dirname "$0")/../frontend"
export PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$HOME/.local/share/.dx/tools/wasm-bindgen-v0.2.100/bin:$HOME/.local/cargo/bin:$PATH"
export ANDROID_HOME="$HOME/.android-sdk"
export ANDROID_NDK_HOME="$HOME/.android-sdk/ndk/r27d"
export JAVA_HOME="/nix/store/nk7ra0997wd9pa517baq399kf9j5vgx3-openjdk-17.0.18+8/lib/openjdk"
export PATH="$JAVA_HOME/bin:$PATH"
ICON=assets/icons/icon-1024.png
VERSION=$(cat ../VERSION | tr -d '\n')
APK_NAME="recurrency-${VERSION}-aarch64.apk"

echo "--- Building Android APK (aarch64) ---"
dx bundle --platform android --target aarch64-linux-android --release

# Replace icons
RES_DIR="target/dx/recurrency/release/android/app/app/src/main/res"
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
        convert "$ICON" -resize ${SIZE}x${SIZE} "$TARGET"
        echo "Replaced $TARGET with ${SIZE}x${SIZE} icon"
    fi
done

# Patch Gradle: extractNativeLibs + Java 17
GRADLE_FILE="target/dx/recurrency/release/android/app/app/build.gradle.kts"
if [ -f "$GRADLE_FILE" ]; then
    sed -i 's/extractNativeLibs="false"/extractNativeLibs="true"/g' "$GRADLE_FILE"
    sed -i 's/jvmTarget = "1\.8"/jvmTarget = "17"/g' "$GRADLE_FILE"
    echo "--- Patched Gradle ---"
fi

# Reassemble APK
echo "--- Reassembling APK ---"
cd target/dx/recurrency/release/android/app
./gradlew assembleRelease -x lintVitalRelease -x lint 2>&1 | tail -5
cd -

RELEASE_APK=$(find target/dx/recurrency/release/android/app/app/build/outputs/apk -name "*-release*.apk" 2>/dev/null | grep -v universal | head -1)
DEBUG_APK=$(find target/dx/recurrency/release/android/app/app/build/outputs/apk -name "app-*.apk" 2>/dev/null | head -1)

if [ -n "$RELEASE_APK" ]; then
    cp "$RELEASE_APK" "dist/$APK_NAME"
    echo "--- Release APK ---"
elif [ -n "$DEBUG_APK" ]; then
    cp "$DEBUG_APK" "dist/$APK_NAME"
    echo "--- Debug APK (unsigned) ---"
else
    BUNDLETOOL="$HOME/.local/bin/bundletool"
    AAB=$(ls dist/Recurrency-aarch64-linux-android.aab 2>/dev/null || ls dist/*.aab 2>/dev/null | head -1)
    java -jar "$BUNDLETOOL" build-apks --bundle="$AAB" --output="dist/${APK_NAME}.apks" --mode=universal
    unzip -o "dist/${APK_NAME}.apks" -d dist/ universal.apk
    mv dist/universal.apk "dist/$APK_NAME"
    rm "dist/${APK_NAME}.apks"
    echo "--- Universal APK via bundletool ---"
fi

ls -la "dist/$APK_NAME"
echo "--- Done: dist/$APK_NAME ---"
