---
name: remaster-ps2
description: Remaster a PS2 game end-to-end - extract the ISO, reverse-engineer it with the PCSX2 debugger, modify code and assets, repack a runnable ISO, and verify it in the emulator.
---

# REmaster skill

Create a `WORK.md` if it doesn't exist yet. This file tracks the progress of
our work and lets a fresh session resume where the last one left off. Keep it
in this shape:

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
|---------|------|-------------|

## Change log
<each Fabricate change: what, where, how to verify it in-game>
```

Update the relevant section immediately after finishing work in a step, not at
the end of the session.

## Extract step

If we have a disc image under `./workspace`, extract it to `./workspace/game`.
Use `7z x <image> -o./workspace/game` for ISO files. CHD images must first be
converted with `chdman extractdvd`; BIN/CUE can be converted with `7z` or
`iat`. Keep the original image untouched — it's needed again in the Repack
step.

This step is complete when `./workspace/game` contains `SYSTEM.CNF` and the
boot ELF it names (e.g. `SLUS_123.45`). Do not proceed until then.

## Reverse step

Search `./workspace/game` for the game executable (named by `BOOT2` in
`SYSTEM.CNF`). Catalog the available files and assets in `WORK.md` for future
steps. The final goal of this step is to reverse-engineer parts of the game
that we might want to modify later. Usually we want to leave the low-level
engine code untouched.

Connect to the PCSX2 MCP debugger (the `pcsx2` server in `.mcp.json`). With
stock PCSX2 only the PINE backend is available: memory read/write, pattern
search, and savestates. Full debugging (breakpoints, disassembly, registers)
requires a PCSX2 build with the pcsx2-mcp DebugServer patch.

If the user is present, ask them to play the game so you can watch memory
during a real execution, and ask for in-game values (health, money, item
counts) to locate structs via pattern search. If working autonomously, boot
the game, use savestates to capture memory at known points, and diff snapshots
to find changing values. Record every confirmed address in the `Memory
findings` table of `WORK.md` and create `repr(C)` Rust structs for them.

## Fabricate step

This is the part where we introduce new content into the game. If the user
explicitly asked for changes, then just address them and nothing else.

Otherwise, you are given the open-ended task to generally remaster the game:
add new content, improve existing content, design brand new levels, etc. Be
creative and remember that games should be fun.

Two mechanisms are available:

- **Asset changes**: overwrite files in `./workspace/game` directly. Prefer
  same-size replacements so the Repack step stays simple.
- **Code changes**: use the `resplice` crate, which splices compiled Rust
  functions over address ranges in the game ELF (`#[Splice(begin, end)]`).
  Compile the shim with the custom PS2 target installed at
  `/workspace/mipsel-ps2-elf.json`:

  ```sh
  cargo build --release -Z build-std=core,alloc -Z json-target-spec \
      --target /workspace/mipsel-ps2-elf.json
  ```

  The shim must be `#![no_std]`. The target uses a conservative MIPS II
  baseline with soft floats; avoid atomics — the R5900 lacks LL/SC, so
  emitted atomic sequences would fault on the real CPU. For quick
  experiments, test changes via debugger memory writes before committing a
  splice to the ELF.

If you find you need to modify some code that hasn't been reverse-engineered
yet, go back to the Reverse step and do so before continuing. Log every change
in the `Change log` section of `WORK.md`.

## Repack step

Rebuild a runnable image from `./workspace/game`:

```sh
xorriso -as mkisofs -iso-level 3 -o ./workspace/remaster.iso ./workspace/game
```

Some games are sensitive to file placement (LBA) on the disc. If the rebuilt
ISO fails to boot in the Verify step but the original does, fall back to
patching the original image in place: for same-size file changes, locate the
file's offset in the original ISO (pattern-search for its first bytes) and
overwrite those bytes directly in a copy of the original image.

## Verify step

Boot `./workspace/remaster.iso` in PCSX2 and confirm through the MCP debugger
that the game reaches gameplay (use memory reads at known-good addresses from
the Reverse step, or ask the user to confirm visually). Then verify each entry
in the `Change log` actually took effect in-game.

If verification fails, mark Verify as `failed` with the reason in `WORK.md`
and return to the Fabricate or Repack step. The remaster is done only when
Verify passes.
