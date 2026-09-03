## Status

- Extract: done
- Reverse: done for the shop/economy/parts layer (live debugging WORKING)
- Fabricate: pending
- Repack: pending
- Verify: pending

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
balance must be set by a code splice or written live.

## Change log

<each Fabricate change: what, where, how to verify it in-game>
