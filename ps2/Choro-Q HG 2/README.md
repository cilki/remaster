## Status

- Extract: done
- Reverse: done for the shop / economy / parts / car layer and the new-game
  initialiser (live debugging working; data *and* code splices proven in-game)
- Fabricate: done
- Repack: done
- Textures: HD texture pack built (see "Texture upscaling"); grows as the
  game is played
- Verify: passing — every change confirmed in the running game; three of seven
  also confirmed on screen (see "What is not verified on screen" below)

## Game

- Title: Road Trip
- Region/ID: SLUS-20398 (SLUS_203.98)
- Version: 1.02, NTSC
- Boot: cdrom0:\SLUS_203.98;1 (from SYSTEM.CNF)
- Image: /workspace/Road Trip.iso (536,670,208 bytes)
- Franchise: internal strings confirm this is a **Choro Q** (Takara) title,
  localized as "Road Trip" (money = "Choro Q Coins", town/shop/garage system,
  `Q##` car-part naming).

## Live debugging

Working, using the standard cage/grim/wtype + PINE-over-socat recipe from the
skill. `pcsx2_connect(mode="pine")` reports "connected (Road Trip)".

## Catalog

Top level: IOPRP234.IMG (IOP reboot image),
LIBSD/MCMAN/MCSERV/PADMAN/SDRDRV/SIO2MAN/SNDMOD .IRX (IOP modules), SLUS_203.98
(main ELF, stripped, MIPS-III, 1.25MB), SYSTEM.CNF.

Asset directories under /workspace/extracted:

- ACTION/ (38M, 21 files A00-A16.BIN) — action/minigame scene data
- CAR0..CAR4/ (5x30 files, ~6M each) — car models/data, Q##.BIN numbered 00-136
  across the 5 dirs
- CARS/ (836K) — PARTS.BIN, Q150.BIN, TIRE.BIN, WHEEL.BIN — shared car parts
- COURSE/ (46M, 19 files C00-C18.BIN, missing C15/C17) — track/course data
- FLD/ (172M, 64 files, 3-digit numbered \*.BIN, e.g. 000-033, 100...) —
  open-world field/map data
- ITEM/ (704K) — METER0-10.GSL (HUD gauge gfx), PARTS.GSL
- SHOP/ (65M) — GARAGE.BIN, T00-T15.BIN — shop/garage textures & models
- SOUND/ (143M) — 1CH/2CH/3CH_L/R.VAG (streamed audio channels), ACTION/BGM
  .TSQ/.TVB (sequenced music)
- SYS/ (17M) — UI/system assets: FONT.GSL, FONTE.GSL, LOGO.GSL, OPTION.BIN,
  OPTION2.GSL, PUTI.BIN/.GSL (mascot char?), POSE.GSL, NOTE.GSL,
  S00-S02.E3D/.GSL (system 3D models), COIN.BIN, EFFECT.GSL, HG2.ICO

.GSL = likely a proprietary texture/graphics archive format (game engine
specific). .E3D = likely a 3D model format. .BIN in COURSE/FLD/ACTION/CAR\*/SHOP
= packed scene/model/texture archives (need further RE to find offsets).

### ELF memory map (from program/section headers)

| Section   | Range              | Notes                                       |
| --------- | ------------------ | ------------------------------------------- |
| `.text`   | 0x200000-0x28e72c  | code, fully stripped                        |
| `.vutext` | 0x28e730-0x29a830  | VU microcode                                |
| `.data`   | 0x29a880-0x2e99b0  | writable globals, prize tables, NPC table   |
| `.rodata` | 0x2e9a00-0x332af0  | **parts tables, shop tables, all dialogue** |
| `.sdata`  | 0x333280-0x33567b  | short strings (part names)                  |
| `.bss`    | 0x335780-0x17874e0 | 21 MB — live game state lives here          |

File offset = `vaddr - 0x29a880 + 0x9b880` for everything from `.data` onward
(`.text` uses `vaddr - 0x200000 + 0x1000`); `elfmap.Img.off()` does this.

## Memory findings

Addresses below are PS2 virtual addresses. Everything at/below 0x332af0 is
file-backed in `SLUS_203.98` and so is directly patchable in the ELF.

