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

  mipselBinutils = pkgsCross.mipsel-linux-gnu.buildPackages.binutils;

  # mipsel-ps2-elf.json names mipsel-none-elf-ld as its linker
  mipsel-none-elf-ld = writeShellScriptBin "mipsel-none-elf-ld" ''
    exec ${mipselBinutils}/bin/mipsel-unknown-linux-gnu-ld "$@"
  '';

  # tools/dis.sh disassembles with `mipsel-objdump -m mips:5900`
  mipsel-objdump = writeShellScriptBin "mipsel-objdump" ''
    exec ${mipselBinutils}/bin/mipsel-unknown-linux-gnu-objdump "$@"
  '';

  resplice = (import (fetchTarball
    "https://github.com/cilki/nixpkgs/archive/refs/heads/resplice.tar.gz")
    { }).resplice;

  # Ghidra extension adding the PS2's Emotion Engine (R5900 core: MMI, VU0
  # macro mode, COP2, ...) as the language r5900:LE:32:default, plus a STABS
  # analyzer for ELFs with .mdebug and a PCSX2 savestate importer. The upstream
  # release ships one prebuilt zip per Ghidra version; the version in the file
  # name must match nixpkgs' ghidra (currently 12.1.2) or Ghidra refuses to
  # load it, so bump both together.
  # https://github.com/chaoticgd/ghidra-emotionengine-reloaded
  ghidra-emotionengine = stdenv.mkDerivation {
    pname = "ghidra-emotionengine-reloaded";
    version = "2.1.38";
    src = fetchurl {
      url =
        "https://github.com/chaoticgd/ghidra-emotionengine-reloaded/releases/download/v2.1.38/ghidra_${ghidra.version}_PUBLIC_20261004_ghidra-emotionengine-reloaded.zip";
      hash = "sha256-LmvdjtjnYrMCtH8M56D99Vtf561rBM/oz6B4qXWOZ64=";
    };
    nativeBuildInputs = [ unzip ];
    dontUnpack = true;
    # Same layout nixpkgs' buildGhidraExtension produces, so ghidra.withExtensions
    # can symlink it into the install tree
    installPhase = ''
      mkdir -p $out/lib/ghidra/Ghidra/Extensions
      unzip -q -d $out/lib/ghidra/Ghidra/Extensions $src
      # Prevent attempted creation of plugin lock files in the Nix store
      for i in $out/lib/ghidra/Ghidra/Extensions/*; do touch "$i/.dbDirLock"; done
    '';
  };

  # Two views of the same thing, needed because REA and Ghidra each look for
  # the extension differently (REA runs Ghidra headless with a throwaway user
  # config dir, so ~/.config/ghidra extensions would never be seen):
  #  - ghidraExtensions: extensions-only overlay (lib/ghidra/Ghidra/Extensions)
  #    that Ghidra itself loads as an extra application root via NIX_GHIDRAHOME.
  #    Ghidra locates its primary root through the real path of its jars, so
  #    it does not see Extensions symlinked next to them.
  #  - ghidraPs2: the full install plus the extension in one tree, for REA's
  #    GHIDRA_INSTALL_DIR: it validates support/analyzeHeadless etc. there and
  #    scans only <install>/Ghidra/{Processors,Extensions} for the .ldefs that
  #    must declare REA_GHIDRA_LANGUAGE_ID.
  ghidraExtensions = ghidra.withExtensions (_: [ ghidra-emotionengine ]);
  ghidraPs2 = symlinkJoin {
    name = "ghidra-ps2-${ghidra.version}";
    paths = [ ghidra ghidra-emotionengine ];
  };

  # cage must not bind its own socket next to the host's wayland-0 (bind()
  # fails with EADDRINUSE), so the container env gives everything cage's own
  # runtime dir (/tmp/cagert, see Dockerfile) and this wrapper points cage's
  # outbound connection at the host socket by absolute path. `cage -- ...`
  # then works with no env setup, as do grim/wtype against cage's socket.
  cage-nested = writeShellScriptBin "cage" ''
    mkdir -p /tmp/cagert && chmod 700 /tmp/cagert
    WAYLAND_DISPLAY=/tmp/runtime/wayland-0 exec ${cage}/bin/cage "$@"
  '';

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
    [ pkg-config rustToolchain rust-analyzer mipsel-none-elf-ld mipsel-objdump ];
  # cage runs PCSX2 in a nested wlroots compositor: to the host it's one
  # ordinary Wayland window, while its own socket (wayland-0 in /tmp/cagert)
  # gives the agent screencopy (grim) and virtual-keyboard (wtype) access
  buildInputs = [
    pcsx2
    claude-code
    p7zip
    ffmpeg
    mame-tools
    resplice
    python
    cage-nested
    grim
    wtype
    # bridges PINE's unix socket to TCP for pcsx2-mcp
    socat
    # Decompiler for the Reverse step, driven headless by REA (installed in the
    # Dockerfile). REA requires the full JDK Ghidra was built against and a
    # procps-compatible `ps` for its child-process ownership checks.
    ghidraPs2
    openjdk21
    procps
  ];

  # REA's Ghidra provider: https://github.com/morluto/rea/blob/main/docs/installation.md#ghidra
  GHIDRA_INSTALL_DIR = "${ghidraPs2}/lib/ghidra";
  # nixpkgs patches Ghidra to take an extra application root from this var.
  # The wrappers in ${ghidraExtensions}/bin set it themselves, but REA starts
  # the JVM directly from GHIDRA_INSTALL_DIR, so export it for everything.
  NIX_GHIDRAHOME = "${ghidraExtensions}/lib/ghidra/Ghidra";
  JAVA_HOME = openjdk21.home;
  # Import PS2 ELFs as the Emotion Engine language rather than whatever the
  # loader's opinion picks from the header (stock MIPS matches too). Note REA
  # also screens the ELF header itself before Ghidra runs and rejects a PS2
  # ELF's MIPS-III/5900/EABI64 flags; tools/rea_elf.py makes the copy it will
  # accept.
  REA_GHIDRA_LANGUAGE_ID = "r5900:LE:32:default";
  REA_GHIDRA_COMPILER_SPEC_ID = "default";
  REA_ANALYSIS_PROVIDER = "ghidra";

  # The container has no /etc/fonts; without this PCSX2's UI renders no text
  FONTCONFIG_FILE = makeFontsConf { fontDirectories = [ dejavu_fonts ]; };

  # Nix-built apps expect GPU drivers at /run/opengl-driver, which only
  # exists on NixOS hosts; without these PCSX2 finds no adapter and falls
  # back to the Null renderer. lvp (llvmpipe) is a CPU fallback in case the
  # host GPU isn't AMD/Intel/nouveau.
  LIBGL_DRIVERS_PATH = "${mesa}/lib/dri";
  # cage needs Mesa's GBM backend, which is likewise only searched for under
  # /run/opengl-driver by default; without it cage falls back to a software
  # renderer and PCSX2 dies with VK_ERROR_SURFACE_LOST_KHR
  GBM_BACKENDS_PATH = "${mesa}/lib/gbm";
  __EGL_VENDOR_LIBRARY_DIRS = "${mesa}/share/glvnd/egl_vendor.d";
  VK_DRIVER_FILES = lib.concatStringsSep ":"
    (map (icd: "${mesa}/share/vulkan/icd.d/${icd}_icd.x86_64.json") [
      "radeon"
      "intel"
      "nouveau"
      "lvp"
    ]);
}
