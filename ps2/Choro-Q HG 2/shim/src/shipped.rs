//! Byte-exact snapshot of the shipped data tables, spliced back over their own
//! addresses.
//!
//! **This file is generated** by `tools/re/gen_shipped.py`, which re-serialises
//! everything it emits and asserts the bytes match `SLUS_203.98`. As written,
//! every splice here is a no-op: it writes the table back exactly as shipped.
//! That is the point -- it gives a place to change a price, a stat or a gear
//! ratio as ordinary Rust, instead of hand-patching bytes.
//!
//! Pointer fields are raw 32-bit addresses of strings already in the ELF.
//! Repointing one at new text means the text has to exist somewhere; editing a
//! name in place is usually easier, and for horns the name slots are a fixed
//! 16 bytes (see `tables::HORN_NAME_SLOT`).

#![allow(clippy::unreadable_literal)]

use resplice_macros::Splice;

use crate::strings;
use crate::tables::{HornEntry, Name, OptionPart, PartEntry, SimplePart};

/// Tire catalogue. `stats` are per-surface grip in 8.8 fixed point.
///
/// Shipped range: `0x002eca70..0x002ecbdc` (13 x 28 bytes).
#[Splice(begin = 0x002eca70, end = 0x002ecbdc)]
pub static TIRES: [PartEntry; 13] = [
    // Normal: Standard
    PartEntry { name: Name::addr(0x00333968), description: Name::addr(0x002ecea8), price: 200, stats: [196, 168, 128, 168, 96, 64, 0, 0] },
    // Sports: All-around
    PartEntry { name: Name::addr(0x00333960), description: Name::addr(0x002ece80), price: 1000, stats: [240, 196, 154, 160, 96, 64, 0, 0] },
    // Semi-Racing: Road racing type
    PartEntry { name: Name::addr(0x002ece70), description: Name::addr(0x002ece40), price: 2000, stats: [333, 168, 154, 140, 80, 64, 0, 0] },
    // Racing: Medium Tread
    PartEntry { name: Name::addr(0x00333958), description: Name::addr(0x002ece18), price: 5000, stats: [512, 102, 102, 100, 51, 64, 0, 0] },
    // HG Racing: Great on raceway
    PartEntry { name: Name::addr(0x002ece08), description: Name::addr(0x002ecdd8), price: 10000, stats: [512, 160, 102, 128, 51, 64, 0, 0] },
    // Wet: Water Durable
    PartEntry { name: Name::addr(0x00333950), description: Name::addr(0x002ecda0), price: 2000, stats: [240, 196, 196, 160, 128, 64, 0, 0] },
    // HG Wet: Super Water Durable
    PartEntry { name: Name::addr(0x00333948), description: Name::addr(0x002ecd60), price: 3000, stats: [240, 196, 240, 160, 128, 64, 0, 0] },
    // Off-Road: Off-road type
    PartEntry { name: Name::addr(0x002ecd50), description: Name::addr(0x002ecd18), price: 500, stats: [196, 196, 160, 160, 128, 64, 0, 0] },
    // HG Off-Road: Mountain ride
    PartEntry { name: Name::addr(0x002ecd08), description: Name::addr(0x002eccd0), price: 3000, stats: [240, 240, 160, 180, 168, 128, 0, 0] },
    // Studless: Ice Durable
    PartEntry { name: Name::addr(0x002eccc0), description: Name::addr(0x002ecc88), price: 1000, stats: [196, 196, 160, 168, 168, 196, 0, 0] },
    // HG Studless: Ice and Land Durable
    PartEntry { name: Name::addr(0x002ecc78), description: Name::addr(0x002ecc38), price: 3000, stats: [240, 196, 160, 168, 196, 196, 0, 0] },
    // Big: Drive everywhere
    PartEntry { name: Name::addr(0x00333940), description: Name::addr(0x002ecbf8), price: 5000, stats: [240, 240, 230, 210, 196, 96, 0, 1] },
    // Devil: Unimaginable! Shipped at 200000 -- roughly 200 rank-C wins, so
    // effectively unreachable. Repriced to sit just above the 50000 Flight
    // Wing as the last thing left to buy. Stats are left exactly as shipped:
    // its absurd 255.0 grip is the joke, and the remaster only makes the
    // joke reachable.
    PartEntry { name: Name::addr(0x00333938), description: Name::addr(0x002ecbe0), price: 60000, stats: [65280, 65280, 65280, 65280, 65280, 65280, 0, 0] },
];

