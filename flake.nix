{
  description = "Recurrency - Subscription & Expense Tracker";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
            inherit system;
            overlays = [
              (import rust-overlay)
              (final: prev: {
                wasm-bindgen-cli = final.rustPlatform.buildRustPackage rec {
                  pname = "wasm-bindgen-cli";
                  version = "0.2.130";
                  src = final.fetchCrate {
                    inherit pname version;
                    hash = "sha256-9Og1O/6/VC3sc5hBjoIWBErmLW2UHGMsTQO+6+m/dng=";
                  };
                  cargoHash = "sha256-j81sUf1m5yXU4FJfGj7GV+hdqVCV8mRmNb0QIsMCjqw=";
                };
              })
            ];
          };
        rustAndroid = pkgs.rust-bin.stable.latest.default.override {
          targets = [
            "aarch64-linux-android"
            "armv7-linux-androideabi"
            "x86_64-linux-android"
          ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # Rust toolchain
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
            cargo-llvm-cov
            llvmPackages.llvm
            llvmPackages.lld
            # Wasm toolchain
            dioxus-cli
            wasm-bindgen-cli
            # Needed by `dx bundle --release` (wasm-opt)
            binaryen

            # Node.js & Bun
            nodejs_22
            bun

            # Tauri v2 Linux dependencies
            gtk3
            webkitgtk_4_1
            # Native Shell (nsh) Linux backend
            gtk4
            webkitgtk_6_0
            gst_all_1.gstreamer
            gst_all_1.gst-plugins-base
            clang
            librsvg
            glib
            libsoup_3
            glib-networking
            libayatana-appindicator

            # Additional build tools
            pkg-config
            alsa-lib
            openssl
            cacert
            dbus
            libxml2
            libxslt

            # X11 libraries
            cairo
            gdk-pixbuf
            pango
            at-spi2-core
            imagemagick
            libXcomposite
            libXdamage
            libXfixes
            libXrandr
            libICE
            libSM
            libX11
            libXcursor
            libXext
            libXi
            xdotool
            linuxdeploy
            libXinerama
            libxcb
          ];

          RUST_BACKTRACE = "1";
          OPENSSL_NO_VENDOR = "1";

          # cargo-llvm-cov uses these to find llvm tools
          LLVM_COV = "${pkgs.llvmPackages.llvm}/bin/llvm-cov";
          LLVM_PROFDATA = "${pkgs.llvmPackages.llvm}/bin/llvm-profdata";

          shellHook = ''
            echo "🟢 Recurrency dev environment ready"
            echo "Node: $(node --version)"
            echo "Rust: $(rustc --version)"
            echo "Bun: $(bun --version)"
          '';
        };

        devShells.nsh-android = pkgs.mkShell {
          packages = with pkgs; [
            rustAndroid
            pkg-config
            openssl
            cacert
            dbus
            glib
            gtk3
            gtk4
            webkitgtk_6_0
            pango
            cairo
            gdk-pixbuf
            gst_all_1.gstreamer
            gst_all_1.gst-plugins-base
            graphene
            libsoup_3
            libxml2
            libxslt
            jdk17
          ];

          RUST_BACKTRACE = "1";
          shellHook = ''
            export ANDROID_HOME="${ANDROID_HOME:-$HOME/.android-sdk}"
            export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/r27d}"
            export PATH="$ANDROID_HOME/platform-tools:$ANDROID_HOME/cmdline-tools/latest/bin:$PATH"
          '';
          OPENSSL_NO_VENDOR = "1";
        };

        devShells.android = pkgs.mkShell {
          packages = with pkgs; [
            cargo rustc rustfmt clippy
            nodejs_22 bun
            gtk3 webkitgtk_4_1 librsvg glib libsoup_3 glib-networking
            pkg-config openssl cacert dbus libxml2 libxslt
            cairo gdk-pixbuf pango
            jdk17
          ];

          # Android SDK/NDK from environment
          ANDROID_HOME = "$HOME/.android-sdk";
          ANDROID_NDK_HOME = "$HOME/.android-sdk/ndk/r27d";

          RUST_BACKTRACE = "1";
          OPENSSL_NO_VENDOR = "1";
        };
      }
    );
}
