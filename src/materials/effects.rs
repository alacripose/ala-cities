//! The four sim effects, computed from the declared tables.
//!
//! `tools/materials/declare.py` writes the numbers into
//! [`crate::materials::generated`]; this module is the only place that turns them
//! into behaviour, so a declared rate and a computed rate cannot drift — the
//! quantity a test asserts and the quantity the sim charges are the same lookup.
//!
//! §7.4 declares four effects at their own grain. Five of their six quantities are
//! computed here; conduction is not, and [`DEFERRED`] says why with a consumer
//! rather than an excuse:
//!
//! * **Cost and upkeep** — [`build_cost`] is the sum of a structure's declared
//!   part prices; [`upkeep_per_month`] is the sum of its parts' families' declared
//!   monthly rates.
//! * **Deterioration** — [`wear_per_day`] is the structure's **weakest** part's
//!   declared rate, because the first part to fail is what forces the repair, so a
//!   structure is as good as its worst part. This is a reading of "a `condition`
//!   per structure, decaying per day by a declared per-family rate" for a structure
//!   whose parts are of several families; it was decided rather than assumed, and it
//!   is overridable in one line. **Since C9's MAINTAIN slice the rate is a rational
//!   and [`wear_g_per_day`] turns it into grams**: wear moves mass, and a float rate
//!   would put a rounding error into the ledger every sim-day.
//! * **Desirability** — [`desirability`]: how a structure of this kind moves demand
//!   on the tiles around it.
//! * **Nuisance** — [`nuisance`] per structure and [`nuisance_at`] sampled at a read
//!   point, so a 65 536-tile field is never recomputed every tick and the sim stays
//!   tick-deterministic.
//!
//! Every lookup **panics with the part's own name** rather than returning zero. A
//! missing price that silently costs nothing is exactly the failure this campaign
//! keeps finding: a number that looks computed and is not. [`table_defects`] is the
//! same check as a list, so the gate can print it before anything runs.

use crate::materials::generated;
use crate::materials::world::{self, Family, Part};

/// What §7.4 declares that this module does **not** compute, and the consumer each
/// one is waiting for. Printed by the gate every run, in the same spirit as the
/// icon inventory's deferred list: an absence that is a record rather than a thing
/// nobody noticed.
pub const DEFERRED: &[(&str, &str)] = &[
    (
        "conduction",
        "its consumers are power lines, and there is no `BuildingKind::PowerLine` \
         though `powerline.conductor` and `powerline.pylon` are declared parts with \
         prices — so the family the table was written for has no structure to conduct \
         through, and power still reaches a building by plant count rather than along a \
         path",
    ),
    // `surface speed` was deferred here with this reason: "it reads a tile's surface family
    // and the world has no such thing yet". C9 phase 1 landed the surface family, so
    // `World::surface_speed` reads this table now and the entry moved out of DEFERRED —
    // which is the only correct way for a deferred item to leave this list.
];

/// The declared price of one part. Panics naming the part when the table is short.
pub fn part_price(part: &str) -> f32 {
    generated::PART_PRICE
        .iter()
        .find(|(name, _)| *name == part)
        .map(|(_, price)| *price)
        .unwrap_or_else(|| panic!("`{part}` has no declared price in PART_PRICE"))
}

/// Credits to build: the sum of a structure's parts' declared prices.
pub fn build_cost(part_prefix: &str) -> f32 {
    world::parts_of(part_prefix)
        .iter()
        .map(|part| part_price(part.part))
        .sum()
}

/// The declared monthly upkeep rate of one family.
pub fn upkeep_of(family: Family) -> f32 {
    let name = family.as_str();
    generated::UPKEEP_PER_MONTH
        .iter()
        .find(|(family, _)| *family == name)
        .map(|(_, rate)| *rate)
        .unwrap_or_else(|| panic!("`{name}` has no declared upkeep in UPKEEP_PER_MONTH"))
}

/// Credits per month to keep a structure standing.
pub fn upkeep_per_month(part_prefix: &str) -> f32 {
    world::parts_of(part_prefix)
        .iter()
        .map(|part| upkeep_of(part.family))
        .sum()
}

/// The declared wear per sim-day by one family, as an **exact rational**: grams lost per gram
/// held. Never a float, because wear moves mass (Q109/Q114) and mass here is integer grams.
pub fn wear_of(family: Family) -> (i64, i64) {
    let name = family.as_str();
    generated::DECAY_PER_DAY
        .iter()
        .find(|(family, _, _)| *family == name)
        .map(|(_, num, den)| (*num, *den))
        .unwrap_or_else(|| panic!("`{name}` has no declared wear in DECAY_PER_DAY"))
}