/// Transmission catalogue -- the shop's "Transmission" list, not its "Engine"
/// list. `stats[0]` is reverse (negative), the rest are forward gears.
/// Descriptions come from `strings`: the shipped table gives all five upgrades
/// the same "Good engine," blurb.
///
/// Shipped range: `0x002ed328..0x002ed3d0` (6 x 28 bytes).
#[Splice(begin = 0x002ed328, end = 0x002ed3d0)]
pub static TRANSMISSIONS: [PartEntry; 6] = [
    // Normal: Standard
    PartEntry { name: Name::addr(0x00333968), description: Name::addr(0x002ed4d0), price: 200, stats: [65441, 116, 162, 227, 318, 446, 0, 0] },
    // Sports: Good engine,
    PartEntry { name: Name::addr(0x00333960), description: Name::new(strings::TRANS_SPORTS), price: 1000, stats: [65441, 116, 182, 291, 408, 490, 0, 0] },
    // Power: Good engine,
    PartEntry { name: Name::addr(0x003339b8), description: Name::new(strings::TRANS_POWER), price: 2000, stats: [65441, 128, 220, 276, 387, 464, 557, 0] },
    // Speed: Good engine,
    PartEntry { name: Name::addr(0x003339b0), description: Name::new(strings::TRANS_SPEED), price: 4000, stats: [65441, 128, 260, 327, 458, 550, 660, 0] },
    // Wide: Good engine,
    PartEntry { name: Name::addr(0x003339a8), description: Name::new(strings::TRANS_WIDE), price: 7000, stats: [65441, 144, 300, 414, 539, 647, 711, 0] },
    // Hyper: Good engine,
    PartEntry { name: Name::addr(0x00333988), description: Name::new(strings::TRANS_HYPER), price: 10000, stats: [65441, 156, 350, 446, 550, 625, 750, 0] },
];

/// Chassis catalogue. `value` is weight, so lower is better.
///
/// Shipped range: `0x002ed1f8..0x002ed248` (5 x 16 bytes).
#[Splice(begin = 0x002ed1f8, end = 0x002ed248)]
pub static CHASSIS: [SimplePart; 5] = [
    // Normal: Standard
    SimplePart { name: Name::addr(0x00333968), description: Name::addr(0x002ed308), price: 200, value: 25 },
    // Light: Good top-speed
    SimplePart { name: Name::addr(0x003339a0), description: Name::addr(0x002ed2d8), price: 500, value: 22 },
    // Feather: Better top-speed
    SimplePart { name: Name::addr(0x00333998), description: Name::addr(0x002ed2a8), price: 1000, value: 20 },
    // Phantom: Best top-speed
    SimplePart { name: Name::addr(0x00333990), description: Name::addr(0x002ed278), price: 2000, value: 18 },
    // Hyper: Super top-speed
    SimplePart { name: Name::addr(0x00333988), description: Name::addr(0x002ed248), price: 4000, value: 15 },
];

