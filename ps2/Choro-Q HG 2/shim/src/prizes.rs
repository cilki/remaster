//! Race prize money, spliced back over its own address range.
//!
//! `0x0029fcb8..0x0029fd30` is 30 consecutive `u32`s: **five classes of six
//! finishing positions**, 1st through 6th. The region is bounded on both
//! sides -- the words below `0x29fcb8` are `0, 1, 12, 0` and the words from
//! `0x29fd30` on are `.sdata` pointers (`0x333xxx`), neither of which is
//! money.
//!
//! ## The eight bytes this table nearly lost
//!
//! An earlier build of this shim spliced `0x29fcc0..0x29fd30` and described it
//! as 28 entries "bounded on both sides". It was not: the table starts two
//! words lower, at `0x29fcb8`, and those two words are Rank C's 1st and 2nd
//! prizes. Splicing from `0x29fcc0` rescaled everything *except* them, which
//! left Rank C paying
//!
//! ```text
//! 1st 800   2nd 500   3rd 800   4th 600   5th 400   6th 200
//! ```
//!
//! -- 3rd place paying the same as 1st and more than 2nd. The mistake was
//! invisible in the ELF (the run still looked plausible) and invisible to a
//! read of the patched table on its own terms; it showed up the moment a real
//! race was finished and the results screen paid 500 for 2nd, a number that
//! does not occur anywhere in the patched table. Hence the run-shape assertion
//! at the bottom of this file, which now checks the class structure rather
//! than trusting it.
//!
//! ## What the remaster changes
//!
//! Choro Q's reputation is the money grind: rank C pays a few hundred coins a
//! race while a Sports engine costs 1000, so the opening hours are lap
//! repetition rather than the town-exploring RPG underneath.
//!
//! Every prize is passed through
//!
//! ```text
//! f(x) = 2x                 for x <= 10000
//! f(x) = x + 10000          for x >  10000
//! ```
//!
//! which is non-decreasing across its whole domain, so it cannot reorder a
//! descending run. It doubles the early and mid game -- roughly halving the
//! races needed per upgrade tier -- while the 80,000-coin class, already the
//! game's payoff, moves only 12%.

use resplice_macros::Splice;

/// Number of finishing positions in each race class.
const PLACES: usize = 6;
/// Number of race classes in the table.
const CLASSES: usize = 5;
/// Total prize entries.
pub const PRIZE_COUNT: usize = CLASSES * PLACES;

/// Prize money by race class and finishing position, 1st through 6th.
///
/// Each row's shipped values are in the comment above it.
#[Splice(begin = 0x0029fcb8, end = 0x0029fd30)]
pub static PRIZES: [u32; PRIZE_COUNT] = [
    // Rank C -- shipped 800 / 500 / 400 / 300 / 200 / 100
    1600, 1000, 800, 600, 400, 200,
    // Rank B -- shipped 1500 / 1200 / 1000 / 800 / 600 / 500
    3000, 2400, 2000, 1600, 1200, 1000,
    // Rank A -- shipped 2500 / 2000 / 1600 / 1200 / 1000 / 800
    5000, 4000, 3200, 2400, 2000, 1600,
    // Top class -- shipped 80000 / 60000 / 40000 / 30000 / 20000 / 10000
    90000, 70000, 50000, 40000, 30000, 20000,
    // Shipped 10000 / 7500 / 5000 / 3000 / 2000 / 1000
    20000, 15000, 10000, 6000, 4000, 2000,
];

/// Pin both properties the rescale relies on, rather than trusting them:
/// every entry is `f` of its shipped value, and every class still descends
/// from 1st to 6th.
const _: () = {
    const SHIPPED: [u32; PRIZE_COUNT] = [
        800, 500, 400, 300, 200, 100,
        1500, 1200, 1000, 800, 600, 500,
        2500, 2000, 1600, 1200, 1000, 800,
        80000, 60000, 40000, 30000, 20000, 10000,
        10000, 7500, 5000, 3000, 2000, 1000,
    ];
    const fn f(x: u32) -> u32 {
        if x <= 10000 { 2 * x } else { x + 10000 }
    }
    let mut i = 0;
    while i < PRIZE_COUNT {
        assert!(PRIZES[i] == f(SHIPPED[i]), "prize row is not the rescale of its shipped value");
        // Within a class, a later finishing position must never pay more than
        // an earlier one. Checked on the shipped table too, so a bad class
        // boundary shows up here instead of on the results screen.
        if i % PLACES != 0 {
            assert!(SHIPPED[i - 1] >= SHIPPED[i], "shipped class does not descend");
            assert!(PRIZES[i - 1] >= PRIZES[i], "rescale reordered a prize run");
        }
        i += 1;
    }
};