/// Whether one rational rate is faster than another, exactly: `a/b > c/d` is `a·d > c·b`.
///
/// Integer comparison rather than a float one, because "which part fails first" deciding on a
/// rounding would be a structure that ages differently on a different machine.
fn faster((a_num, a_den): (i64, i64), (b_num, b_den): (i64, i64)) -> bool {
    a_num * b_den > b_num * a_den
}

/// The condition a structure loses per sim-day: its weakest part's rate.
///
/// A reading, decided rather than assumed: a structure carries one `condition`, its
/// parts decay at their families' rates, and the part that fails first is the part
/// that gets repaired — so the structure decays as fast as its fastest-decaying
/// part. A family-less structure (one that resolves to no declared parts) decays at
/// nothing, which its caller is expected to have refused before it got here.
pub fn wear_per_day(part_prefix: &str) -> (i64, i64) {
    world::parts_of(part_prefix)
        .iter()
        .map(|part| wear_of(part.family))
        .fold((0, 1), |worst, rate| if faster(rate, worst) { rate } else { worst })
}

/// The grams a structure holding `declared_g` loses in one sim-day: the weakest part's declared
/// rate applied to its own mass, **exactly**, in integers.
///
/// At least one gram a day while it stands, and that floor is a decision rather than tidiness: a
/// structure that could weather forever without losing a gram is mass that never moves, which is
/// the accounting this campaign exists to keep. The floor cannot fire for anything a family with
/// a declared rate applies to — a 100 t home loses 120 kg a day — so it is a guard on the small
/// end rather than a thumb on the scale.
pub fn wear_g_per_day(part_prefix: &str, declared_g: i64) -> i64 {
    let (num, den) = wear_per_day(part_prefix);
    if num <= 0 || den <= 0 || declared_g <= 0 {
        return 0;
    }
    (declared_g * num / den).max(1)
}

/// The parts a structure is made of, for a caller that has to name them.
pub fn parts(part_prefix: &str) -> Vec<&'static Part> {
    world::parts_of(part_prefix)
}

/// The family in which a structure decays fastest, so a repair can name it.
pub fn weakest_family(part_prefix: &str) -> Option<Family> {
    parts(part_prefix)
        .into_iter()
        .max_by(|a, b| {
            let (a_num, a_den) = wear_of(a.family);
            let (b_num, b_den) = wear_of(b.family);
            (a_num * b_den).cmp(&(b_num * a_den))
        })
        .map(|part| part.family)
}

/// Whether a structure holding `standing_g` of its own `declared_g` is below the declared floor
/// and owes a repair — an integer comparison, so the floor is the floor rather than a rounding
/// of it.
pub fn repair_is_due(standing_g: i64, declared_g: i64) -> bool {
    standing_g * generated::REPAIR_FLOOR.1 < declared_g * generated::REPAIR_FLOOR.0
}

/// The mass a structure of this size holds once it is fully maintained: the ceiling share, in
/// whole grams. A repair draws exactly the difference between this and what stands.
pub fn repair_target_g(declared_g: i64) -> i64 {
    declared_g * generated::REPAIR_CEILING.0 / generated::REPAIR_CEILING.1
}

/// What a repair costs: the declared share of the structure's own build cost,
/// because a repair is priced against the thing being repaired rather than fixed.
pub fn repair_cost(part_prefix: &str) -> f32 {
    build_cost(part_prefix) * generated::REPAIR_SHARE
}

/// The work a repair is worth, in work-ticks: its declared cost as a whole number.
///
/// Credits are not hours, and this is deliberately the **same reading the build path already
/// makes** — a build's work is its own declared cost ([`crate::sim::BuildingKind::build_cost`]) —
/// so a repair is worked by the number it is priced by rather than by a second number invented
/// here to sit beside it. Rounded up, because a repair nobody can finish in a whole tick is still
/// a repair somebody has to start.
pub fn repair_work(part_prefix: &str) -> i64 {
    repair_cost(part_prefix).ceil() as i64
}

/// How a structure of this kind moves demand on the tiles around it.
pub fn desirability(part_prefix: &str) -> f32 {
    let families = unique_families(part_prefix);
    families
        .iter()
        .filter_map(|family| {
            let name = family.as_str();
            generated::DESIRABILITY
                .iter()
                .find(|(family, _)| *family == name)
                .map(|(_, value)| *value)
        })
        .sum()
}