/// Steering catalogue. `value` is response, higher is quicker.
///
/// Descriptions are left as shipped: line 1 repeats "Steer quickly" across all
/// three, but line 2 already gives the ratio ("1.5 times normal"), so there is
/// nothing here a rewrite would add.
///
/// Shipped range: `0x002ed4e8..0x002ed528` (4 x 16 bytes).
#[Splice(begin = 0x002ed4e8, end = 0x002ed528)]
pub static STEERING: [SimplePart; 4] = [
    // Normal: Standard
    SimplePart { name: Name::addr(0x00333968), description: Name::addr(0x002ed5c0), price: 200, value: 64 },
    // Quick: Steer quickly
    SimplePart { name: Name::addr(0x003339c0), description: Name::addr(0x002ed598), price: 500, value: 96 },
    // X2 Quick: Steer quickly
    SimplePart { name: Name::addr(0x002ed588), description: Name::addr(0x002ed568), price: 1000, value: 128 },
    // X3 Quick: Steer quickly
    SimplePart { name: Name::addr(0x002ed558), description: Name::addr(0x002ed528), price: 2000, value: 160 },
];

/// Lights catalogue.
///
/// Shipped range: `0x002eff40..0x002eff70` (3 x 16 bytes).
#[Splice(begin = 0x002eff40, end = 0x002eff70)]
pub static LIGHTS: [SimplePart; 3] = [
    // Headlights: Standard lights
    SimplePart { name: Name::addr(0x002effc8), description: Name::addr(0x002effb0), price: 0, value: 0 },
    // Fog Lights: Wide beam
    SimplePart { name: Name::addr(0x002effa0), description: Name::addr(0x002eff90), price: 500, value: 262160 },
    // Beam Lights: Hi-Beam
    SimplePart { name: Name::addr(0x002eff80), description: Name::addr(0x002eff70), price: 500, value: 262160 },
];

/// Propulsion catalogue: None / Propeller / Jet Turbine.
///
/// Shipped range: `0x002f02f8..0x002f0328` (3 x 16 bytes).
#[Splice(begin = 0x002f02f8, end = 0x002f0328)]
pub static PROPULSION: [SimplePart; 3] = [
    // None: Propulsion device
    SimplePart { name: Name::addr(0x00333ee8), description: Name::addr(0x002f03c0), price: 0, value: 0 },
    // Propeller: Works with engine
    SimplePart { name: Name::addr(0x002f03b0), description: Name::addr(0x002f0378), price: 3000, value: 64 },
    // Jet Turbine: Speeds up even
    SimplePart { name: Name::addr(0x002f0368), description: Name::addr(0x002f0328), price: 10000, value: 8192 },
];

/// Bolt-on options, including Water Ski and the 50000-coin Flight Wing.
///
/// Shipped range: `0x002f03e0..0x002f0494` (9 x 20 bytes).
#[Splice(begin = 0x002f03e0, end = 0x002f0494)]
pub static OPTIONS: [OptionPart; 9] = [
    // None: Optional parts
    OptionPart { name: Name::addr(0x00333ee8), description: Name::addr(0x002f06a0), price: 0, equip_mask: 0, part_id: 0, variant: 0, reserved: 0 },
    // Water Ski: Drive on water.
    OptionPart { name: Name::addr(0x002f0690), description: Name::addr(0x002f0648), price: 3000, equip_mask: 256, part_id: 8, variant: 0, reserved: 0 },
    // Flight Wing: Fly at 190mph.
    OptionPart { name: Name::addr(0x002f0638), description: Name::addr(0x002f0610), price: 50000, equip_mask: 4, part_id: 2, variant: 0, reserved: 0 },
    // Police Light: Has no effect.
    OptionPart { name: Name::addr(0x002f0600), description: Name::addr(0x002f05d8), price: 1000, equip_mask: 32, part_id: 5, variant: 0, reserved: 0 },
    // Billboard: Post it up to
    OptionPart { name: Name::addr(0x002f05c8), description: Name::addr(0x002f0590), price: 0, equip_mask: 512, part_id: 9, variant: 0, reserved: 0 },
    // Billboard: Post it up to
    OptionPart { name: Name::addr(0x002f05c8), description: Name::addr(0x002f0550), price: 0, equip_mask: 512, part_id: 9, variant: 1, reserved: 0 },
    // Billboard: Post it up to
    OptionPart { name: Name::addr(0x002f05c8), description: Name::addr(0x002f0518), price: 0, equip_mask: 512, part_id: 9, variant: 2, reserved: 0 },
    // Billboard: Post it up to
    OptionPart { name: Name::addr(0x002f05c8), description: Name::addr(0x002f04d8), price: 0, equip_mask: 512, part_id: 9, variant: 3, reserved: 0 },
    // Billboard: Post it up to
    OptionPart { name: Name::addr(0x002f05c8), description: Name::addr(0x002f0498), price: 0, equip_mask: 512, part_id: 9, variant: 4, reserved: 0 },
];

