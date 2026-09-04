//! Race prize money, spliced back over its own address range.
//!
//! `0x0029fcc0..0x0029fd30` is 28 consecutive `u32`s that read as descending
//! runs -- one per race class, indexed by finishing position. The region is
//! bounded on both sides: from `0x29fd30` on the words are `.sdata` pointers
//! (0x333xxx), not money.
//!
//! ## What the remaster changes
//!
//! Choro Q's reputation is the money grind: rank C pays 100-400 coins a race
//! while a Sports engine costs 1000, so the opening hours are lap repetition
//! rather than the town-exploring RPG underneath.
//!
//! Where one run ends and the next begins is *inferred* from where the values
//! stop descending, so the rescale deliberately avoids depending on it. Every
//! prize is passed through
//!
//! ```text
//! f(x) = 2x                 for x <= 10000
//! f(x) = x + 10000          for x >  10000
//! ```
//!
//! which is non-decreasing across its whole domain. Applying a non-decreasing
//! map elementwise cannot reorder a descending run, so each class stays
//! correctly ordered *whatever* the true class boundaries are. It doubles the
//! early and mid game -- roughly halving the races needed per upgrade tier --
//! while the 80,000-coin class, already the game's payoff, moves only 12%.

use resplice_macros::Splice;

/// Prize money by race class and finishing position.
///
/// Each row's shipped values are in the comment above it.
#[Splice(begin = 0x0029fcc0, end = 0x0029fd30)]
pub static PRIZES: [u32; 28] = [
    // 400 / 300 / 200 / 100
    800, 600, 400, 200,
    // 1500 / 1200 / 1000 / 800 / 600 / 500
    3000, 2400, 2000, 1600, 1200, 1000,
    // 2500 / 2000 / 1600 / 1200 / 1000 / 800
    5000, 4000, 3200, 2400, 2000, 1600,
    // 80000 / 60000 / 40000 / 30000 / 20000 / 10000
    90000, 70000, 50000, 40000, 30000, 20000,
    // 10000 / 7500 / 5000 / 3000 / 2000 / 1000
    20000, 15000, 10000, 6000, 4000, 2000,
];

/// The rescale is only sound if it never reorders a run, so pin that here
/// rather than trusting the arithmetic above: `f` must be non-decreasing.
const _: () = {
    const SHIPPED: [u32; 28] = [
        400, 300, 200, 100,
        1500, 1200, 1000, 800, 600, 500,
        2500, 2000, 1600, 1200, 1000, 800,
        80000, 60000, 40000, 30000, 20000, 10000,
        10000, 7500, 5000, 3000, 2000, 1000,
    ];
    const fn f(x: u32) -> u32 {
        if x <= 10000 { 2 * x } else { x + 10000 }
    }
    let mut i = 0;
    while i < 28 {
        assert!(PRIZES[i] == f(SHIPPED[i]), "prize row is not the rescale of its shipped value");
        // Descending order is preserved wherever the shipped run descends.
        if i > 0 && SHIPPED[i - 1] >= SHIPPED[i] {
            assert!(PRIZES[i - 1] >= PRIZES[i], "rescale reordered a prize run");
        }
        i += 1;
    }
};
