//! Seed-derived geology: where the world's material comes from (C9 phase 1).
//!
//! Round 1 answered that **the world itself and its resources** are the only source, and
//! round 3's Q20 answered that **the site, not a mass figure, is the unit of depletion**:
//! a deposit occupies tiles, you work it tile by tile, and a worked-out tile is exhausted.
//! This module is the first half of that — *which tiles hold what, and how much*, derived
//! from the world's seed so that the world can be regenerated from a seed plus its deltas
//! (round 9's Q59) rather than stored whole.
//!
//! Three properties are the point, and each has a test:
//!
//! * **Deterministic.** The same seed and tile give the same deposit, forever, on any
//!   machine — so the headless verifier can replay a run and a save can store only what
//!   changed. This is why the noise below is **integer** arithmetic and not float: an
//!   `f32` bilinear interpolation is deterministic in practice and not guaranteed across
//!   platforms, and a replay that disagrees with its own seed is the one failure this
//!   repo's whole persistence story cannot absorb.
//! * **Regional.** Deposits come from interpolated value noise on a coarse lattice, so
//!   they form patches you can drive to, not confetti. A mine map that looks like static
//!   has no sites in it.
//! * **Exact.** Mass is derived from the declared scale factor in integers — see
//!   [`PROVISIONAL`] for why that phrase is doing work here.
//!
//! # Where the numbers live
//!
//! The **densities have moved** into `SUBSTANCES` (`tools/materials/declare.py`, emitted
//! into `src/materials/generated.rs`), which is what the previous note here said would
//! happen when that table landed: a substance's density is a fact about the substance, not
//! about one place it is found. What stays here is the **derivation** — the tile size, the
//! seam depth, the noise, the kind thresholds and shares — because those are facts about a
//! site rather than about a material.
//!
//! One declared scale factor still sets everything: [`TILE_METRES`] times the seam depth
//! times the substance's own density. A wrong density is now a row somebody can read beside
//! every other material fact, which is the whole reason the table exists.

/// Metres to a centimetre, so the arithmetic below stays in integers.
const CM_PER_M: i64 = 100;

/// **The declared scale factor.** A tile is 8 m on a side — Cities Skylines' own cell —
/// so a tile is 64 m². This number sets every mass in the game, which is why round 3's Q31
/// answered *declare the scale factor and derive it* rather than declaring mass directly:
/// the number that can be wrong is one visible declaration instead of an invisible
/// consequence.
pub const TILE_METRES: i64 = 8;

/// A tile's area in cm²: 800 × 800.
pub const TILE_AREA_CM2: i64 = TILE_METRES * CM_PER_M * TILE_METRES * CM_PER_M;

/// The declared seam depth worked by a mine, in cm. 2 m is a modest open-cast bench, and it
/// is the number round 3's arithmetic was built on.
pub const SEAM_DEPTH_CM: i64 = 200;

/// The volume one tile of deposit offers, in mL (= cm³). 64 m² × 2 m = 128 m³ = 128 L...
/// no: 128 m³ is 128 000 L, which is 128 000 000 mL. See the test.
pub fn tile_volume_ml() -> i64 {
    TILE_AREA_CM2 * SEAM_DEPTH_CM
}

/// Tiles across one geology cell. Bigger than a tile, so deposits are regional.
pub const GEOLOGY_CELL: u32 = 32;

/// One declared deposit kind: **where** a substance is found, not what it is.
///
/// The substance, its density and the family it presents as all come from
/// [`crate::materials::schema`]'s `SUBSTANCES` table, so a deposit cannot hold a material
/// the table does not describe. What belongs to a *site* is here: the noise floor a tile
/// must clear, and the kind's weight when the field decides which substance a patch holds
/// — so iron is rarer than stone as a declared number rather than as an accident of a hash.
pub struct DepositKind {
    /// The substance, in the schema's vocabulary.
    pub substance: &'static str,
    /// The noise floor (0..=1000) a tile must clear to hold this kind.
    pub min_richness: i64,
    /// Relative weight when the field picks a kind, largest last is *not* assumed.
    pub share: i64,
}

/// The declared deposit kinds, in the order the field slices into them.
pub const DEPOSIT_KINDS: &[DepositKind] = &[
    DepositKind { substance: "stone", min_richness: 620, share: 34 },
    DepositKind { substance: "sand", min_richness: 660, share: 22 },
    DepositKind { substance: "clay", min_richness: 640, share: 22 },
    DepositKind { substance: "coal", min_richness: 700, share: 14 },
    DepositKind { substance: "iron_ore", min_richness: 740, share: 8 },
];

/// The substance a kind holds.
pub fn kind_substance(kind: usize) -> &'static str {
    DEPOSIT_KINDS[kind].substance
}

