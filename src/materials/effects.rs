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
//! * **Deterioration** — [`decay_per_day`] is the structure's **weakest** part's
//!   declared rate, because the first part to fail is what forces the repair, so a
//!   structure is as good as its worst part. This is a reading of "a `condition`
//!   per structure, decaying per day by a declared per-family rate" for a structure
//!   whose parts are of several families; it was decided rather than assumed, and it
//!   is overridable in one line.
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
    (
        "surface speed",
        "it reads a tile's surface family and the world has no such thing yet: \
         `RoadGraph::rebuild` inserts only tiles where `road` is true, so every graph \
         node is already the table's own `road: 1.0` and every other entry is \
         unreachable. It lands with the per-tile surface layer",
    ),
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

/// The declared condition lost per sim-day by one family.
pub fn decay_of(family: Family) -> f32 {
    let name = family.as_str();
    generated::DECAY_PER_DAY
        .iter()
        .find(|(family, _)| *family == name)
        .map(|(_, rate)| *rate)
        .unwrap_or_else(|| panic!("`{name}` has no declared decay in DECAY_PER_DAY"))
}

/// The condition a structure loses per sim-day: its weakest part's rate.
///
/// A reading, decided rather than assumed: a structure carries one `condition`, its
/// parts decay at their families' rates, and the part that fails first is the part
/// that gets repaired — so the structure decays as fast as its fastest-decaying
/// part. A family-less structure (one that resolves to no declared parts) decays at
/// nothing, which its caller is expected to have refused before it got here.
pub fn decay_per_day(part_prefix: &str) -> f32 {
    world::parts_of(part_prefix)
        .iter()
        .map(|part| decay_of(part.family))
        .fold(0.0_f32, f32::max)
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
            decay_of(a.family)
                .partial_cmp(&decay_of(b.family))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|part| part.family)
}

/// Whether a structure is below the declared repair floor and owes a repair.
pub fn repair_is_due(condition: f32) -> bool {
    condition < generated::REPAIR_FLOOR
}

/// What a repair costs: the declared share of the structure's own build cost,
/// because a repair is priced against the thing being repaired rather than fixed.
pub fn repair_cost(part_prefix: &str) -> f32 {
    build_cost(part_prefix) * generated::REPAIR_SHARE
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
        if !generated::DECAY_PER_DAY.iter().any(|(f, _)| *f == name) {
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
    /// ceramic 0.0004, polymer 0.0012 and glass 0.0005, so it decays at the
    /// polymer's rate and names the polymer as what will fail first.
    #[test]
    fn a_structure_decays_at_its_weakest_part() {
        assert_eq!(decay_per_day("home"), 0.0012);
        assert_eq!(weakest_family("home"), Some(Family::Polymer));
        assert_eq!(decay_of(Family::Ceramic), 0.0004);
        assert_eq!(decay_of(Family::Glass), 0.0005);
        // Not the mean, which would be 0.0007 and would let a structure outlive the
        // part that actually fails.
        let mean = (0.0004 + 0.0012 + 0.0005) / 3.0;
        assert!((decay_per_day("home") - mean).abs() > 0.0004);
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
        assert!(repair_is_due(0.34));
        assert!(!repair_is_due(0.35), "the floor is the floor, not below it");
        assert!(!repair_is_due(1.0));
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
        // The plant's weakest part is its metal frame at 0.0008, ahead of enamel at
        // 0.0006 and ceramic at 0.0004.
        assert_eq!(decay_per_day("power"), 0.0008);
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
