//! Surface deposits: what the world *grows* (C9, Q102 and Q108).
//!
//! `geology` is where a substance is **mined** — a seam under a tile, worked out and gone for
//! good. This is the other half of Q102: a substance taken **by hand without a mine**, from a
//! patch standing on the ground. Brush for fibre, stands for timber. Both are still ground mass,
//! so taking either debits the ground like anything else, which is what keeps the campaign's
//! claim true of a gather as well as of an extraction.
//!
//! # Three properties, and each has a test
//!
//! * **Derived, like the geology.** The same seed and tile give the same patch, forever, in
//!   integer arithmetic — Q59's rule, one layer up. Nothing about a patch is stored except the
//!   delta the world has already taken.
//! * **Regrowth is a function of elapsed time, not of ticks** (Q61). What a patch stands is
//!   `base - harvested + rate × sim-days since the last take`, so a replay that ran at a
//!   different speed computes the same patch — and a save loaded after a week of real time is a
//!   patch that grew, rather than one that owes the world a week of growth.
//! * **A stripped patch stops growing.** Q108: *"brush regrows on a declared cycle if not
//!   stripped to the soil; stands take years; mineral deposits never regrow."* A take that
//!   leaves less than a tenth of the base strips the tile, and planting it again is Q108's own
//!   deliberate form — a task this rung does not build, named here rather than implied.
//!
//! # The one thing that makes this hard, and how it is handled
//!
//! Regrowth is mass *entering* the world, and [`SCHEMA.md`] is explicit that **gathering is the
//! one place mass legitimately enters the world**. So the take total a tile records is
//! **monotone**: `harvested_g` is what this patch has given up, and it never goes down when the
//! patch grows back. The standing mass is derived from it, and the audit reads it as the ground's
//! contribution. A `harvested_g` that fell as the patch regrew would make the mass audit report
//! material held in a city as *never dug up*, which is the false finding — not the defect — this
//! whole record is written to avoid.

use crate::materials::geology;
use crate::materials::schema;

/// Tiles across one surface patch. Smaller than a geology cell (32) on purpose: a thicket is
/// something you can see, walk around and clear, and a 32-tile patch of brush would be a biome.
pub const SURFACE_CELL: u32 = 8;

/// The lattice the *kind* field reads — coarser than richness, so a patch is one kind rather
/// than a different one every eight metres.
const KIND_CELL: u32 = 24;

/// The fraction of a patch's base mass below which a take leaves it **stripped to the soil**:
/// one tenth, declared, and the only thing that stops a renewable from renewing.
pub const STRIP_FLOOR: (i64, i64) = (1, 10);

/// One declared surface kind: **what grows**, how much of it a full tile holds, and how fast it
/// comes back.
pub struct SurfaceKind {
    /// The substance, in the schema's vocabulary. A surface kind may only name a substance the
    /// table declares `gathered` — a mineral belongs in `geology`'s deposit kinds.
    pub substance: &'static str,
    /// Mass a full tile holds, in grams (Q102: a patch is *tonnes*, not hundreds of tonnes).
    pub base_g: i64,
    /// Grams the patch regrows per sim-day once it has been cut back.
    pub regrowth_g_per_day: i64,
    /// The noise floor (0..=1000) a tile must clear to hold this kind.
    pub min_richness: i64,
    /// Relative weight when the field picks a kind.
    pub share: i64,
    /// Why these numbers, so the declaration can be argued with rather than trusted.
    pub note: &'static str,
}

/// The declared surface kinds, in the order the field slices into them.
pub const SURFACE_KINDS: &[SurfaceKind] = &[
    SurfaceKind {
        substance: "timber",
        base_g: 6_000_000,
        regrowth_g_per_day: 4_200,
        min_richness: 620,
        share: 45,
        note: "a stand: six tonnes on one tile is a few trees, and 4 200 g/day is Q108's \
               *stands take years* — four years for the whole of it, declared rather than \
               sampled",
    },
    SurfaceKind {
        substance: "plant_fibre",
        base_g: 1_200_000,
        regrowth_g_per_day: 20_000,
        min_richness: 520,
        share: 55,
        note: "brush: 1.2 t of fibre on a tile, back in about sixty days at 20 kg/day. This is \
               Q108's declared cycle, and the renewable half of the pair",
    },
];

/// The substance a kind grows.
pub fn kind_substance(kind: usize) -> &'static str {
    SURFACE_KINDS[kind].substance
}

/// The appearance family a kind presents as — the substance's own, so brush draws in the
/// world's declared colours rather than in a second opinion about what it looks like.
pub fn kind_family(kind: usize) -> &'static str {
    schema::substance(kind_substance(kind))
        .map(|entry| entry.family)
        .unwrap_or_else(|| panic!("surface kind {kind} names a substance the table does not declare"))
}