| Address        | Type         | Description                                                        |
| -------------- | ------------ | ------------------------------------------------------------------ |
| 0x002eca70     | rec[13] x28B | Tire parts table (name, desc, price, 6 grip stats)                 |
| 0x002ed328     | rec[6] x28B  | Engine/gearbox table (name, desc, price, gear ratios)              |
| 0x002f00a8     | rec[15] x16B | Horn shop table (price, sound id, name)                            |
| 0x002f0190     | char[16][]   | Horn name slots                                                    |
| 0x0029fcc0     | u32[]        | Race prize-money tables, descending by finishing rank              |
| 0x002be398     | char\*[23]   | Track / event name table                                           |
| 0x002a4590     | char\*[104]  | Shop and location names (stride 32)                                |
| 0x0029ace0     | char\*[]     | Main-menu labels, EN/FR/DE interleaved                             |
| 0x002c0724     | rec x16B     | NPC table: name pointer + RGB body colour                          |
| 0x002ed1f8     | rec[5] x16B  | Chassis/weight table (value = weight, lower better)                |
| 0x002ed4e8     | rec[4] x16B  | Steering table (value = response, higher quicker)                  |
| 0x002eff40     | rec[3] x16B  | Lights table                                                       |
| 0x002f02f8     | rec[3] x16B  | Propulsion (None / Propeller / Jet Turbine)                        |
| 0x002f03e0     | rec[9] x20B  | Special options (Water Ski, Flight Wing, Police Light, Billboards) |
| 0x00333940+    | char[8]      | Short part names in .sdata (Normal/Sports/Racing/Wet/HG Wet/Big)   |
| **0x0177fdb4** | **u32**      | **Player money (Choro Q Coins) — confirmed by live write**         |

**Player money — CONFIRMED at `0x0177fdb4` (u32).** Found by scanning `.bss` for
the starting value 1000 (exactly one hit) and confirmed by writing 9999, which
changed the Paint Shop's on-screen total. Lives in `.bss`, so the starting
balance is set by the code splice below.

### The save block, `0x0177f760`

The new-game initialiser pinned the base of the live save block and two fields
inside it:

| Offset  | VA           | Field                                              |
| ------- | ------------ | -------------------------------------------------- |
| `+0x000` | `0x0177f760` | current car's live body colour (`0x00RRGGBB`)      |
| `+0x654` | `0x0177fdb4` | wallet, Choro Q Coins (u32)                        |
| `+0xd70` | `0x017804d0` | per-car paint array, 151 x u32, seeded from the catalogue |

Each save *slot* is 13384 (`0x3448`) bytes; `0x002277b0` clears one by index.

### The car catalogue, `0x002ed720` — 151 x 68 bytes

Found by following the copy loop in the new-game initialiser. Runs contiguously
up to the lights table at `0x2eff40`, which pins the count at exactly 151.

| Offset  | Type       | Field                                                  |
| ------- | ---------- | ------------------------------------------------------ |
| `+0x00` | `char *`   | model name, `"Q001".."Q150"` in `.sdata` (entry 149 has a null name) |
| `+0x04` | `u32`      | default body colour `0x00RRGGBB` — showroom, AI traffic *and* save seed |
| `+0x08` | `f32[15]`  | handling parameters (grip, mass, drive balance, ...)   |

File-backed in `.rodata`, so directly patchable.

### The new-game initialiser, `0x00229fd8..0x0022a04c`

Located by scanning `.text` for `li reg, 1000` (7 candidates) and disassembling
each to find the one storing to the wallet:

```
229ff0:  lui   a3, 0x178          ; a3 = 0x1780000
229ff8:  addiu v0, a3, -2208      ; v0 = 0x177f760  <- save base
229ffc:  li    v1, 1000
22a004:  sw    v1, 1620(v0)       ; 0x177fdb4 = money
```

It wipes both save slots, seeds the wallet, then copies all 151 catalogue
colours into the save's paint array. Reimplemented in Rust in the shim
(`newgame.rs`) — this is the remaster's code splice.

## The .GSL texture format

`.GSL` = "GS list".  Cracked this session; `gsl.py` (global tools, on PATH
alongside its CLIs `gsldump.py`/`gslimport.py`) reads and writes it,
and round-trips all 49 files in the game byte-for-byte.

A `.GSL` is a flat chain of records, each a **0x70-byte header followed by its
payload**, ending in 48 bytes of trailer.  The header is a GS `A+D` packet:
four 128-bit (data, register) pairs, with the register selector as a `u32` at
+0x28, +0x38, +0x48, +0x58 holding exactly `0x50, 0x51, 0x52, 0x53`.  Those
four dwords are the record signature.

| Header offset | GS register | What we take from it            |
| ------------- | ----------- | ------------------------------- |
| +0x20         | `BITBLTBUF` | `SPSM` (bits 56-61) = **format**|
| +0x30         | `TRXPOS`    | destination x/y                 |
| +0x40         | `TRXREG`    | `RRW` bits 0-11, `RRH` bits 32-43 = **width/height** |
| +0x50         | `TRXDIR`    | transfer direction              |

