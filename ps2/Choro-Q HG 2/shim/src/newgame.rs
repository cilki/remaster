//! Code splice over the game's "start a new game" initialiser.
//!
//! The shipped routine lives at `0x229fd8..0x22a04c` and does three things:
//! wipe both save slots, seed the player's wallet, and copy every car's default
//! body colour out of the catalogue at [`CAR_TABLE`] into the save block (so a
//! repaint has somewhere to live). It was found by scanning `.text` for
//! `li reg, 1000` and checking which candidate stored to the money address:
//!
//! ```text
//! 229ff0:  lui   a3, 0x178          ; a3 = 0x1780000
//! 229ff8:  addiu v0, a3, -2208      ; v0 = 0x177f760   <- save base
//! 229ffc:  li    v1, 1000
//! 22a004:  sw    v1, 1620(v0)       ; 0x177fdb4 = money
//! ```
//!
//! Reimplemented here so the starting balance is a Rust constant. Everything
//! else is a faithful port -- the wipe and the colour copy are load-bearing, and
//! getting them wrong corrupts a new game rather than failing loudly.

use resplice_macros::Splice;

use crate::tables::{CAR_COUNT, CAR_STRIDE, CAR_TABLE};

/// Base of the live save block in `.bss`.
pub const SAVE_BASE: usize = 0x0177_f760;
/// Byte offset of the wallet (Choro Q Coins) within the save block.
pub const MONEY_OFF: usize = 0x654;
/// Byte offset of the 151-entry per-car paint array within the save block.
pub const PAINT_OFF: usize = 0xd70;

/// Clears one save slot (13384 bytes each); the game calls it for slots 0 and 1.
const CLEAR_SLOT: usize = 0x0022_77b0;

/// Coins a brand-new game starts with.
///
/// Shipped value is 1000, which buys exactly one Sports part and leaves nothing
/// for tires -- the opening hours are a money grind before the game opens up.
/// The remaster funds a genuine first build so the fun starts immediately.
pub const STARTING_MONEY: u32 = 10_000;

/// Reimplementation of the new-game initialiser.
///
/// # Safety
/// Spliced over a known address range and only ever reached through the game's
/// own call site, where the save block is allocated and quiescent.
#[Splice(begin = 0x0022_9fd8, end = 0x0022_a04c)]
pub extern "C" fn new_game_init() {
    unsafe {
        let clear_slot: extern "C" fn(u32) = core::mem::transmute(CLEAR_SLOT);
        clear_slot(0);
        clear_slot(1);

        core::ptr::write_volatile((SAVE_BASE + MONEY_OFF) as *mut u32, STARTING_MONEY);

        // Seed each car's editable paint entry from its catalogue default.
        let mut i = 0usize;
        while i < CAR_COUNT {
            let colour = core::ptr::read_volatile((CAR_TABLE + 4 + i * CAR_STRIDE) as *const u32);
            core::ptr::write_volatile((SAVE_BASE + PAINT_OFF + i * 4) as *mut u32, colour);
            i += 1;
        }

        // The starting car's live colour mirrors catalogue entry 0.
        let first = core::ptr::read_volatile((CAR_TABLE + 4) as *const u32);
        core::ptr::write_volatile(SAVE_BASE as *mut u32, first);
    }
}
