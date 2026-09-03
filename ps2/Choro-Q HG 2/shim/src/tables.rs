//! `repr(C)` definitions for the game data tables recovered during the Reverse
//! step, together with the PS2 virtual addresses they live at.
//!
//! Every address below 0x332af0 is file-backed in `SLUS_203.98`, so each of
//! these tables can equally be edited statically in the ELF or written live
//! through the debugger. Offsets and field widths were confirmed both ways
//! (static parse of the ELF and reads of PCSX2's live memory agree).

use core::ffi::c_char;

/// A pointer stored in the game's tables, as the raw 32-bit PS2 address.
///
/// The tables are spliced back into the ELF verbatim, so pointer fields are
/// modelled as plain addresses rather than `*const c_char`: that keeps the
/// byte layout exact and lets the shipped tables live in `static`s (a raw
/// pointer would not be `Sync`). Use the `*_str` helpers to read one.
pub type Ptr32 = u32;

/// Read a NUL-terminated string out of the game's address space.
///
/// # Safety
/// `addr` must point at a live NUL-terminated string in the loaded game.
#[inline]
pub const unsafe fn cstr(addr: Ptr32) -> *const c_char {
    addr as *const c_char
}

/// Many stat fields are 8.8 fixed point: `256` == 1.0.
pub const FIXED_ONE: u16 = 256;

/// Convert an 8.8 fixed-point stat to whole + fractional hundredths, avoiding
/// floats (the target is soft-float and we want these cheap).
#[inline]
pub const fn fixed_to_centi(v: u16) -> u32 {
    (v as u32 * 100) / FIXED_ONE as u32
}

// ---------------------------------------------------------------------------
// Parts catalogue
// ---------------------------------------------------------------------------

/// One entry in a parts catalogue (tires, engines, ...). 28 bytes.
///
/// The same record shape backs every parts shop list; only the meaning of
/// [`PartEntry::stats`] changes per category.
#[repr(C)]
pub struct PartEntry {
    /// Short display name. Lives in `.sdata` for short names ("Normal",
    /// "Sports") and in `.rodata` for longer ones ("HG Studless").
    pub name: Ptr32,
    /// Blurb shown under the name, e.g. `" Roads **** Off-Road ****"`.
    pub description: Ptr32,
    /// Price in Choro Q Coins.
    pub price: u32,
    /// Eight 8.8 fixed-point stats. How many are meaningful, and what they
    /// mean, depends on the table: tires use the first six (per-surface grip),
    /// engines use the first seven (reverse plus up to six forward gears).
    /// The tail is zero on most rows but not all -- "Big" tires carry a 1 in
    /// the last slot -- so all eight are kept rather than assumed padding.
    pub stats: [u16; 8],
}

impl PartEntry {
    /// Tire reading of [`PartEntry::stats`]: per-surface grip.
    #[inline]
    pub fn grip(&self) -> &TireGrip {
        // Safety: TireGrip is a repr(C) block of the same eight u16s.
        unsafe { &*(self.stats.as_ptr() as *const TireGrip) }
    }

    /// Engine reading of [`PartEntry::stats`]: reverse plus five forward gears.
    #[inline]
    pub fn gearing(&self) -> &EngineGearing {
        unsafe { &*(self.stats.as_ptr() as *const EngineGearing) }
    }
}

/// Tire stat block. Exact per-surface ordering is inferred from the star
/// ratings in each tire's description (Roads / Off-Road / Wet Roads) and from
/// how the values move across the shipped tires; the last two are unconfirmed.
#[repr(C)]
pub struct TireGrip {
    pub road: u16,
    pub off_road: u16,
    pub wet: u16,
    pub unknown_a: u16,
    pub unknown_b: u16,
    pub unknown_c: u16,
    pub spare: [u16; 2],
}

/// Engine stat block: `reverse` is stored as a negative 8.8 value
/// (0xFFA1 == -95, i.e. -0.37), the rest ascend across the five forward gears.
#[repr(C)]
pub struct EngineGearing {
    pub reverse: i16,
    /// Up to six forward gears; unused trailing gears are 0. The shipped
    /// Normal/Sports engines use five, the rest six.
    pub gears: [u16; 6],
    pub spare: u16,
}

/// Tire catalogue: 13 entries.
///
/// Shipped contents (name / price):
/// Normal 200, Sports 1000, Semi-Racing 2000, Racing 5000, HG Racing 10000,
/// Wet 2000, HG Wet 3000, Off-Road 500, HG Off-Road 3000, Studless 1000,
/// HG Studless 3000, Big 5000, Devil 200000. "Devil" has every stat at
/// 0xFF00 (255.0) and is priced far out of normal reach.
pub const TIRE_TABLE: u32 = 0x002e_ca70;
pub const TIRE_COUNT: usize = 13;

/// Engine / gearbox catalogue: 6 entries.
///
/// Shipped contents: Normal 200, Sports 1000, Power 2000, Speed 4000,
/// Wide 7000, Hyper 10000.
pub const ENGINE_TABLE: u32 = 0x002e_d328;
pub const ENGINE_COUNT: usize = 6;

