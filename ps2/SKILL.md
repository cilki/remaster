---
name: remaster-ps2
description:
  Remaster a PS2 game end-to-end - extract the ISO, reverse-engineer it with the
  PCSX2 debugger, modify code and assets, repack a runnable ISO, and verify it
  in the emulator.
---

# REmaster skill

Create `/workspace/README.md` if it doesn't exist yet. This file tracks the
progress of our work and lets a fresh session resume where the last one left
off. Keep it in this shape:

```markdown
## Status

- Extract: pending | done
- Reverse: pending | in progress | done
- Fabricate: pending | in progress | done

## Game

<title, region/ID (e.g. SLUS-20398), boot path from SYSTEM.CNF, image path/size>

## Catalog

<file/asset inventory from the Reverse step>

## Change log

<each Fabricate change: what, where, how to verify it in-game>
```

Update the relevant section immediately after finishing work in a step, not at
the end of the session.

## Extract step

If we have a disc image under `/workspace`, extract it to
`/workspace/extracted`. Use `7z x <image> -o/workspace/extracted` for ISO files.
CHD images must first be converted with `chdman extractdvd`; BIN/CUE can be
converted with `7z` or `iat`. Keep the original image untouched — it's needed
again in the Repack step.

This step is complete when `/workspace/extracted` contains `SYSTEM.CNF` and the
boot ELF it names (e.g. `SLUS_123.45`). Do not proceed until then.

### Shim crate

Create the "shim" crate at `/workspace/shim` using `resplice`. This will be used
to reimplement small portions of the game in Rust via binary patching.

The shim must be `#![no_std]` and produce an rlib. The target uses a
conservative MIPS II baseline with soft floats; avoid atomics — the R5900 lacks
LL/SC, so emitted atomic sequences would fault on the real CPU. For quick
experiments, test changes via debugger memory writes before committing a splice
to the ELF.

> `resplice` is still immature, so attempt to find and patch any bugs in it.

```sh
cargo build --release -Z build-std=core,alloc -Z json-target-spec \
    --target /root/.claude/skills/remaster-ps2/mipsel-ps2-elf.json
```

## Viewing and controlling the game

