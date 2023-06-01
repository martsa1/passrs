# https://github.com/jraygauthier/jrg-rust-cross-experiment/tree/master/simple-static-rustup-target-windows
# source: https://nixos.wiki/wiki/Rust#Installation_via_rustup
{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    clang
    cmake
    llvmPackages.bintools
    rustup

    fontconfig
    freetype
  ];
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];
  RUSTC_VERSION = pkgs.lib.readFile ./.rust-toolchain;
  # https://github.com/rust-lang/rust-bindgen#environment-variables
  LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
  shellHook = ''
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
    export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION/bin/

    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
      with pkgs;
      lib.makeLibraryPath [ libGL xorg.libX11 xorg.libXcursor xorg.libXrandr xorg.libXi ]
    }"
  '';
}