/// # Safety
/// Valid only while running inside the game with the ELF loaded at its
/// link-time addresses.
#[inline]
pub unsafe fn tires() -> &'static [PartEntry] {
    core::slice::from_raw_parts(TIRE_TABLE as *const PartEntry, TIRE_COUNT)
}

/// # Safety
/// See [`tires`].
#[inline]
pub unsafe fn engines() -> &'static [PartEntry] {
    core::slice::from_raw_parts(ENGINE_TABLE as *const PartEntry, ENGINE_COUNT)
}

// ---------------------------------------------------------------------------
// Horn shop
// ---------------------------------------------------------------------------

/// One horn in the horn shop list. 16 bytes.
#[repr(C)]
pub struct HornEntry {
    /// Price in Choro Q Coins. Air Horn ships at 0, every other horn at 1000.
    pub price: u32,
    /// Sound id. Shipped ids step by 2 from 0x3c ("Air Horn") to 0x56
    /// ("Train Horn").
    pub sound_id: u32,
    /// Name, stored in a 16-byte fixed slot in the block at [`HORN_NAMES`].
    pub name: Ptr32,
    /// Points at an empty string in `.sdata` (0x333e70) for every shipped
    /// entry, so its role is unconfirmed — most likely an unused description.
    pub description: Ptr32,
}

/// Horn shop table: 14 entries (ids 0x3c..0x56 stepping by 2).
///
/// The name block at [`HORN_NAMES`] holds 15 slots; the last, "Normal Horn",
/// is the default horn and is not referenced by any table row. The 8 bytes
/// immediately after the table (0x2f0188: price 1000, id 0x58) look like a
/// truncated 15th header but its name field is string data, so the table
/// genuinely ends at 14.
pub const HORN_TABLE: u32 = 0x002f_00a8;
pub const HORN_COUNT: usize = 14;
/// Block of 16-byte fixed-width name slots the horn table points into.
pub const HORN_NAMES: u32 = 0x002f_0190;
/// Width of one slot in [`HORN_NAMES`]; names must stay NUL-terminated within it.
pub const HORN_NAME_SLOT: usize = 16;

/// # Safety
/// See [`tires`].
#[inline]
pub unsafe fn horns() -> &'static [HornEntry] {
    core::slice::from_raw_parts(HORN_TABLE as *const HornEntry, HORN_COUNT)
}

// ---------------------------------------------------------------------------
// Town NPCs
// ---------------------------------------------------------------------------

/// A townsfolk / shop-staff car definition. 16 bytes.
#[repr(C)]
pub struct NpcEntry {
    /// Speaker name shown in the dialogue box ("Parts Shop Staff",
    /// "Bartender", "Policeman", "Kevin's mom", ...).
    pub name: Ptr32,
    /// Inferred: 24-bit RGB body colour (e.g. 0x02e01e green for Parts Shop
    /// Staff). Consistent across the table but not yet confirmed on screen.
    pub body_rgb: u32,
    /// Inferred: car model id selecting which `CAR*/Q##.BIN` body is used.
    pub car_id: u32,
    /// Mostly zero; 0x20 on some entries. Purpose unknown.
    pub flags: u32,
}

/// First of several per-town NPC tables in `.data`. Each town has its own run
/// of these records; this is the one beginning with "Q's Factory Staff".
pub const NPC_TABLE: u32 = 0x002c_0724;

// ---------------------------------------------------------------------------
// Name / label tables
// ---------------------------------------------------------------------------

/// Track and event names, a flat `char*[23]`: Peach Raceway, Peach Raceway II,
/// Temple Raceway, ... Drag Race, Roulette, Rainbow Jump, Ski Jumping.
pub const TRACK_NAMES: u32 = 0x002b_e398;
pub const TRACK_NAME_COUNT: usize = 23;

/// Shop and location names, `char*` at stride 32, 104 entries
/// (Parts Shop, Recycle Shop, Quick-Pic Shop No.1 ...).
pub const SHOP_NAMES: u32 = 0x002a_4590;
pub const SHOP_NAME_COUNT: usize = 104;
pub const SHOP_NAME_STRIDE: usize = 32;

/// Main-menu labels. English, French and German variants are interleaved in
/// one array, so any edit must be applied per language.
pub const MENU_LABELS: u32 = 0x0029_ace0;

/// Race prize money: consecutive descending `u32` runs, one per race class,
/// indexed by finishing position. Shipped runs include
/// `400/300/200/100`, `1500/1200/1000/800/600/500` and
/// `80000/60000/40000/30000/20000/10000`.
pub const PRIZE_TABLES: u32 = 0x0029_fcc0;

/// # Safety
/// See [`tires`].
#[inline]
pub unsafe fn track_names() -> &'static [*const c_char] {
    core::slice::from_raw_parts(TRACK_NAMES as *const *const c_char, TRACK_NAME_COUNT)
}

