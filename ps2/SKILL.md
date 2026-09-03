---
name: remaster-ps2
description:
  Remaster a PS2 game end-to-end - extract the ISO, reverse-engineer it with the
  PCSX2 debugger, modify code and assets, repack a runnable ISO, and verify it
  in the emulator.
---

# REmaster skill

Create `/workspace/WORK.md` if it doesn't exist yet. This file tracks the
progress of our work and lets a fresh session resume where the last one left
off. Keep it in this shape:

```markdown
# WORK.md

## Status

- Extract: pending | done
- Reverse: pending | in progress | done
- Fabricate: pending | in progress | done
- Repack: pending | done
- Verify: pending | passed | failed (<reason>)

## Catalog

<file/asset inventory from the Reverse step>

## Memory findings

| Address | Type | Description |
| ------- | ---- | ----------- |

## Change log

<each Fabricate change: what, where, how to verify it in-game>
```

Update the relevant section immediately after finishing work in a step, not at
the end of the session.

## Extract step

If we have a disc image under `/workspace`, extract it to
`/workspace/extracted`. Use `7z x <image> -o/workspace/extracted` for
ISO files. CHD images must first be converted with `chdman extractdvd`; BIN/CUE
can be converted with `7z` or `iat`. Keep the original image untouched — it's
needed again in the Repack step.

This step is complete when `/workspace/extracted` contains `SYSTEM.CNF` and
the boot ELF it names (e.g. `SLUS_123.45`). Do not proceed until then.

### Shim crate

Create the "shim" crate in the workspace using `resplice`. This will be used to
reimplement small portions of the game in rust via binary patching.

The shim must be `#![no_std]` and produce an rlib. The target uses a
conservative MIPS II baseline with soft floats; avoid atomics — the R5900 lacks
LL/SC, so emitted atomic sequences would fault on the real CPU. For quick
experiments, test changes via debugger memory writes before committing a splice
to the ELF.

Temporary: `resplice` is still immature, so attempt to find and patch any bugs
in `resplice` itself.

```sh
cargo build --release -Z build-std=core,alloc -Z json-target-spec \
    --target /root/.claude/skills/remaster-ps2/mipsel-ps2-elf.json
```

## Viewing and controlling the game

Always launch PCSX2 through `cage`, a nested compositor that shows up as a
normal window on the user's desktop while giving you screen capture and keyboard
injection on its own Wayland socket. cage creates `wayland-1` in
`$XDG_RUNTIME_DIR` (`wayland-0` is the host's socket — never capture or type on
that one).

```sh
# Launch PCSX2 in the background:
cage -- pcsx2-qt /workspace/remaster.iso &

# Screenshot what PCSX2 is rendering, then view the PNG with the Read tool:
WAYLAND_DISPLAY=wayland-1 grim /tmp/screen.png

# Send keys into PCSX2 (F8 = native PCSX2 screenshot hotkey):
WAYLAND_DISPLAY=wayland-1 wtype -k F8

# Game input: tap a button, or hold a key for N ms (-P press, -s sleep,
# -p release):
WAYLAND_DISPLAY=wayland-1 wtype -k k                # Cross
WAYLAND_DISPLAY=wayland-1 wtype -P w -s 1000 -p w   # left stick up, 1s
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

If cage fails to start with a renderer error, retry with `WLR_RENDERER=pixman`
as a CPU fallback.

## Reverse step

Search `/workspace/extracted` for the game executable (named by `BOOT2` in
`SYSTEM.CNF`). Catalog the available files and assets in `WORK.md` for future
steps. Look for debug symbols. The final goal of this step is to
reverse-engineer parts of the game that we might want to modify later. Usually
we want to leave the low-level engine code untouched.

### Live debugging

Ask the user if they want to run some live trials where they play the game and
you watch memory locations during execution via the debugger. If the user isn't
available, you can play the game yourself with `wtype` and watch the screen with
`grim` (see "Viewing and controlling the game").

Ask the user to report current in-game values (health, money, item counts) to
locate important structs via pattern search. The debugger has a way to narrow
memory locations by giving a sequence of values. Record every confirmed address
in the `Memory findings` table of `WORK.md` and lift `repr(C)` Rust structs for
them.

Connect to the PCSX2 MCP debugger (the `pcsx2` server, preconfigured in the
container image). With stock PCSX2 only the PINE backend is available: memory
read/write, pattern search, and savestates.

> Tip: pcsx2-qt doesn't respond to --help

## Fabricate step

This is the part where we introduce new content into the game. If the user
explicitly asked for changes, then just address them and nothing else.

Otherwise, you are given the open-ended task to generally remaster the game: add
new content, improve existing content, design brand new levels, etc. Be creative
and remember that games should be fun.

Two mechanisms are available:

- **Asset changes**: overwrite files in `/workspace/extracted` directly.
  Prefer same-size replacements so the Repack step stays simple.
- **Code changes**: use the `resplice` crate, which splices compiled Rust
  functions over address ranges in the game ELF (`#[Splice(begin, end)]`).
  Compile the shim with the custom PS2 target installed alongside this skill:

If you find you need to modify some code that hasn't been reverse-engineered
yet, go back to the Reverse step and do so before continuing. Log every change
in the `Change log` section of `WORK.md`.

## Repack step

Rebuild a runnable image from `/workspace/extracted`:

```sh
xorriso -as mkisofs -iso-level 3 -o /workspace/remaster.iso \
    /workspace/extracted
```

Some games are sensitive to file placement (LBA) on the disc. If the rebuilt ISO
fails to boot in the Verify step but the original does, fall back to patching
the original image in place: for same-size file changes, locate the file's
offset in the original ISO (pattern-search for its first bytes) and overwrite
those bytes directly in a copy of the original image.

## Verify step

Boot `/workspace/remaster.iso` in PCSX2 (via cage, see "Viewing and
controlling the game") and confirm the game reaches gameplay: take `grim`
screenshots and inspect them, backed up by memory reads at known-good addresses
from the Reverse step. Drive menus and gameplay yourself with `wtype`. Then
verify each entry in the `Change log` actually took effect in-game — visually
via screenshots where possible.

If verification fails, mark Verify as `failed` with the reason in `WORK.md` and
return to the Fabricate or Repack step. The remaster is done only when Verify
passes.

# Environment

You're running in a nix-based docker container. The shebang you should use on
your shell scripts is: `#!/usr/bin/env bash`.