/// The appearance family a kind presents as — the substance's own, so a deposit draws in
/// the world's declared colours rather than in a second opinion about what it looks like.
pub fn kind_family(kind: usize) -> &'static str {
    crate::materials::schema::substance(kind_substance(kind))
        .map(|entry| entry.family)
        .unwrap_or_else(|| panic!("deposit kind {kind} names a substance the table does not declare"))
}

/// A kind's density as a declared rational, in g/mL.
fn kind_density(kind: usize) -> (i64, i64) {
    let entry = crate::materials::schema::substance(kind_substance(kind))
        .unwrap_or_else(|| panic!("deposit kind {kind} names an undeclared substance"));
    (entry.density.num, entry.density.den)
}

/// One tile's deposit: which kind, and how much of it is there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Deposit {
    /// Index into [`DEPOSIT_KINDS`].
    pub kind: usize,
    /// How rich the patch is at this tile, 0..=1000. The mass scales with it, so a patch
    /// has a fat middle and thin edges — which is what makes where you put the mine a
    /// decision rather than a formality.
    pub richness: i64,
    /// The mass this tile holds, in grams, exactly.
    pub mass_g: i64,
}

/// The declared mass of one tile of a kind at a given richness, in grams.
///
/// Exact integer arithmetic: volume in mL × a rational density, scaled by richness over
/// 1000. Iron ore at full richness is 128 000 000 mL × 27/10 = 345 600 000 g — **345.6 t**,
/// which is the "one worked tile is worth about a house and a half" figure the record
/// states, now derived rather than asserted.
pub fn mass_g(kind: usize, richness: i64) -> i64 {
    let (density_num, density_den) = kind_density(kind);
    let richness = richness.clamp(0, RICHNESS_ONE);
    tile_volume_ml() * density_num * richness / (density_den * RICHNESS_ONE)
}

/// The top of the richness scale. A plain integer scale so that comparisons and slices are
/// exact rather than floating.
pub const RICHNESS_ONE: i64 = 1000;

/// The deposit on one tile, or `None` where the ground is barren.
pub fn deposit(seed: u64, x: u32, y: u32) -> Option<Deposit> {
    let richness = noise(seed, SALT_RICHNESS, x, y);
    let kind = kind_at(seed, x, y);
    let declaration = &DEPOSIT_KINDS[kind];
    if richness < declaration.min_richness {
        return None;
    }
    Some(Deposit { kind, richness, mass_g: mass_g(kind, richness) })
}

/// The declared total mass of one tile of a kind at full richness, in grams — the number a
/// gate reads when it wants to state a deposit's worth without picking a tile.
pub fn full_tile_mass_g(kind: usize) -> i64 {
    mass_g(kind, RICHNESS_ONE)
}

/// Which kind a patch holds, by the declared shares. Sliced from a second, coarser field so
/// that a patch is *one* substance rather than a different one every few tiles — a mine that
/// changes what it is digging every 8 metres is not a mine.
fn kind_at(seed: u64, x: u32, y: u32) -> usize {
    let cell = GEOLOGY_CELL * 4;
    let pick = noise(seed, SALT_KIND, (x / cell) * cell, (y / cell) * cell);
    let total: i64 = DEPOSIT_KINDS.iter().map(|kind| kind.share).sum();
    let mut target = pick * total / RICHNESS_ONE;
    for (index, kind) in DEPOSIT_KINDS.iter().enumerate() {
        target -= kind.share;
        if target < 0 {
            return index;
        }
    }
    DEPOSIT_KINDS.len() - 1
}

const SALT_RICHNESS: u64 = 0x51_01;
const SALT_KIND: u64 = 0xD3_02;

/// Integer bilinear value noise on a lattice one [`GEOLOGY_CELL`] wide.
///
/// Integer, not float, and that is a decision rather than a style: see this module's header.
fn noise(seed: u64, salt: u64, x: u32, y: u32) -> i64 {
    let span = GEOLOGY_CELL as i64;
    let cx = (x as i64) / span;
    let cy = (y as i64) / span;
    let fx = (x as i64) % span;
    let fy = (y as i64) % span;

    let corner = |dx: i64, dy: i64| lattice(seed, salt, cx + dx, cy + dy);
    let top = corner(0, 0) + (corner(1, 0) - corner(0, 0)) * fx / span;
    let bottom = corner(0, 1) + (corner(1, 1) - corner(0, 1)) * fx / span;
    top + (bottom - top) * fy / span
}

