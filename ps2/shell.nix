# Pinned nixos-unstable; keep in sync with the pin in the Dockerfile
{ pkgs ? import (fetchTarball
  "https://github.com/NixOS/nixpkgs/archive/56c02bc00adcf003215cc4bd996d6efaf4cff188.tar.gz")
  { } }:

with pkgs;

let
  # Nightly Rust with rust-src: the custom PS2 target (mipsel-ps2-elf.json)
  # has no prebuilt core, so builds go through -Z build-std
  fenix = import (fetchTarball
    "https://github.com/nix-community/fenix/archive/0bc017b037b7cb5a492758d7753182e9862ac342.tar.gz") {
      inherit pkgs;
    };
  rustToolchain = fenix.complete.withComponents [
    "cargo"
    "clippy"
    "rust-src"
    "rustc"
    "rustfmt"
  ];

  # mipsel-ps2-elf.json names mipsel-none-elf-ld as its linker
  mipsel-none-elf-ld = writeShellScriptBin "mipsel-none-elf-ld" ''
    exec ${pkgsCross.mipsel-linux-gnu.buildPackages.binutils}/bin/mipsel-unknown-linux-gnu-ld "$@"
  '';

  resplice = (import (fetchTarball
    "https://github.com/cilki/nixpkgs/archive/refs/heads/resplice.tar.gz")
    { }).resplice;

  # Scripting for the Reverse/Fabricate steps: capstone/keystone/unicorn
  # handle MIPS disassembly/assembly/emulation (stock PCSX2 has no
  # disassembler), pyelftools/construct parse the game ELF and custom asset
  # formats, numpy diffs memory snapshots, pillow edits textures
  python = python3.withPackages (ps:
    with ps; [
      capstone
      keystone-engine
      unicorn
      pyelftools
      construct
      numpy
      pillow
    ]);
in mkShell rec {
  nativeBuildInputs =
    [ pkg-config rustToolchain rust-analyzer mipsel-none-elf-ld ];
  # cage runs PCSX2 in a nested wlroots compositor: to the host it's one
  # ordinary Wayland window, while its own socket (wayland-1) gives the
  # agent screencopy (grim) and virtual-keyboard (wtype) access to PCSX2
  buildInputs = [
    pcsx2
    claude-code
    p7zip
    xorriso
    mame-tools
    resplice
    python
    cage
    grim
    wtype
  ];

  # The container has no /etc/fonts; without this PCSX2's UI renders no text
  FONTCONFIG_FILE = makeFontsConf { fontDirectories = [ dejavu_fonts ]; };

  # Nix-built apps expect GPU drivers at /run/opengl-driver, which only
  # exists on NixOS hosts; without these PCSX2 finds no adapter and falls
  # back to the Null renderer. lvp (llvmpipe) is a CPU fallback in case the
  # host GPU isn't AMD/Intel/nouveau.
  LIBGL_DRIVERS_PATH = "${mesa}/lib/dri";
  __EGL_VENDOR_LIBRARY_DIRS = "${mesa}/share/glvnd/egl_vendor.d";
  VK_DRIVER_FILES = lib.concatStringsSep ":"
    (map (icd: "${mesa}/share/vulkan/icd.d/${icd}_icd.x86_64.json") [
      "radeon"
      "intel"
      "nouveau"
      "lvp"
    ]);
}
