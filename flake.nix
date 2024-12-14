let
  pkgs = import <nixpkgs> { };
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    gobject-introspection
    cargo
    cargo-tauri
    nodejs
    libudev-zero
    libudev0-shim
    libgudev

    librsvg
    pkgs.stdenv.cc.cc
  ];

  buildInputs = with pkgs;[
    pkgs.stdenv.cc.cc
    at-spi2-atk
    atkmm
    cairo
    gdk-pixbuf
    glib
    gtk3
    harfbuzz
    librsvg
    libsoup_3
    pango
    webkitgtk_4_1
    openssl
    pnpm
    helvum

    alsa-lib
    pkg-config
    libudev-zero
    libudev0-shim
    libgudev

    librsvg
  ];
}