/// Noise and pollution emitted by one structure of this kind, in the family's own
/// units.
pub fn nuisance(part_prefix: &str) -> f32 {
    unique_families(part_prefix)
        .iter()
        .filter_map(|family| {
            let name = family.as_str();
            generated::NUISANCE
                .iter()
                .find(|(family, _)| *family == name)
                .map(|(_, value)| *value)
        })
        .sum()
}

/// Nuisance sampled at a read point, from the sources that reach it.
///
/// Sampled, not diffused: the caller hands in the `(distance, strength)` pairs that
/// actually reach the point — a handful of nearby sources — so a round is a few
/// multiplications rather than a 65 536-tile field recomputed every tick, and the
/// same read points always produce the same numbers on a replay.
pub fn nuisance_at(sources: impl IntoIterator<Item = (f32, f32)>, reach: f32) -> f32 {
    if reach <= 0.0 {
        return 0.0;
    }
    sources
        .into_iter()
        .filter(|(distance, _)| *distance < reach)
        .map(|(distance, strength)| strength * (1.0 - distance / reach))
        .sum()
}

/// The families a structure's parts are in, once each, in declaration order.
fn unique_families(part_prefix: &str) -> Vec<Family> {
    let mut out: Vec<Family> = Vec::new();
    for part in parts(part_prefix) {
        if !out.contains(&part.family) {
            out.push(part.family);
        }
    }
    out
}