/// How fast a kind comes back, in grams per sim-day. Zero for anything that does not.
pub fn kind_regrowth(kind: usize) -> i64 {
    SURFACE_KINDS[kind].regrowth_g_per_day
}

/// A patch on one tile: which kind, how rich, and the mass it holds untouched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Surface {
    /// Index into [`SURFACE_KINDS`].
    pub kind: usize,
    /// How rich the patch is at this tile, 0..=1000. The mass scales with it, so a thicket has
    /// a fat middle and thin edges — which is where you site the first gather.
    pub richness: i64,
    /// The mass this tile holds before anyone takes from it, in grams, exactly.
    pub base_g: i64,
}

/// The patch on one tile, or `None` where the ground grows nothing worth taking.
///
/// Pure and seed-derived: whether a tile is *reachable*, paved or built on is the world's
/// business, not this function's — the same boundary `geology::deposit` keeps.
pub fn surface(seed: u64, x: u32, y: u32) -> Option<Surface> {
    let richness = richness_at(seed, x, y);
    let kind = kind_at(seed, x, y);
    if richness < SURFACE_KINDS[kind].min_richness {
        return None;
    }
    Some(Surface { kind, richness, base_g: mass_g(kind, richness) })
}

/// The mass of one tile of a kind at a given richness, in grams: `base × richness / 1000`,
/// exactly, with no float in it.
pub fn mass_g(kind: usize, richness: i64) -> i64 {
    let richness = richness.clamp(0, geology::RICHNESS_ONE);
    SURFACE_KINDS[kind].base_g * richness / geology::RICHNESS_ONE
}

/// The declared base mass of a kind at full richness — the number a gate reads when it wants to
/// state a patch's worth without picking a tile.
pub fn full_mass_g(kind: usize) -> i64 {
    SURFACE_KINDS[kind].base_g
}

/// Which kind a patch grows, by the declared shares — sliced from a coarser field than richness,
/// so a patch is *one* kind rather than a different one every tile.
fn kind_at(seed: u64, x: u32, y: u32) -> usize {
    let cell = KIND_CELL;
    let pick = geology::noise_at(seed, SALT_SURFACE_KIND, (x / cell) * cell, (y / cell) * cell, KIND_CELL as i64);
    let total: i64 = SURFACE_KINDS.iter().map(|kind| kind.share).sum();
    let mut target = pick * total / geology::RICHNESS_ONE;
    for (index, kind) in SURFACE_KINDS.iter().enumerate() {
        target -= kind.share;
        if target < 0 {
            return index;
        }
    }
    SURFACE_KINDS.len() - 1
}

fn richness_at(seed: u64, x: u32, y: u32) -> i64 {
    geology::noise_at(seed, SALT_SURFACE_RICHNESS, x, y, SURFACE_CELL as i64)
}

const SALT_SURFACE_KIND: u64 = 0x5A_11;
const SALT_SURFACE_RICHNESS: u64 = 0x5A_22;

/// This tile's own regrowth rate, in grams per sim-day: the kind's declared rate scaled by the
/// tile's share of the kind's full base.
///
/// Derived rather than declared twice — the same reasoning that moved densities into
/// `SUBSTANCES`. A thin edge of a thicket comes back more slowly than its middle, exactly as it
/// holds less of it.
pub fn tile_regrowth_g_per_day(kind: usize, base_g: i64) -> i64 {
    let full = SURFACE_KINDS[kind].base_g;
    if full <= 0 {
        return 0;
    }
    SURFACE_KINDS[kind].regrowth_g_per_day * base_g / full
}

/// What a patch stands right now, given the delta the world stores for that tile.
///
/// `harvested_g` is the **take total** rather than the shortfall: it never goes down when the
/// patch grows back (see this module's header), and `harvested_day` is when the last take
/// happened. `day` is sim-days, so this is Q61's *lazy on a boundary, a function of elapsed
/// time* rather than of a tick count.
pub fn standing_g(
    base_g: i64,
    regrowth_g_per_day: i64,
    harvested_g: i64,
    harvested_day: u64,
    stripped: bool,
    day: u64,
) -> i64 {
    if stripped {
        // Stripped to the soil. The remainder is soil rather than a standing patch, and what
        // brings this tile back is planting (Q108's (c)), not time.
        return 0;
    }
    let elapsed = day.saturating_sub(harvested_day) as i64;
    (base_g - harvested_g + regrowth_g_per_day * elapsed).clamp(0, base_g)
}

