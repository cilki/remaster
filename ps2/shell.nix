{ pkgs ? import (fetchTarball
  "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz") { } }:

with pkgs;

mkShell rec {
  nativeBuildInputs =
    [ pkg-config cargo rustc rust-analyzer rustfmt clippy ];
  buildInputs = [ pcsx2 claude-code ];
}

