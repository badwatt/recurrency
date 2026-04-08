{
  description = "Recur - Subscription & Expense Tracker";

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

            # X11 libraries (optional, for some features)
            cairo
            gdk-pixbuf
            pango
            at-spi2-core
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

          # Environment variables
          RUST_BACKTRACE = "1";
          OPENSSL_NO_VENDOR = "1";

          # Shell hook
          shellHook = ''
            echo "🟢 Recur dev environment ready"
            echo "Node: $(node --version)"
            echo "Rust: $(rustc --version)"
            echo "Bun: $(bun --version)"
          '';
        };
      }
    );
}