/// One take's arithmetic: what came off the patch, what the tile's delta becomes, and whether
/// the patch is stripped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Take {
    /// What actually came off, in grams — never more than the patch stood.
    pub taken_g: i64,
    /// The tile's new take total, in grams.
    pub harvested_g: i64,
    /// Whether the patch is now stripped to the soil.
    pub stripped: bool,
}

/// Take up to `want_g` off a patch, returning **only what stood there** — the same rule
/// `World::extract` follows, for the same reason: a caller handed more than the ground held
/// would break the ledger at the next audit.
pub fn take(
    base_g: i64,
    regrowth_g_per_day: i64,
    harvested_g: i64,
    harvested_day: u64,
    stripped: bool,
    day: u64,
    want_g: i64,
) -> Take {
    let now = standing_g(base_g, regrowth_g_per_day, harvested_g, harvested_day, stripped, day);
    let taken_g = want_g.clamp(0, now);
    let after = now - taken_g;
    let stripped = stripped || after * STRIP_FLOOR.1 < base_g * STRIP_FLOOR.0;
    Take { taken_g, harvested_g: harvested_g + taken_g, stripped }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Q59's premise one layer up: a patch is a function of the seed, not a roll.
    #[test]
    fn the_same_seed_and_tile_always_give_the_same_patch() {
        for x in (0..256).step_by(5) {
            for y in (0..256).step_by(9) {
                assert_eq!(surface(0xC117_2026, x, y), surface(0xC117_2026, x, y));
            }
        }
        assert_eq!(surface(7, 40, 40), surface(7, 40, 40));
    }

    /// Every declared kind occurs, and every one of them is a substance the table declares
    /// `gathered` — a surface kind that named a mineral would be a mine wearing brush.
    #[test]
    fn every_declared_kind_grows_a_gathered_substance() {
        let seed = 0xC117_2026;
        let mut seen = vec![false; SURFACE_KINDS.len()];
        for x in (0..2048).step_by(11) {
            for y in (0..2048).step_by(13) {
                if let Some(found) = surface(seed, x, y) {
                    seen[found.kind] = true;
                }
            }
        }
        for (index, kind) in SURFACE_KINDS.iter().enumerate() {
            assert!(seen[index], "`{}` is declared but never grows anywhere", kind.substance);
            let entry = schema::substance(kind.substance)
                .unwrap_or_else(|| panic!("`{}` is a surface and not a substance", kind.substance));
            assert_eq!(
                entry.source, "gathered",
                "`{}` grows on the ground and the table does not declare it gathered",
                kind.substance
            );
            assert!(kind.base_g > 0, "`{}` holds nothing at all", kind.substance);
            assert!(
                kind.regrowth_g_per_day > 0,
                "`{}` is a renewable with no declared cycle, so Q108 is unanswerable for it",
                kind.substance
            );
        }
    }

    /// A patch covers part of the map rather than all of it or none of it, and it is regional:
    /// a field of isolated tiles would pass every other test here and leave nowhere to gather.
    #[test]
    fn patches_form_regions_rather_than_confetti() {
        let seed = 0xC117_2026;
        let (mut grown, mut with_grown_neighbour) = (0, 0);
        for x in 0..255 {
            for y in 0..255 {
                if surface(seed, x, y).is_none() {
                    continue;
                }
                grown += 1;
                if surface(seed, x + 1, y).is_some() || surface(seed, x, y + 1).is_some() {
                    with_grown_neighbour += 1;
                }
            }
        }
        let fraction = grown as f64 / (255.0 * 255.0);
        assert!(
            (0.05..0.80).contains(&fraction),
            "patches cover {fraction:.3} of the map, which is neither barren nor carpet"
        );
        assert!(
            with_grown_neighbour as f64 / grown as f64 > 0.6,
            "most patches should have a grown neighbour, so the field is regional"
        );
    }

    /// Q108, as arithmetic: what you take comes off, and what you leave grows back **as a
    /// function of sim-days** — not of ticks, and not of a flag somebody set.
    #[test]
    fn a_patch_grows_back_with_elapsed_time() {
        let brush = SURFACE_KINDS
            .iter()
            .position(|kind| kind.substance == "plant_fibre")
            .expect("brush is declared");
        let base = full_mass_g(brush);
        let rate = tile_regrowth_g_per_day(brush, base);
        let day = 10;

        // Untouched: the whole patch stands, whatever the day.
        assert_eq!(standing_g(base, rate, 0, 0, false, 0), base);
        assert_eq!(standing_g(base, rate, 0, 0, false, 10_000), base);

        // A take leaves a hole, and the hole closes at the declared rate.
        let take_now = take(base, rate, 0, 0, false, day, base / 2);
        assert_eq!(take_now.taken_g, base / 2, "half the patch, exactly");
        assert_eq!(take_now.harvested_g, base / 2);
        assert!(!take_now.stripped, "half a patch is not stripped soil");
        assert_eq!(
            standing_g(base, rate, take_now.harvested_g, day, false, day),
            base / 2,
            "the day of the take is the patch it left"
        );

        // Twenty days later, twenty days of growth — the declared 20 kg/day, scaled to the tile.
        let grown = standing_g(base, rate, take_now.harvested_g, day, false, day + 20);
        assert_eq!(
            grown,
            base / 2 + rate * 20,
            "regrowth is the declared rate times elapsed sim-days"
        );
        assert!(grown > base / 2, "and a patch without a take is a patch that grows");

        // A thin edge of the same thicket comes back more slowly than its middle, derived from
        // the same declaration rather than declared a second time.
        assert_eq!(rate, kind_regrowth(brush), "a full tile regrows at the declared rate");
        assert!(tile_regrowth_g_per_day(brush, base / 4) < rate);
    }

    /// The take total is monotone even as the patch regrows — the property that keeps the mass
    /// audit from reporting a city's own stock as material the ground never gave up.
    #[test]
    fn growing_back_does_not_lower_what_the_ground_gave_up() {
        let brush = SURFACE_KINDS
            .iter()
            .position(|kind| kind.substance == "plant_fibre")
            .expect("brush is declared");
        let base = full_mass_g(brush);
        let rate = tile_regrowth_g_per_day(brush, base);
        let first = take(base, rate, 0, 0, false, 0, base / 4);
        // A year of regrowth, then another take: the take total only ever rises.
        let second = take(base, rate, first.harvested_g, 0, first.stripped, 360, base / 4);
        assert!(second.harvested_g > first.harvested_g, "{second:?}");
        assert_eq!(
            second.harvested_g,
            first.harvested_g + second.taken_g,
            "the total is the sum of the takes, and regrowth is not a negative one"
        );
    }

    /// Q108's condition, as a rule: a take that leaves under a tenth strips the tile, and a
    /// stripped tile never regrows. That is what makes stripping a real decision.
    #[test]
    fn a_patch_stripped_to_the_soil_does_not_grow_back() {
        let stand = SURFACE_KINDS
            .iter()
            .position(|kind| kind.substance == "timber")
            .expect("a stand is declared");
        let base = full_mass_g(stand);
        let rate = tile_regrowth_g_per_day(stand, base);

        // Take all but a twentieth: under the declared floor, so the patch is stripped.
        let stripped = take(base, rate, 0, 0, false, 0, base - base / 20);
        assert!(stripped.stripped, "less than a tenth left is stripped soil: {stripped:?}");
        assert_eq!(
            standing_g(base, rate, stripped.harvested_g, 0, true, 100_000),
            0,
            "a stripped patch grows nothing, however long you wait"
        );

        // Leave a fifth instead, and it comes back.
        let kept = take(base, rate, 0, 0, false, 0, base - base / 5);
        assert!(!kept.stripped, "a fifth left is a patch that will return: {kept:?}");
        assert!(
            standing_g(base, rate, kept.harvested_g, 0, false, 200) > base / 5,
            "and it returns at the declared rate"
        );
    }

    /// A take is bounded by what stands, which is the rule that keeps the ledger whole: a
    /// caller asking for more than the patch holds is handed what is there.
    #[test]
    fn a_take_is_bounded_by_what_the_patch_holds() {
        let stand = SURFACE_KINDS
            .iter()
            .position(|kind| kind.substance == "timber")
            .expect("a stand is declared");
        let base = full_mass_g(stand);
        let rate = tile_regrowth_g_per_day(stand, base);
        let greedy = take(base, rate, 0, 0, false, 0, base * 4);
        assert_eq!(greedy.taken_g, base, "the patch gives up what it has, and no more");
        assert_eq!(
            standing_g(base, rate, greedy.harvested_g, 0, false, 0),
            0,
            "and it is bare, not negative"
        );
        let nothing = take(base, rate, 0, 0, false, 0, -5);
        assert_eq!(nothing.taken_g, 0, "a negative want takes nothing");
    }

    /// Mass scales with richness, exactly, with no float anywhere in it.
    #[test]
    fn a_thin_edge_holds_proportionally_less() {
        for kind in 0..SURFACE_KINDS.len() {
            let full = full_mass_g(kind);
            assert_eq!(mass_g(kind, geology::RICHNESS_ONE), full);
            assert_eq!(mass_g(kind, geology::RICHNESS_ONE / 2), full / 2);
            assert_eq!(mass_g(kind, 0), 0);
        }
    }
}