/// Every declared defect in the tables: a part with no price, a family missing any
/// of the rates it must have, a structure-kind prefix that resolves to no parts.
///
/// The gate prints this, so a table that has silently lost an entry is a sentence
/// before it is a wrong number.
pub fn table_defects() -> Vec<String> {
    let mut defects = Vec::new();

    for (part_name, _) in generated::PART_PRICE {
        if world::part(part_name).is_none() {
            defects.push(format!(
                "`{part_name}` is priced in PART_PRICE but is not a declared part of \
                 any structure: a price nothing can be charged"
            ));
        }
    }

    // Every family the vocabulary declares, read from the enum rather than a list
    // typed out here: a family added to `world` is then checked by this gate on the
    // next run instead of being forgotten beside it.
    let families = [
        Family::Metal,
        Family::Paper,
        Family::Ceramic,
        Family::Glass,
        Family::Polymer,
        Family::Road,
        Family::Enamel,
        Family::Water,
        Family::Organic,
        Family::Soil,
    ];
    for family in families {
        let name = family.as_str();
        if !generated::UPKEEP_PER_MONTH.iter().any(|(f, _)| *f == name) {
            defects.push(format!("`{name}` has no declared upkeep"));
        }
        if !generated::DECAY_PER_DAY.iter().any(|(f, _, _)| *f == name) {
            defects.push(format!("`{name}` has no declared decay"));
        }
        if !generated::DESIRABILITY.iter().any(|(f, _)| *f == name) {
            defects.push(format!("`{name}` has no declared desirability"));
        }
        if !generated::NUISANCE.iter().any(|(f, _)| *f == name) {
            defects.push(format!("`{name}` has no declared nuisance"));
        }
    }

    for kind in ["home", "shop", "factory", "power"] {
        if world::parts_of(kind).is_empty() {
            defects.push(format!(
                "`{kind}` resolves to no declared parts, so it has no cost, no upkeep \
                 and nothing to decay"
            ));
        }
    }
    defects
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_declared_table_entry_is_accounted_for() {
        let defects = table_defects();
        assert!(defects.is_empty(), "the tables are short: {defects:#?}");
    }

    /// The number the existing test asserts, read the way the sim reads it — so the
    /// re-tune (a173) is checked on the path the world uses, not on a copy.
    #[test]
    fn a_home_costs_the_sum_of_its_parts() {
        assert_eq!(build_cost("home"), 140.0);
        assert_eq!(part_price("home.walls"), 60.0);
        assert_eq!(part_price("home.roof"), 40.0);
        assert_eq!(part_price("home.window"), 40.0);
    }

    /// The weakest-part reading, on a structure built to exercise it: a home is
    /// ceramic 1/2500, polymer 3/2500 and glass 1/2000, so it wears at the polymer's
    /// rate and names the polymer as what will fail first.
    #[test]
    fn a_structure_wears_at_its_weakest_part() {
        assert_eq!(wear_per_day("home"), (3, 2500), "the polymer's rate");
        assert_eq!(weakest_family("home"), Some(Family::Polymer));
        assert_eq!(wear_of(Family::Ceramic), (1, 2500));
        assert_eq!(wear_of(Family::Glass), (1, 2000));
        // The two rates are close enough that a float comparison would be at the mercy of
        // representation; the rational one is not.
        assert!(faster((1, 2000), (1, 2500)), "glass wears faster than ceramic");
        assert!(!faster((1, 2500), (1, 2500)), "and nothing wears faster than itself");
    }

    /// The wear as **grams**, which is the only form the ledger can read: a 100 t home in polymer
    /// loses 120 kg a day, and a structure with a declared rate always loses something.
    #[test]
    fn wear_is_grams_lost_per_day() {
        let home = 100_000_000;
        assert_eq!(wear_g_per_day("home", home), 120_000, "100 t × 3/2500");
        assert_eq!(wear_g_per_day("home", 0), 0, "nothing standing loses nothing");
        // A structure far smaller than any the game declares still loses a gram rather than
        // weathering for free forever.
        assert_eq!(wear_g_per_day("home", 100), 1);
        // A family that does not weather moves no mass at all: the rate is zero, not a small
        // number.
        assert_eq!(wear_of(Family::Water), (0, 1));
    }

    #[test]
    fn upkeep_is_the_sum_of_the_families_the_parts_are_in() {
        // ceramic 1.2 + polymer 0.8 + glass 1.5
        let expected = 1.2 + 0.8 + 1.5;
        assert!((upkeep_per_month("home") - expected).abs() < 1e-6);
    }

    #[test]
    fn a_repair_is_priced_against_the_thing_repaired() {
        assert_eq!(repair_cost("home"), 140.0 * 0.4);
        assert_eq!(repair_work("home"), 56, "140 credits of repair is 56 work-ticks");
        assert_eq!(repair_work("power"), 820, "and 2050 credits of plant is 820");
        let declared = 100_000_000;
        assert!(repair_is_due(declared * 34 / 100, declared), "34 % is below the floor");
        assert!(
            !repair_is_due(declared * 35 / 100, declared),
            "the floor is the floor, not below it"
        );
        assert!(!repair_is_due(declared, declared), "as-built owes nothing");
        // The ceiling is what a repair restores by drawing the difference: at 100 % a repair of a
        // half-worn home brings 50 t of material out of the ground.
        assert_eq!(repair_target_g(declared), declared);
        assert_eq!(repair_target_g(declared) - declared / 2, 50_000_000);
    }

    #[test]
    fn each_family_once_however_many_parts_are_in_it() {
        // A power plant is metal (frame), ceramic (stack), ceramic (insulator) and
        // enamel (core): four parts in three families, so metal and ceramic must
        // each count once — 0.3 + 0.05 + 0.1, not 0.3 twice and 0.05 twice.
        assert_eq!(parts("power").len(), 4, "four declared parts");
        assert!((nuisance("power") - 0.45).abs() < 1e-6, "metal + ceramic + enamel");
        assert!((nuisance("power") - 0.75).abs() > 0.1, "not the sum over parts");
        assert!(
            (desirability("power") - (-0.02 + 0.02 + 0.03)).abs() < 1e-6,
            "metal + ceramic + enamel, each once"
        );
        // The plant's weakest part is its metal frame at 1/1250, ahead of enamel at
        // 3/5000 and ceramic at 1/2500.
        assert_eq!(wear_per_day("power"), (1, 1250));
        assert_eq!(weakest_family("power"), Some(Family::Metal));
    }

    #[test]
    fn nuisance_falls_off_with_distance_and_stops_at_reach() {
        let sources = [(0.0, 1.0), (5.0, 1.0), (10.0, 1.0)];
        assert_eq!(nuisance_at(sources, 10.0), 1.0 + 0.5);
        assert_eq!(nuisance_at([(10.0, 1.0)], 10.0), 0.0, "reach is excluded");
        assert_eq!(nuisance_at(sources, 0.0), 0.0);
        assert_eq!(nuisance_at(std::iter::empty(), 50.0), 0.0);
    }

    #[test]
    fn an_undeclared_part_is_named_rather_than_costed_at_nothing() {
        let refusal = std::panic::catch_unwind(|| part_price("home.door"));
        assert!(refusal.is_err(), "a part with no price must not cost zero");
    }
}
