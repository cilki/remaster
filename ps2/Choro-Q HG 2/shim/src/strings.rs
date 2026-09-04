//! Replacement shop text, injected by the shim.
//!
//! ## The description format
//!
//! A description pointer does not address a single string: it addresses a
//! *block* of consecutive NUL-terminated lines, and the renderer keeps
//! printing lines until it reaches an empty one. So a block is terminated by a
//! **double NUL**, and a replacement that ends in a single NUL makes the shop
//! run straight on into whatever bytes follow it. That is not theoretical --
//! an earlier build of this shim did exactly that and the Steering panel
//! rendered three descriptions plus a fragment of a Rust source path.
//!
//! The shipped blocks are richer than their first lines suggest:
//!
//! ```text
//! Quick steering:  "Steer quickly"  / "1.5 times normal" / ""
//! Light chassis:   "Good top-speed" / "and acceleration" / " Weight ****"
//! Sports gearbox:  "Good engine,"   / "better top-speed" / " Gear Ratio**"
//! Panther engine:  ""               / " Power 180"       / " Energy Use 4"
//! ```
//!
//! Those second and third lines carry the real information -- the ratio, the
//! star rating -- so replacing a block with one tidy line *destroys* content.
//! Steering and chassis are therefore left exactly as shipped: their text
//! already says what the part does.
//!
//! What is left are the two places where the game genuinely tells the player
//! nothing:
//!
//! * **Engines.** Eleven of the twelve ship with an empty *first* slot, so the
//!   shop prints a blank row above "Power" and "Energy Use". Filling it
//!   destroys nothing and adds the trade-off the two numbers hide.
//! * **Transmissions.** All five upgrades open with the same "Good engine," --
//!   which is doubly wrong, since this is the gearbox, not the engine. Only
//!   the first line is rewritten; the shipped second line and the
//!   " Gear Ratio***" star rating are reproduced exactly.
//!
//! The engine blocks live in `engines.rs`, generated from the shipped bytes.
//!
//! `Name::new` turns each block into an `R_MIPS_32` relocation, which resplice
//! resolves by injecting the bytes into the ELF as a new segment and
//! repointing the table field at them.

/// Number of line slots the shop reads out of a description block.
///
/// Every shipped block across every category is four slots wide, and the
/// runaway above rendered exactly four lines, which pins the count.
pub const SLOTS: usize = 4;

/// Compile-time guard for one description block.
///
/// Checks the two things that actually broke: that the block supplies all
/// [`SLOTS`] lines (so the shop cannot read past its end into neighbouring
/// data), and that no line exceeds the 20-character maximum the shipped
/// strings observe.
pub const fn block(s: &'static [u8]) -> &'static [u8] {
    let mut slots = 0;
    let mut line = 0;
    let mut i = 0;
    while i < s.len() {
        if s[i] == 0 {
            assert!(line <= 20, "description line longer than the shipped maximum of 20");
            slots += 1;
            line = 0;
        } else {
            line += 1;
        }
        i += 1;
    }
    assert!(line == 0, "description block must end in a NUL");
    assert!(
        slots >= SLOTS,
        "description block has fewer than 4 line slots, so the shop will render \
         past its end into whatever bytes follow"
    );
    s
}

// --- Transmissions -----------------------------------------------------------
// Shipped line 1 is "Good engine," on all five. Lines 2 and 3 are reproduced
// byte for byte from the shipped blocks, star counts included.
//
// Gear counts come from `shipped::TRANSMISSIONS`: Normal and Sports run five
// forward gears, the rest six, with top-gear ratios 490 / 557 / 660 / 711 / 750
// in 8.8 fixed point -- which is what separates "long legs" from "wide band".

pub static TRANS_SPORTS: &[u8] = block(b"5-speed, quicker\0better top-speed\0 Gear Ratio**\0\0");
pub static TRANS_POWER: &[u8] = block(b"6-speed, torquey\0better top-speed\0 Gear Ratio***\0\0");
pub static TRANS_SPEED: &[u8] = block(b"6-speed, long legs\0better top-speed\0 Gear Ratio****\0\0");
pub static TRANS_WIDE: &[u8] = block(b"6-speed, wide band\0better top-speed\0 Gear Ratio*****\0\0");
pub static TRANS_HYPER: &[u8] = block(b"6-speed, maximum\0better top-speed\0 Gear Ratio******\0\0");