/// One lattice point's value, 0..=1000, from the seed and a salt.
fn lattice(seed: u64, salt: u64, cx: i64, cy: i64) -> i64 {
    let mixed = mix(
        seed ^ mix(salt)
            ^ (cx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (cy as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F),
    );
    (mixed % (RICHNESS_ONE as u64 + 1)) as i64
}

/// A cheap integer finaliser, so neighbouring lattice points are uncorrelated.
fn mix(mut value: u64) -> u64 {
    value ^= value >> 33;
    value = value.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    value ^= value >> 33;
    value = value.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    value ^ (value >> 33)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The scale factor multiplies out to the number the record already states. If a tile
    /// size or a seam depth is edited, this test is where it announces itself.
    #[test]
    fn one_worked_tile_is_worth_about_a_house_and_a_half() {
        assert_eq!(TILE_AREA_CM2, 640_000, "an 8 m tile is 64 m²");
        assert_eq!(tile_volume_ml(), 128_000_000, "a 2 m seam is 128 m³");
        let iron = DEPOSIT_KINDS
            .iter()
            .position(|kind| kind.substance == "iron_ore")
            .expect("iron is declared");
        assert_eq!(
            full_tile_mass_g(iron),
            345_600_000,
            "345.6 t of ore in one tile, in exact grams"
        );
        assert_eq!(mass_g(iron, RICHNESS_ONE / 2), full_tile_mass_g(iron) / 2);
        assert_eq!(mass_g(iron, 0), 0, "a thin edge holds nothing");
    }

    /// Q59's whole premise: the world is derived from its seed, so the derivation must be a
    /// function and not a roll.
    #[test]
    fn the_same_seed_and_tile_always_give_the_same_deposit() {
        for x in (0..256).step_by(7) {
            for y in (0..256).step_by(11) {
                assert_eq!(deposit(0xC117_2026, x, y), deposit(0xC117_2026, x, y));
            }
        }
    }

    #[test]
    fn a_different_seed_is_a_different_world() {
        let mut differences = 0;
        for x in 0..128 {
            for y in 0..128 {
                if deposit(1, x, y) != deposit(2, x, y) {
                    differences += 1;
                }
            }
        }
        assert!(differences > 2_000, "two seeds agreed almost everywhere: {differences}");
    }

    /// Deposits are regions you can drive to. A field of isolated tiles would pass every
    /// other test here and still leave nowhere to put a mine.
    #[test]
    fn deposits_form_regions_rather_than_confetti() {
        let seed = 0xC117_2026;
        let mut deposited = 0;
        let mut with_deposited_neighbour = 0;
        for x in 0..255 {
            for y in 0..255 {
                if deposit(seed, x, y).is_none() {
                    continue;
                }
                deposited += 1;
                if deposit(seed, x + 1, y).is_some() || deposit(seed, x, y + 1).is_some() {
                    with_deposited_neighbour += 1;
                }
            }
        }
        let fraction = deposited as f64 / (255.0 * 255.0);
        assert!(
            (0.05..0.45).contains(&fraction),
            "deposits cover {fraction:.3} of the map, which is neither barren nor carpet"
        );
        assert!(
            with_deposited_neighbour as f64 / deposited as f64 > 0.5,
            "most deposits should have a deposited neighbour, so the field is regional"
        );
    }

    /// Every kind is reachable, or a declared substance is a substance nobody can mine.
    #[test]
    fn every_declared_kind_is_actually_reachable() {
        let seed = 0xC117_2026;
        let mut seen = vec![false; DEPOSIT_KINDS.len()];
        for x in (0..4096).step_by(13) {
            for y in (0..4096).step_by(17) {
                if let Some(found) = deposit(seed, x, y) {
                    seen[found.kind] = true;
                }
            }
        }
        for (index, kind) in DEPOSIT_KINDS.iter().enumerate() {
            assert!(seen[index], "`{}` is declared but never occurs", kind.substance);
            assert!(
                crate::materials::schema::substance(kind.substance).is_some(),
                "`{}` is a deposit and not a substance in the table",
                kind.substance
            );
        }
    }

    /// A patch holds one substance, because `kind_at` reads a coarser field than richness.
    #[test]
    fn a_patch_holds_one_substance() {
        let seed = 0xC117_2026;
        let mut same_as_the_next_tile = 0;
        let mut compared = 0;
        for x in 0..512 {
            for y in (0..512).step_by(5) {
                if let (Some(left), Some(right)) =
                    (deposit(seed, x, y), deposit(seed, x + 1, y))
                {
                    compared += 1;
                    if left.kind == right.kind {
                        same_as_the_next_tile += 1;
                    }
                }
            }
        }
        assert!(compared > 100, "the sample found too few adjacent deposits");
        assert!(
            same_as_the_next_tile as f64 / compared as f64 > 0.95,
            "kinds are being re-sliced every tile: {same_as_the_next_tile}/{compared}"
        );
    }
}