/// Horn shop. Note this table's field order is price-first, unlike the parts
/// tables.
///
/// Every shipped horn past the free Air Horn costs a flat 1000, so once a
/// player can afford one they can afford all thirteen and the shop stops being
/// a reason to earn money. The remaster spreads them into a ladder -- silly
/// novelty horns cheap enough to buy on the way past, the showpieces saved for
/// later -- turning the shop into a collection that lasts the whole game.
/// Sound ids are untouched, so every horn still plays what its name says.
///
/// Shipped range: `0x002f00a8..0x002f0188` (14 x 16 bytes).
#[Splice(begin = 0x002f00a8, end = 0x002f0188)]
pub static HORNS: [HornEntry; 14] = [
    // Air Horn (sound id 0x3c)
    HornEntry { price: 0, sound_id: 0x3c, name: Name::addr(0x002f0260), description: Name::addr(0x00333e70) },
    // Echo Air Horn (sound id 0x3e)
    HornEntry { price: 400, sound_id: 0x3e, name: Name::addr(0x002f0250), description: Name::addr(0x00333e70) },
    // Bus Horn (sound id 0x40)
    HornEntry { price: 1200, sound_id: 0x40, name: Name::addr(0x002f0240), description: Name::addr(0x00333e70) },
    // Bicycle Bell (sound id 0x42)
    HornEntry { price: 200, sound_id: 0x42, name: Name::addr(0x002f0230), description: Name::addr(0x00333e70) },
    // Venus Horn (sound id 0x44)
    HornEntry { price: 1500, sound_id: 0x44, name: Name::addr(0x002f0220), description: Name::addr(0x00333e70) },
    // Chicken Horn (sound id 0x46)
    HornEntry { price: 300, sound_id: 0x46, name: Name::addr(0x002f0210), description: Name::addr(0x00333e70) },
    // Fantasy Horn (sound id 0x48)
    HornEntry { price: 2000, sound_id: 0x48, name: Name::addr(0x002f0200), description: Name::addr(0x00333e70) },
    // Trumpet Horn (sound id 0x4a)
    HornEntry { price: 1800, sound_id: 0x4a, name: Name::addr(0x002f01f0), description: Name::addr(0x00333e70) },
    // Christmas Horn (sound id 0x4c)
    HornEntry { price: 2500, sound_id: 0x4c, name: Name::addr(0x002f01e0), description: Name::addr(0x00333e70) },
    // Duck Horn (sound id 0x4e)
    HornEntry { price: 500, sound_id: 0x4e, name: Name::addr(0x002f01d0), description: Name::addr(0x00333e70) },
    // Space Horn (sound id 0x50)
    HornEntry { price: 3000, sound_id: 0x50, name: Name::addr(0x002f01c0), description: Name::addr(0x00333e70) },
    // Horse Horn (sound id 0x52)
    HornEntry { price: 800, sound_id: 0x52, name: Name::addr(0x002f01b0), description: Name::addr(0x00333e70) },
    // Baby Horn (sound id 0x54)
    HornEntry { price: 600, sound_id: 0x54, name: Name::addr(0x002f01a0), description: Name::addr(0x00333e70) },
    // Train Horn (sound id 0x56)
    HornEntry { price: 4000, sound_id: 0x56, name: Name::addr(0x002f0190), description: Name::addr(0x00333e70) },
];