// ---------------------------------------------------------------------------
// Layout guards
// ---------------------------------------------------------------------------
// These sizes are the record strides actually observed in the ELF; if a future
// edit changes a field width the build fails here rather than silently reading
// the tables at the wrong stride.

const _: () = assert!(core::mem::size_of::<PartEntry>() == 28);
const _: () = assert!(core::mem::size_of::<HornEntry>() == 16);
const _: () = assert!(core::mem::size_of::<NpcEntry>() == 16);
const _: () = assert!(core::mem::size_of::<TireGrip>() == 16);
const _: () = assert!(core::mem::size_of::<EngineGearing>() == 16);

// ---------------------------------------------------------------------------
// Remaining parts categories
// ---------------------------------------------------------------------------
// Every parts table shares the prefix {name, description, price}; only the
// stat block after it differs, which is why three record types cover all
// seven catalogues.

/// A parts entry carrying a single scalar/bitmask stat. 16 bytes.
///
/// Used by the chassis, steering, lights and propulsion catalogues. The
/// meaning of [`SimplePart::value`] is per-category — see the table consts.
#[repr(C)]
pub struct SimplePart {
    pub name: Ptr32,
    pub description: Ptr32,
    pub price: u32,
    pub value: u32,
}

/// A bolt-on option (water ski, wing, police light, billboard). 20 bytes.
#[repr(C)]
pub struct OptionPart {
    pub name: Ptr32,
    pub description: Ptr32,
    pub price: u32,
    /// Inferred: equip bitmask, consistently `1 << part_id` in the shipped rows.
    pub equip_mask: u16,
    /// Inferred: option slot id.
    pub part_id: u16,
    /// Variant index — 0..4 across the five "Billboard" rows.
    pub variant: u16,
    pub reserved: u16,
}

/// Chassis / weight catalogue: 5 entries. [`SimplePart::value`] is weight, so
/// **lower is better** and it descends as price rises.
/// Normal 200/25, Light 500/22, Feather 1000/20, Phantom 2000/18, Hyper 4000/15.
pub const CHASSIS_TABLE: u32 = 0x002e_d1f8;
pub const CHASSIS_COUNT: usize = 5;

/// Steering catalogue: 4 entries. [`SimplePart::value`] is steering response,
/// higher is quicker. Normal 200/64, Quick 500/96, X2 Quick 1000/128,
/// X3 Quick 2000/160.
pub const STEERING_TABLE: u32 = 0x002e_d4e8;
pub const STEERING_COUNT: usize = 4;

/// Lights catalogue: 3 entries — Headlights (free), Fog Lights 500,
/// Beam Lights 500. Both paid rows carry `value == 0x00040010`, which looks
/// like a packed `{u16 mask, u16 id}` rather than a scalar.
pub const LIGHTS_TABLE: u32 = 0x002e_ff40;
pub const LIGHTS_COUNT: usize = 3;

/// Propulsion catalogue: 3 entries — None (free), Propeller 3000,
/// Jet Turbine 10000. `value` is 64 and 8192 respectively; bitmask-like.
pub const PROPULSION_TABLE: u32 = 0x002f_02f8;
pub const PROPULSION_COUNT: usize = 3;

/// Special options: 9 entries, of which five are "Billboard" variants.
/// None 0, **Water Ski 3000** ("Drive on water."),
/// **Flight Wing 50000** ("Fly at 190mph."), Police Light 1000
/// ("Has no effect."), Billboard x5 (free).
pub const OPTIONS_TABLE: u32 = 0x002f_03e0;
pub const OPTIONS_COUNT: usize = 9;

/// # Safety
/// See [`tires`].
#[inline]
pub unsafe fn chassis() -> &'static [SimplePart] {
    core::slice::from_raw_parts(CHASSIS_TABLE as *const SimplePart, CHASSIS_COUNT)
}

/// # Safety
/// See [`tires`].
#[inline]
pub unsafe fn steering() -> &'static [SimplePart] {
    core::slice::from_raw_parts(STEERING_TABLE as *const SimplePart, STEERING_COUNT)
}

/// # Safety
/// See [`tires`].
#[inline]
pub unsafe fn options() -> &'static [OptionPart] {
    core::slice::from_raw_parts(OPTIONS_TABLE as *const OptionPart, OPTIONS_COUNT)
}

// ---------------------------------------------------------------------------
// Live game state
// ---------------------------------------------------------------------------

/// The player's Choro Q Coin balance, as a `u32` in `.bss`.
///
/// **Confirmed live**: writing 9999 here changed the on-screen total in the
/// Paint Shop from 1000 to 9999. Because it lives in `.bss` it is *not*
/// file-backed, so the starting balance cannot be changed by editing the ELF
/// at this address — it has to be set by code (a splice) or written live.
pub const MONEY: u32 = 0x0177_fdb4;

/// # Safety
/// Valid only while a game is loaded.
#[inline]
pub unsafe fn money() -> *mut u32 {
    MONEY as *mut u32
}

const _: () = assert!(core::mem::size_of::<SimplePart>() == 16);
const _: () = assert!(core::mem::size_of::<OptionPart>() == 20);