Two traps, both of which cost time before the format made sense:

* **The header is full of garbage.**  The authoring tool ran on Windows and
  left uninitialised memory in every bit the GS ignores, so Win32 stack
  (`0x0012xxxx`) and image-base (`0x0040xxxx`) pointers litter the header.
  `TRXREG` reads as `0x00406180_0012f200` and looks like nonsense until each
  field is masked to its real width, at which point it says `512 x 384`.
  Never compare raw dwords in these headers; mask first.

* **`DPSM` is not the pixel format.**  It reads `PSMCT32` on essentially every
  record, including 4bpp ones.  The *source* format `SPSM` is the honest field.
  With `SPSM` the identity `len(payload) == w * h * bpp / 8` holds for all
  **1142 records in all 49 files**, which is what confirms the layout.

Records are padded to a 16-byte qword.  Indexed records are followed by their
CLUT as a small direct-colour record: 16x16 for a 256-entry 8bpp palette,
16x2 for 4bpp.  256-entry CLUTs are in the GS's CSM1 storage order, where each
32-entry block has its middle two runs of 8 exchanged; `unswizzle_clut()`
undoes that.  16-entry palettes are stored straight and must not be touched.

Inventory: 570 textures / 8.7 Mpx across the 49 `.GSL` files (`SYS/` 483,
`ITEM/` 87) — fonts, HUD meters, logos, menus.  Everything else (the world,
courses and cars) lives inside the `CAR*/FLD/COURSE/SHOP` `.BIN` archives,
whose container format is *not* reversed.

```sh
gsldump.py extracted/SYS/FONTE.GSL /workspace/tex   # -> PNGs
```

## Texture upscaling

The pack lives in `/workspace/textures/SLUS-20398/` (the emulator's
`textures` dir is a symlink to it) and is built by
`upscale.py /workspace/textures/SLUS-20398 [factor]`.

### Why the pack is emulator-side and not in the ISO

The GS has 4 MB of VRAM and the game's display lists carry hard-coded buffer
pointers and texel-unit UVs, so a texture cannot be made *larger* in the ISO
without also moving every allocation and rewriting every UV that samples it.
Raising resolution is therefore done through PCSX2's texture-replacement path,
which is also what makes it cover the `.BIN` archives that were never
reversed: replacement keys on what the GS actually draws, not on a file
format.  The ISO changes from the earlier session are untouched by any of this.

### The loop (this is the fast part)

```sh
retex.sh [factor]     # rebuild replacements from dumps, then F7 in-emulator
```

About 10 s for 3,000 textures, and the result is on screen without rebuilding
the ISO or rebooting the emulator.  `DumpReplaceableTextures` is on, so simply
playing into a new area grows `dumps/`; re-running picks the new ones up.
`ab.sh <prefix>` captures the same frame with replacements off and then on,
which is the only honest way to judge a change.

Hotkeys (bound in the base PCSX2 config): **F7** reload replacements, **F6**
toggle them off/on, F8 screenshot.

`shotf8.sh <out>` captures strictly through PCSX2's own screenshot.  Use it,
not `shot.sh`, for any before/after pair: `shot.sh` falls back between `grim`
and F8, the two render at different sizes, and a pair taken through different
paths cannot honestly be compared.  This bit me twice.

