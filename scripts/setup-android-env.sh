#!/usr/bin/env bash
set -e
ANDROID_DIR="${HOME}/.android-sdk"
NDK_VERSION="r27d"
CMDLINE_TOOLS_VERSION="11076708"
mkdir -p "$ANDROID_DIR" "$ANDROID_DIR/ndk"

NDK_DIR="$ANDROID_DIR/ndk/$NDK_VERSION"
if [ ! -d "$NDK_DIR" ]; then
    echo "Downloading Android NDK $NDK_VERSION..."
    cd "$ANDROID_DIR"
    curl -sL "https://dl.google.com/android/repository/android-ndk-${NDK_VERSION}-linux.zip" -o ndk.zip
    unzip -q ndk.zip
    mv "android-ndk-${NDK_VERSION}" "$NDK_DIR"
    rm ndk.zip
fi

if [ ! -d "$ANDROID_DIR/cmdline-tools/latest/bin" ]; then
    echo "Downloading command-line tools..."
    cd "$ANDROID_DIR"
    rm -rf cmdline-tools cmdline.zip
    curl -sL "https://dl.google.com/android/repository/commandlinetools-linux-${CMDLINE_TOOLS_VERSION}_latest.zip" -o cmdline.zip
    unzip -q cmdline.zip
    # The zip extracts directly into cmdline-tools/
    mkdir -p cmdline-tools/latest
    mv cmdline-tools/bin cmdline-tools/lib cmdline-tools/*.txt cmdline-tools/*.properties cmdline-tools/latest/ 2>/dev/null || true
fi

export ANDROID_HOME="$ANDROID_DIR"
export ANDROID_NDK_HOME="$NDK_DIR"
export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"
yes | "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --licenses > /dev/null 2>&1 || true
echo "Installing SDK components..."
"$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" "platform-tools" "platforms;android-34" "build-tools;34.0.0" > /dev/null
echo "Done! ANDROID_HOME=$ANDROID_HOME ANDROID_NDK_HOME=$ANDROID_NDK_HOME"
