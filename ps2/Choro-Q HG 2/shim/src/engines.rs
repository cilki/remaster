//! The shop's engine catalogue, spliced back over its own address range.
//!
//! **Generated** by `tools/gen_engines.py`, which reads
//! `0x002eced0..0x002ecff0` out of `SLUS_203.98`. Every numeric field is the
//! shipped value and is asserted against the ELF at generation time, as is the
//! shape of each description block.
//!
//! The one change is the description. A description points at four
//! NUL-terminated line slots that the shop renders in order, and eleven of the
//! twelve engines ship with an **empty first slot** above their " Power N" and
//! " Energy Use N" lines -- so the shop prints a blank row where the blurb
//! should be. Each block below is the shipped one with that slot filled in and
//! every other slot reproduced byte for byte.
//!
//! Careful: this is the list the shop sells under **Engine**. The similarly
//! shaped table in `shipped.rs` is the **Transmission** list -- its stats are
//! gear ratios, not power.

#![allow(clippy::unreadable_literal)]

use resplice_macros::Splice;

use crate::strings::block;
use crate::tables::{EngineEntry, Name};

/// Panther -- shipped block b"\0 Power 180\0 Energy Use 4\0\0",
/// i.e. a blank row above the shop's own Power / Energy Use lines.
pub static ENG_PANTHER: &[u8] = block(b"Cheap and willing\0 Power 180\0 Energy Use 4\0\0");

/// Blue MAX -- shipped block b"\0 Power 220\0 Energy Use 5\0\0",
/// i.e. a blank row above the shop's own Power / Energy Use lines.
pub static ENG_BLUE_MAX: &[u8] = block(b"Smooth midrange\0 Power 220\0 Energy Use 5\0\0");

/// Blue MAX V2 -- shipped block b"\0 Power 260\0 Energy Use 6\0\0",
/// i.e. a blank row above the shop's own Power / Energy Use lines.
pub static ENG_BLUE_MAX_V2: &[u8] = block(b"Stronger Blue MAX\0 Power 260\0 Energy Use 6\0\0");

/// MAD -- shipped block b"\0 Power 290\0 Energy Use 7\0\0",
/// i.e. a blank row above the shop's own Power / Energy Use lines.
pub static ENG_MAD: &[u8] = block(b"Raw power, thirsty\0 Power 290\0 Energy Use 7\0\0");

/// MAD V2 -- shipped block b"\0 Power 330\0 Energy Use 8\0\0",
/// i.e. a blank row above the shop's own Power / Energy Use lines.
pub static ENG_MAD_V2: &[u8] = block(b"More MAD, more fuel\0 Power 330\0 Energy Use 8\0\0");

/// Long MAD -- shipped block b"\0 Power 360\0 Energy Use 6\0\0",
/// i.e. a blank row above the shop's own Power / Energy Use lines.
pub static ENG_LONG_MAD: &[u8] = block(b"Strong and frugal\0 Power 360\0 Energy Use 6\0\0");

/// Black MAX -- shipped block b"\0 Power 390\0 Energy Use 8\0\0",
/// i.e. a blank row above the shop's own Power / Energy Use lines.
pub static ENG_BLACK_MAX: &[u8] = block(b"Race-bred muscle\0 Power 390\0 Energy Use 8\0\0");

/// RS Magnum -- shipped block b"\0 Power 420\0 Energy Use 12\0\0",
/// i.e. a blank row above the shop's own Power / Energy Use lines.
pub static ENG_RS_MAGNUM: &[u8] = block(b"Fast, very thirsty\0 Power 420\0 Energy Use 12\0\0");

/// Speed MAX -- shipped block b"\0 Power 450\0 Energy Use 16\0\0",
/// i.e. a blank row above the shop's own Power / Energy Use lines.
pub static ENG_SPEED_MAX: &[u8] = block(b"Top-tier road power\0 Power 450\0 Energy Use 16\0\0");

/// Hyper MAX -- shipped block b"\0 Power 600\0 Energy Use 20\0\0",
/// i.e. a blank row above the shop's own Power / Energy Use lines.
pub static ENG_HYPER_MAX: &[u8] = block(b"Colossal output\0 Power 600\0 Energy Use 20\0\0");

/// Devil Engine -- shipped block b"\0 Power 3000\0 Energy Use 0\0\0",
/// i.e. a blank row above the shop's own Power / Energy Use lines.
pub static ENG_DEVIL_ENGINE: &[u8] = block(b"Limitless. No fuel.\0 Power 3000\0 Energy Use 0\0\0");

/// Engine catalogue: Normal through Devil Engine.
///
/// Shipped range: `0x002eced0..0x002ecff0` (12 x 24 bytes).
#[Splice(begin = 0x002eced0, end = 0x002ecff0)]
pub static ENGINES: [EngineEntry; 12] = [
    // Normal: left as shipped
    EngineEntry { name: Name::addr(0x00333968), description: Name::addr(0x002ed1d0),
                  price: 200, power: 1500, energy_use: 3, tier: 0 },
    // Panther: blank first line filled
    EngineEntry { name: Name::addr(0x00333980), description: Name::new(ENG_PANTHER),
                  price: 500, power: 1800, energy_use: 4, tier: 0 },
    // Blue MAX: blank first line filled
    EngineEntry { name: Name::addr(0x002ed1a0), description: Name::new(ENG_BLUE_MAX),
                  price: 1000, power: 2200, energy_use: 5, tier: 0 },
    // Blue MAX V2: blank first line filled
    EngineEntry { name: Name::addr(0x002ed170), description: Name::new(ENG_BLUE_MAX_V2),
                  price: 1500, power: 2600, energy_use: 6, tier: 0 },
    // MAD: blank first line filled
    EngineEntry { name: Name::addr(0x00333978), description: Name::new(ENG_MAD),
                  price: 2000, power: 2900, energy_use: 7, tier: 0 },
    // MAD V2: blank first line filled
    EngineEntry { name: Name::addr(0x00333970), description: Name::new(ENG_MAD_V2),
                  price: 4000, power: 3300, energy_use: 8, tier: 0 },
    // Long MAD: blank first line filled
    EngineEntry { name: Name::addr(0x002ed100), description: Name::new(ENG_LONG_MAD),
                  price: 8000, power: 3600, energy_use: 6, tier: 0 },
    // Black MAX: blank first line filled
    EngineEntry { name: Name::addr(0x002ed0d0), description: Name::new(ENG_BLACK_MAX),
                  price: 12000, power: 3900, energy_use: 8, tier: 0 },
    // RS Magnum: blank first line filled
    EngineEntry { name: Name::addr(0x002ed0a0), description: Name::new(ENG_RS_MAGNUM),
                  price: 16000, power: 4200, energy_use: 12, tier: 0 },
    // Speed MAX: blank first line filled
    EngineEntry { name: Name::addr(0x002ed070), description: Name::new(ENG_SPEED_MAX),
                  price: 20000, power: 4500, energy_use: 16, tier: 0 },
    // Hyper MAX: blank first line filled
    EngineEntry { name: Name::addr(0x002ed040), description: Name::new(ENG_HYPER_MAX),
                  price: 80000, power: 6000, energy_use: 20, tier: 1 },
    // Devil Engine: blank first line filled
    EngineEntry { name: Name::addr(0x002ed010), description: Name::new(ENG_DEVIL_ENGINE),
                  price: 160000, power: 30000, energy_use: 0, tier: 2 },
];