`tools/shots.sh <ab-prefix> <outdir>` builds the labelled side-by-side crops
(this game's crop list, fed to the global `compare.py`).
For a comparison that needs an emulator restart (anything in `PCSX2.ini`), save
a savestate over PINE first and reload it after; a dialogue box on screen
freezes the world and gives pixel-identical framing across the restart.

### Two kernels, chosen per texture

Measured first: checkerboard-dither energy across every world texture peaks at
0.034, so these textures are **not** dithered and a dither remover would have
been wasted work.  What does vary is whether the art is flat or shaded, and the
two want opposite filters — verified by rendering both and looking:

| Content                          | Best kernel | Why the other loses            |
| -------------------------------- | ----------- | ------------------------------ |
| Shaded/photographic (title plate, 255 colours) | Lanczos + unsharp | Scale2x preserves the blockiness inside shaded areas |
| Flat art (signs, HUD, glyphs, <=48 colours)    | Scale2x, blended 35% toward Lanczos | Lanczos rounds off the hard edges the art is made of and haloes them |

`is_flat()` counts unique colours among *opaque* texels only — a sprite's
transparent surround is one RGBA colour but says nothing about the artwork.
The font's glyphs land on the Scale2x side, which is correct: they are 4-colour
hard-edged art, not anti-aliased, and they come out visibly crisper for it.

Lanczos runs in premultiplied alpha.  The game stores an arbitrary colour under
alpha 0, so resampling RGB and A together drags a dark halo into every soft
edge; premultiplying first and dividing it back out afterwards avoids that.

### Coverage

~3,235 textures dumped so far, all but 5 replaced (those are under 8 px),
59 MB at 4x.  The count drifts upward while the game runs.  That covers the intro,
Q's Factory and the parts shop.  **It is not the whole game** — PCSX2 keys
replacements by texture hash, so a texture that has never been drawn cannot be
replaced.  Playing further and re-running
`retex.sh` extends it; nothing else is needed.

## Repack method

Repacked with the skill's `repack_iso.py` (in-place + append-and-retarget-LBA —
see the skill's Repack step). Proven on this game: relocating an untouched
SYS/TITLE.GSL to the appended region boots and renders the title fine, with CDVD
reads observed at the appended sectors (`--test-move /SYS/TITLE.GSL` reproduces
that experiment). Files with no LBA record here — loaded by name, so free to
change size: SYSTEM.CNF, IOPRP234.IMG, \*.IRX, FONT.GSL.

### The LBA table

The game reads assets by raw sector number. All records live in one region of
`.data`, VA 0x29adf4..0x29c01c (file offset 0x9bdf4), ~384 records covering 364
of 376 files, as sub-tables of differing shapes ({lba, blocks, ptr} for SOUND,
{name_ptr, lba, blocks} elsewhere; strides 12/16/24/28), plus a stray LIBSD.IRX
record at VA 0x2a91a0. Records are located layout-agnostically: file's LBA as LE
u32 with its 2048-block count as u32 within ±8 bytes. Files with no record are
loaded by name (FONT.GSL appears unused; FONTE.GSL is the font actually loaded
at boot).

## Build

`build.sh` is the whole pipeline: compile the shim, splice it into a pristine
ELF, and optionally repack.

```sh
./build.sh          # shim -> /workspace/extracted/SLUS_203.98
./build.sh iso      # ... and repack /workspace/remaster.iso
```

`/workspace/orig/SLUS_203.98` is the pristine boot ELF pulled straight from the
original ISO. Splicing always starts from it, never from the previous output,
so builds are idempotent and a mistake is never baked in twice.

### Tooling

The game-agnostic helpers (`elfmap.py`, `dis.sh`, `xref.py`, the `.GSL`
tools, `upscale.py`, `compare.py`) now live in the global tools dir and are on
PATH; pass them the boot ELF, e.g. `dis.sh orig/SLUS_203.98 0x2144b8`.
What remains here is game-specific:

| Path                     | What it does                                            |
| ------------------------ | ------------------------------------------------------- |
| `tools/gen_cars.py`      | regenerates `shim/src/cars.rs` from the ELF             |
| `tools/gen_engines.py`   | regenerates `shim/src/engines.rs` from the ELF          |
| `tools/shots.sh`         | this game's compare crops, fed to the global `compare.py` |
| `shot.sh` / `view.sh`    | frame capture (grim, falling back to PCSX2 F8) + downscale |
| `pine.sh`                | brings up the PINE->TCP bridge for the MCP debugger     |

Both generators re-serialise what they emit and assert it matches
`SLUS_203.98`, so an unintended edit fails the build rather than shipping.

### PS2 memory map, as observed

| Range                   | Contents                                          |
| ----------------------- | ------------------------------------------------- |
| `0x00100000-0x001bffff` | in use — EE kernel / thread structures            |
| `0x001c0000-0x001fffff` | **free** — where the injected segment now lives   |
| `0x00200000-0x0029a848` | `.text` + `.vutext`                               |
| `0x0029a880-0x0033567b` | `.data` / `.rodata` / `.sdata` (file-backed)      |
| `0x00335780-0x017874df` | `.bss`                                            |
| `0x017874e0-0x01ffffff` | game heap, growing up from the end of `.bss`      |

## Change log

Every entry below except the texture pack is spliced from `/workspace/shim`;
nothing is hand-patched.  The texture pack is emulator-side and changes no
bytes in the ISO.

### HD texture pack  *(emulator-side, not in the ISO)*

Every texture the game has drawn so far, upscaled 4x with a per-texture choice
of kernel — Lanczos for shaded art, Scale2x-blend for flat art.  See "Texture
upscaling" above for why it is emulator-side and how the kernels were chosen.

**Verify:** boot the ISO and press **F6** to toggle replacements off and on;
the OSD reports `Replaced: N`.  `ab.sh /tmp/x` captures both states of the same
frame.  Confirmed on screen in Q's Factory: dialogue text, the "Q's factory"
sign and the parts-shop panels are all visibly cleaner with the pack on.

### Starting money: 1000 -> 10,000  *(code splice)*

The one code splice. `newgame.rs` reimplements the new-game initialiser at
`0x229fd8..0x22a04c` in Rust — save-slot wipe, wallet seed, and the 151-entry
car-colour copy — so the starting balance is a constant instead of an
immediate buried in MIPS. 1000 coins buys one Sports part and no tires, which
is why vanilla opens with hours of lap grinding; 10,000 funds a real first
build immediately.

**Verify:** Adventure -> New Game, then any shop — the wallet reads `10000`.
Confirmed on screen in the Parts Shop, and at `0x0177fdb4` over PINE.

### Race prizes rescaled

`prizes.rs`, `0x29fcc0..0x29fd30`. Every prize passes through
`f(x) = 2x` for `x <= 10000`, else `x + 10000`. That map is non-decreasing, so
it cannot reorder a descending run *whatever* the true class boundaries are —
which matters, because those boundaries are inferred. Early and mid-game
payouts double; the 80,000-coin class moves only 12%. A `const` block asserts
both properties at compile time.

**Verify:** finish any race; rank C now pays 800/600/400/200. Confirmed by
PINE read of the live table — all 28 values match the rescale. A payout has
not been watched on screen.

### Engine descriptions: 11 blank rows filled

`engines.rs`, `0x2eced0..0x2ecff0`. Eleven of the twelve engines ship with an
empty first line slot, so the shop prints a **blank row** above "Power" and
"Energy Use". Each is filled with the trade-off the two numbers hide — which
engines are thirsty for their output (RS Magnum draws 12 for 420) and which are
unusually clean (Long MAD draws 6 for 360, less than the cheaper, weaker Black
MAX at 8). Every numeric field, and the Power / Energy lines, are reproduced
byte for byte; the generator asserts the " Power N" text against the record's
own `power` field.

**Verify:** Parts Shop -> Engine. Confirmed on screen: Panther now reads
"Cheap and willing / Power 180 / Energy Use 4".

### Transmission descriptions: "Good engine," replaced

`shipped.rs` `TRANSMISSIONS`, `0x2ed328..0x2ed3d0`. All five upgrades opened
with the same "Good engine," — doubly wrong, since this table is the gearbox,
not the engine (its stats are gear ratios; the engine list is a separate table
at `0x2eced0`). Line 1 now states the gear count and character, e.g.
"6-speed, long legs". Lines 2 and 3, including the `" Gear Ratio****"` star
rating, are reproduced exactly.

**Verify:** Parts Shop -> Transmission. *Not yet seen on screen* — the shop
nearest the start does not stock transmissions. Confirmed in the patched ELF.

### Devil tires: 200,000 -> 60,000

`shipped.rs` `TIRES[12]`. At 200,000 — roughly 200 rank-C wins — the joke item
was unreachable, so the joke never landed. Repriced to sit just above the
50,000 Flight Wing as the last thing left to buy. Its absurd 255.0 grip is
left exactly as shipped: the remaster makes the joke reachable, not smaller.

**Verify:** Parts Shop -> Tires, scroll to Devil. Confirmed at `0x2ecbc8`.

### Horn shop: a ladder instead of a flat 1000

`shipped.rs` `HORNS`. Every horn past the free Air Horn cost exactly 1000, so
the moment a player could afford one they could afford all thirteen and the
shop stopped being a reason to earn money. Now 200 (Bicycle Bell) to 4000
(Train Horn): silly novelty horns cheap enough to buy in passing, showpieces
saved for later. Sound ids untouched — every horn still plays what its name
says.

**Verify:** Horn Shop price list. Confirmed at `0x2f00a8`.

### Fleet recolour: 11 flat greys repainted

`cars.rs`, `0x2ed720..0x2eff3c`. 45 of the 151 cars are greyscale. Whites and
near-blacks read as deliberate choices and are untouched; the eleven flat mid
greys (`#777777`, `#888888` ... `#CCCCCC`) had no character at all and are
repainted into a saturated but period-correct palette — burnt orange, marine
blue, mustard, racing green, violet, rose, teal, apricot, brick, lime, indigo.
All 151 handling blocks are asserted byte-identical, so this is paint only.

Because the colour field also seeds the save's per-car paint array, this shows
up in the showroom, in town traffic, and on a repaint.

**Verify:** confirmed at `0x2ed834` (Q005 now `#E8541E`) and, more tellingly,
at `0x17804d0+16` — the *save block* picked the new colour up, which proves the
code splice is reading the patched table. On-screen confirmation needs one of
those eleven models to appear.