Always launch PCSX2 through `cage`, a nested compositor that shows up as a
normal window on the user's desktop while giving you screen capture and keyboard
injection on its own Wayland socket. The environment is preconfigured for this:
the default `XDG_RUNTIME_DIR`/`WAYLAND_DISPLAY` already target cage's socket
(the host's socket is elsewhere — never capture or type on it), and `cage` is a
wrapper that connects itself out to the host display.

```sh
# Launch PCSX2 in the background (or use launch.sh, see below):
cage -- pcsx2-qt -batch -fullscreen /workspace/remaster.iso &

# Screenshot what PCSX2 is rendering, then view the PNG with the Read tool:
grim /tmp/screen.png

# Send keys into PCSX2 (F8 = native PCSX2 screenshot hotkey):
wtype -k F8

# Game input: tap a button, or hold a key for N ms (-P press, -s sleep,
# -p release):
wtype -k k                # Cross
wtype -P w -s 1000 -p w   # left stick up, 1s
```

Pad 1 is pre-bound to the keyboard in the image's PCSX2 config:

| PS2 control                        | Keys               |
| ---------------------------------- | ------------------ |
| D-pad                              | Arrow keys         |
| Left stick                         | W/A/S/D            |
| Right stick                        | T/F/G/H            |
| Triangle / Square / Cross / Circle | I / J / K / L      |
| Start / Select                     | Return / Backspace |
| L1 / L2 / L3                       | Q / 1 / 2          |
| R1 / R2 / R3                       | E / 3 / 4          |

Keyboard analog input is full-deflection only — fine for menus and basic play,
but you can't do partial stick tilts.

Gotchas:

- If cage fails with a renderer error, do NOT fall back to
  `WLR_RENDERER=pixman`: cage starts, but PCSX2 then dies with
  `VK_ERROR_SURFACE_LOST_KHR` / "Failed to create GS device" because a software
  compositor exposes no GPU-backed surface. Fix the GPU setup instead
  (`GBM_BACKENDS_PATH` must point at Mesa's `lib/gbm`; shell.nix sets it).
- The container has no sound device, so PCSX2 shows a modal audio error on boot
  ("cubeb_stream_init() failed"). It falls back to null output harmlessly —
  dismiss with `wtype -k Return`, or set the audio backend to `Nothing` in
  `/workspace/pcsx2.ini` so it never appears.
- cage exits the moment its child exits, with no error of its own — always check
  `/root/.config/PCSX2/logs/emulog.txt` for the real cause.

Prefer `launch.sh <iso> [wait_s]` (on PATH) for
(re)launching: it kills stale instances (the process is named `.pcsx2-qt-wrapp`,
which plain `pkill pcsx2-qt` misses — and never `pkill -f` a pattern that
appears in your own command line), clears stale cage socket locks and
`/dev/shm/pcsx2*` (the 256M /dev/shm limit SIGBUSes overlapping PCSX2
instances), then launches under cage and waits for it to come up.

### PCSX2 configuration

The config is declarative. The shared base ships at
`/opt/remaster/PCSX2.base.ini` (wizard skip, PINE, pad bindings); each game
may add overrides in `/workspace/pcsx2.ini` — same ini shape, its keys win,
new keys/sections are additive. On every launch, `launch.sh` merges the two
with `pcsx2ini.py` into `/root/.config/PCSX2/inis/PCSX2.ini`, so hand edits to
the live ini are lost on relaunch: put anything that should persist (renderer
upscaling, texture replacement, hotkeys, debug logging, the audio backend) in
`/workspace/pcsx2.ini` instead.

## Helper tools

Game-agnostic helpers installed on PATH (alongside `launch.sh` and
`repack_iso.py`):

| Tool                        | What it does                                                        |
| --------------------------- | ------------------------------------------------------------------- |
| `elfmap.py <elf> [va]`      | VA <-> file offset via the program headers; also a Python lib (`Img`) with string/word reads |
| `dis.sh <elf> <va> [n]`     | disassemble n instructions at a VA (`mipsel-objdump`, R5900)        |
| `xref.py <elf> <lo> [hi]`   | find `lui`/`addiu`-style pairs in executable segments that build an address in the range |
| `gsldump.py <gsl> <outdir>` | export every texture in a `.GSL` (GS upload list) to PNG            |
| `gslimport.py <gsl> <n> <png>` | import a PNG back over texture n, snapping to the record's CLUT  |
| `upscale.py <pack-root> [f]` | build a PCSX2 replacement pack from `<pack-root>/dumps`, per-texture kernel choice (see `scalers.py`) |
| `compare.py <prefix> <outdir> [shots]` | labelled side-by-side A/B images from a `<prefix>-off.png`/`-on.png` pair |
| `pcsx2ini.py <base> [override ...]` | merge PCSX2 ini fragments to stdout, later keys win (used by `launch.sh`) |

`gsl.py` (the `.GSL` reader/writer) and `scalers.py` (upscaling kernels) are
importable modules living in the same directory.

## Reverse step

Search `/workspace/extracted` for the game executable (named by `BOOT2` in
`SYSTEM.CNF`). Catalog the available files and assets in `/workspace/README.md`
for future steps. First check for debug symbols. The final goal of this step is
to reverse-engineer parts of the game that we might want to modify later.
Usually we want to leave the low-level engine code untouched to avoid getting
into trouble.

### Live debugging

In addition to static analysis, you should also run the game under an emulator
and examine the memory via the debugger. Here are two scripts that can help
drive PCSX2:

```sh
#!/usr/bin/env bash
# Game pad helper: gp.sh <key> [hold_ms] [wait_s]  |  gp.sh shot
if [ "$1" = "shot" ]; then grim "${2:-/tmp/screen.png}"; exit; fi
k="$1"; hold="${2:-500}"; wait="${3:-3}"
wtype -P "$k" -s "$hold" -p "$k"
sleep "$wait"
grim "${4:-/tmp/screen.png}"
```

```sh
#!/usr/bin/env bash
# keys.sh "k Down Left" [hold] [wait] -- sends a sequence, then screenshots
hold="${2:-400}"; wait="${3:-1}"
for k in $1; do wtype -P "$k" -s "$hold" -p "$k"; sleep "$wait"; done
sleep 2; grim "${4:-/tmp/screen.png}"
```

Try to identify constructs that the user might want to modify (health, money,
item counts). The debugger has a way to narrow memory locations by searching for
a sequence of values. Lift `repr(C)` Rust structs for these values into the
shim. Also lift the values themselves as static arrays. This allows us to easily
modify the structs embedded in the binary.

Connect to the PCSX2 MCP debugger (the `pcsx2` server, preconfigured in the
container image). With stock PCSX2 only the PINE backend is available: memory
read/write, pattern search, and savestates — no breakpoints, watchpoints, or
disassembly via MCP.

PINE is exposed as a unix socket (`pcsx2.sock` in `$XDG_RUNTIME_DIR`, i.e.
`/tmp/cagert`); bridge it to TCP for the MCP server:

```sh
socat TCP-LISTEN:28011,bind=127.0.0.1,fork,reuseaddr \
  UNIX-CONNECT:/tmp/cagert/pcsx2.sock &
# then MCP pcsx2_connect(mode="pine")
```

A search technique that works well: scan `.bss` for a known on-screen value
(e.g. starting money) to get candidates, then confirm by writing a distinctive
value and watching the screen change.

While reversing, work out which addresses are file-backed: use the ELF program
headers to map each section's virtual address range to its file offset
(`file_off = vaddr - p_vaddr + p_offset` per segment). Anything in
`.data`/`.rodata`/`.sdata` is directly patchable in the boot ELF; anything in
`.bss` only exists at runtime — it cannot be changed by patching the ELF and
needs a code splice or a live debugger write instead. Record which category each
finding falls into.

> Tip: pcsx2-qt doesn't respond to --help or --version.

## Fabricate step

This is the part where we introduce new content into the game. If the user
explicitly asked for changes, then just address them and nothing else.

Otherwise, you are given the open-ended task to generally remaster the game: add
new content, improve existing content, design brand new levels, etc. Be creative
and remember that games should be fun.

Two mechanisms are available:

- **Asset changes**: overwrite files in `/workspace/extracted` directly. Change
  colors, text, upscale textures, etc.
- **Code changes**: use the `resplice` crate to make changes to the game via
  `/workspace/shim`. This splices compiled Rust functions over address ranges in
  the game ELF (`#[Splice(begin, end)]`). Compile the shim with the custom PS2
  target installed alongside this skill. Then call `resplice` on the original
  ELF and the rlib to produce a patched binary.

If you find you need to modify some code that hasn't been reverse-engineered
yet, go back to the Reverse step and do so before continuing. Log every
user-visible change in the `Change log` section of `/workspace/README.md`.

## Repack step

Rebuild a runnable image with `repack_iso.py` (game-agnostic — finds the boot
ELF via SYSTEM.CNF's BOOT2 and walks the ISO9660 tree itself):

```sh
repack_iso.py "/workspace/<original>.iso" /workspace/extracted /workspace/remaster.iso
```

Do NOT rebuild the filesystem with modern mastering tools: `xorriso -as mkisofs`
images do not boot — the PS2 kernel's own ISO9660 walk (sectors 16/48/18) dies
with kernel TLB misses before game code even runs, i.e. the guest FS parser
rejects xorriso's directory layout. Games are also layout-sensitive beyond that:
they typically read assets by raw sector number from LBA tables baked into the
boot ELF.

`repack_iso.py` works around both: it keeps the original image byte-for-byte,
overwrites same-size file changes in place, and _appends_ size-changed files
(including a grown boot ELF) at the end of the image, repointing each one's
ISO9660 directory record and retargeting its LBA-table records inside the boot
ELF (located heuristically: the file's start LBA as an LE u32 with its
2048-block count as a u32 within ±8 bytes). A moved file with no LBA record just
prints a warning — it's assumed to be loaded by name. `--test-move /PATH`
relocates an unchanged file to the appended region; the game booting identically
afterwards proves the retargeting works for it.

To debug repack problems, set `CdvdVerboseReads = true` under `[EmuCore]` plus
`EnableVerbose`/`EnableFileLogging = true` under `[Logging]` in
`/workspace/pcsx2.ini` (and relaunch) to get per-sector read logs in
emulog.txt, and confirm reads land on the expected (e.g. appended) sectors.

## Verify step

Boot `/workspace/remaster.iso` in PCSX2 (via cage) and confirm the game reaches
gameplay: take `grim` screenshots and inspect them, backed up by memory reads at
known-good addresses from the Reverse step. Drive menus and gameplay yourself
with `wtype`. Then verify each entry in the `Change log` actually took effect
in-game — visually via screenshots where possible.

If verification fails, mark Verify as `failed` with the reason in
`/workspace/README.md` and return to the Fabricate or Repack step. The remaster
is done only when Verify passes.

# Environment

You're running in a nix-based docker container. The shebang you should use on
your shell scripts is: `#!/usr/bin/env bash`.
