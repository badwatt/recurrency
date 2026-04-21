{
  description = "Recurrency - Subscription & Expense Tracker";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
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

            # Node.js & Bun
            nodejs_22
            bun

            # Tauri v2 Linux dependencies
            gtk3
            webkitgtk_4_1
            librsvg
            glib
            libsoup_3
            glib-networking
            libayatana-appindicator

            # Additional build tools
            pkg-config
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
