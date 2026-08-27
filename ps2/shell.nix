# Pinned nixos-unstable; keep in sync with the pin in the Dockerfile
{ pkgs ? import (fetchTarball
  "https://github.com/NixOS/nixpkgs/archive/56c02bc00adcf003215cc4bd996d6efaf4cff188.tar.gz") { } }:

with pkgs;

let
  # Nightly Rust with rust-src: the custom PS2 target (mipsel-ps2-elf.json)
  # has no prebuilt core, so builds go through -Z build-std
  fenix = import (fetchTarball
    "https://github.com/nix-community/fenix/archive/0bc017b037b7cb5a492758d7753182e9862ac342.tar.gz")
    { inherit pkgs; };
  rustToolchain =
    fenix.complete.withComponents [ "cargo" "clippy" "rust-src" "rustc" "rustfmt" ];

  # mipsel-ps2-elf.json names mipsel-none-elf-ld as its linker
  mipsel-none-elf-ld = writeShellScriptBin "mipsel-none-elf-ld" ''
    exec ${pkgsCross.mipsel-linux-gnu.buildPackages.binutils}/bin/mipsel-unknown-linux-gnu-ld "$@"
  '';

  resplice = (import (fetchTarball
    "https://github.com/cilki/nixpkgs/archive/refs/heads/resplice.tar.gz")
    { }).resplice;
in mkShell rec {
  nativeBuildInputs =
    [ pkg-config rustToolchain rust-analyzer mipsel-none-elf-ld ];
  buildInputs = [ pcsx2 claude-code p7zip xorriso mame-tools resplice ];

  # The container has no /etc/fonts; without this PCSX2's UI renders no text
  FONTCONFIG_FILE = makeFontsConf { fontDirectories = [ dejavu_fonts ]; };
}
