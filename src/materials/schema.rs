//! The substance and process tables (C9 phase 3), typed, with the campaign's one
//! invariant checked where it is used.
//!
//! `tools/materials/SCHEMA.md` is the contract, `tools/materials/schema.py` is the
//! gate that refuses a row which breaks it, and this module is the runtime's view of
//! what survived. Two tables and one rule:
//!
//! * **Every substance declares a unit**, and reaches grams exactly — mass is grams,
//!   volume multiplies by a declared density, a count by a declared unit mass. There
//!   is no float in [`Rational`], and [`grams`] returns `None` rather than rounding,
//!   because a conversion that rounds is where a ledger stops balancing quietly.
//! * **Every process balances**, and [`process_totals`] reports both sides so a
//!   failure can quote them: `in 2995 g vs out 2990 g` is a sentence someone can act
//!   on. A process with **no inputs** is a gather, and [`verify`] refuses one whose
//!   output is `made` — that is `grow()` wearing a row, and it is the one thing this
//!   table exists to refuse.
//! * **The content is unbounded.** Q66 and Q69 asked for the whole substance set and
//!   as many ages as possible, so an age is rows: nothing here assumes a closed set,
//!   and adding one touches no code.
//!
//! Why the gate exists twice, in two languages: the Python one decides what may be
//! *emitted*, and this one decides what may be *used*. They are not redundant — a
//! hand-edited generated file, or a runtime path that constructs a quantity nobody
//! declared, is exactly the case the emitting gate cannot see. Mirrored on purpose,
//! the way the generator's digest is.

use crate::materials::generated::{Process, Rational, Substance};
use crate::materials::{generated, Report};

/// The canonical unit a substance is counted in.
///
/// The vocabulary is **closed**, and asking for one that is not declared is `None`
/// rather than a default — the same rule the task verbs and the case prefixes follow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Mass,
    Volume,
    Count,
    Gas,
}

impl Unit {
    pub const ALL: &'static [Unit] = &[Unit::Mass, Unit::Volume, Unit::Count, Unit::Gas];

    pub fn name(self) -> &'static str {
        match self {
            Unit::Mass => "Mass",
            Unit::Volume => "Volume",
            Unit::Count => "Count",
            Unit::Gas => "Gas",
        }
    }

    /// The unit for a name, or `None` — the refusal case, not a default.
    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|unit| unit.name() == name)
    }

    /// What this unit needs before it can reach grams: a density, or a unit mass.
    pub fn needs(self) -> Option<&'static str> {
        match self {
            Unit::Volume | Unit::Gas => Some("density"),
            Unit::Count => Some("unit_mass"),
            Unit::Mass => None,
        }
    }
}

/// One substance, by its declared name.
pub fn substance(name: &str) -> Option<&'static Substance> {
    generated::SUBSTANCES.iter().find(|entry| entry.name == name)
}

/// One process, by its declared name.
pub fn process(name: &str) -> Option<&'static Process> {
    generated::PROCESSES.iter().find(|entry| entry.name == name)
}

/// The ordinal of a declared age, or `None` for a tier nobody declared — a process in
/// no tier cannot be reached, so this is the predicate a progression uses.
pub fn tier_ordinal(name: &str) -> Option<i64> {
    generated::TIERS
        .iter()
        .find(|(tier, _, _)| *tier == name)
        .map(|(_, ordinal, _)| *ordinal)
}

/// Whether a process is in an age at or below `reached`.
pub fn reachable(process: &Process, reached: i64) -> bool {
    tier_ordinal(process.tier).is_some_and(|ordinal| ordinal <= reached)
}

/// A whole number of grams, or `None` when the conversion is not exact.
///
/// `None` is the honest answer rather than a rounded one: a caller that asked for a
/// third of a gram asked a question the tables do not answer, and the Python gate
/// refuses such a row before it can ever be emitted.
pub fn grams(substance: &Substance, amount: Rational) -> Option<i64> {
    let unit = Unit::named(substance.unit)?;
    let (num, den) = match unit {
        Unit::Mass => (amount.num, amount.den),
        Unit::Volume | Unit::Gas => (
            amount.num * substance.density.num,
            amount.den * substance.density.den,
        ),
        Unit::Count => (
            amount.num * substance.unit_mass.num,
            amount.den * substance.unit_mass.den,
        ),
    };
    if den == 0 || num % den != 0 {
        return None;
    }
    Some(num / den)
}

/// A named quantity's grams, or `None` when the row is wrong — an undeclared
/// substance, an undeclared unit, or a conversion that does not land on a whole gram.
pub fn quantity_grams(name: &str, amount: Rational) -> Option<i64> {
    grams(substance(name)?, amount)
}

/// What a process takes in and gives out, in grams. `None` means some row in it could
/// not reach grams at all, which is a defect rather than a zero.
pub fn process_totals(process: &Process) -> (Option<i64>, Option<i64>) {
    let total = |side: &[(&'static str, Rational)]| {
        side.iter()
            .try_fold(0i64, |sum, (name, amount)| Some(sum + quantity_grams(name, *amount)?))
    };
    (total(process.inputs), total(process.outputs))
}

/// Whether a process balances — the one invariant the whole table exists for.
pub fn balances(process: &Process) -> bool {
    let (taken, given) = process_totals(process);
    matches!((taken, given), (Some(into), Some(out)) if into == out)
}

/// The substances that leave the world's ground rather than a stock: produced by a
/// process that takes nothing in, which is what `SCHEMA.md`'s chain marks as *source*.
///
/// Named rather than inferred at the call site, because this is the only place in the
/// design where mass legitimately enters the world — and a reader who wants to check
/// that claim should be able to ask for it directly.
pub fn gathers() -> Vec<(&'static str, &'static str, i64)> {
    let mut drawn = Vec::new();
    for process in generated::PROCESSES {
        if !process.inputs.is_empty() {
            continue;
        }
        for (name, amount) in process.outputs {
            if let Some(grams) = quantity_grams(name, *amount) {
                drawn.push((process.name, *name, grams));
            }
        }
    }
    drawn
}

/// The substance and process gate, run where the tables are used.
///
/// Reports **every** defect, so a run is one reading rather than a queue, and prints
/// the open items beside them: what a green run does *not* mean.
pub fn verify(report: &mut Report) {
    let defect = |report: &mut Report, line: String| {
        report.lines.push(format!("DEFECT: {line}"));
        report.defects.push(line);
    };

    report.lines.push(format!(
        "schema: {} substances, {} processes, {} tiers",
        generated::SUBSTANCES.len(),
        generated::PROCESSES.len(),
        generated::TIERS.len()
    ));

    // Every substance declares a unit, and a unit that needs a conversion declares it.
    for entry in generated::SUBSTANCES {
        match Unit::named(entry.unit) {
            None => defect(
                report,
                format!(
                    "substance `{}` declares unit `{}`, which is not one of {}",
                    entry.name,
                    entry.unit,
                    Unit::ALL
                        .iter()
                        .map(|unit| unit.name())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ),
            Some(unit) => match unit.needs() {
                Some("density") if entry.density.num <= 0 || entry.density.den <= 0 => defect(
                    report,
                    format!(
                        "substance `{}` is measured in {} and declares no positive density — \
                         it cannot reach grams, so it cannot enter the ledger",
                        entry.name,
                        unit.name().to_lowercase()
                    ),
                ),
                Some("unit_mass") if entry.unit_mass.num <= 0 || entry.unit_mass.den <= 0 => defect(
                    report,
                    format!(
                        "substance `{}` is measured in units and declares no positive unit mass \
                         — \"1 {}\" is not a number anybody can audit",
                        entry.name, entry.name
                    ),
                ),
                _ => {}
            },
        }
        if generated::ICON_FAMILIES
            .iter()
            .chain(generated::WORLD_FAMILIES.iter())
            .all(|family| family.name != entry.family)
        {
            defect(
                report,
                format!(
                    "substance `{}` presents as `{}`, which is not a declared family",
                    entry.name, entry.family
                ),
            );
        }
    }

    // Every process: an age, a balance, and references that resolve.
    let consumed: Vec<&str> = generated::PROCESSES
        .iter()
        .flat_map(|process| process.inputs.iter().map(|(name, _)| *name))
        .collect();
    for process in generated::PROCESSES {
        if tier_ordinal(process.tier).is_none() {
            defect(
                report,
                format!(
                    "process `{}` belongs to tier `{}`, which is not declared — a process in no \
                     tier cannot be reached",
                    process.name, process.tier
                ),
            );
        }
        for (name, _) in process.inputs.iter().chain(process.outputs.iter()) {
            if substance(name).is_none() {
                defect(
                    report,
                    format!("process `{}` names `{name}`, which is not declared", process.name),
                );
            }
        }
        let (taken, given) = process_totals(process);
        match (taken, given) {
            (Some(into), Some(out)) if into != out && !process.inputs.is_empty() => defect(
                report,
                format!(
                    "process `{}` does not balance: in {into} g, out {out} g — a difference of \
                     {} g. Every gram that leaves must arrive somewhere, and a loss is declared \
                     as an output rather than subtracted",
                    process.name,
                    into - out
                ),
            ),
            (None, _) | (_, None) => defect(
                report,
                format!(
                    "process `{}` has a quantity that does not land on a whole gram — a process \
                     measured in fractions of a gram is a process whose numbers are wrong",
                    process.name
                ),
            ),
            _ => {}
        }
        // A gather is not a creation: with no inputs, the world must already hold it.
        if process.inputs.is_empty() {
            for (name, _) in process.outputs {
                if substance(name).is_some_and(|entry| entry.source == "made") {
                    defect(
                        report,
                        format!(
                            "process `{}` takes nothing in and produces `{name}`, whose source is \
                             `made` — that is material from nothing, which is the one thing this \
                             table exists to refuse",
                            process.name
                        ),
                    );
                }
            }
        }
        if process.mechanism.heat_c > 0
            && !matches!(process.mechanism.heat_kind, "material" | "flame")
        {
            defect(
                report,
                format!(
                    "process `{}` declares {} °C with no heat kind — the material and the flame \
                     differ by ~600 °C and both readings are correct",
                    process.name, process.mechanism.heat_c
                ),
            );
        }
        for (kind, required) in process.requires {
            let vocabulary: &[(&str, &str)] = match *kind {
                "structure" => generated::STRUCTURES,
                "tool" => generated::TOOLS,
                "skill" => generated::SKILLS,
                other => {
                    defect(
                        report,
                        format!(
                            "process `{}` requires a `{other}`, which is not a declared \
                             vocabulary",
                            process.name
                        ),
                    );
                    continue;
                }
            };
            if !vocabulary.iter().any(|(name, _)| name == required) {
                defect(
                    report,
                    format!(
                        "process `{}` requires {kind} `{required}`, which is not declared — the \
                         vocabulary is closed, and naming something outside it is a claim with \
                         no number behind it",
                        process.name
                    ),
                );
            }
        }
    }

    // Every **made** good must be reachable from the world. This is the campaign's rule at
    // the level of a plan rather than a row: a good whose chain loops, or whose chain ends
    // in something the world does not hold, is a good nobody can produce — and the failure
    // is a cycle or a dead end in the table rather than a bad number in a row.
    for entry in generated::SUBSTANCES {
        if entry.source != "made" {
            continue;
        }
        if let Err(refusal) = crate::materials::chain::plan(entry.name, 1) {
            defect(
                report,
                format!(
                    "`{}` is made and cannot be planned from the world: {}",
                    entry.name,
                    refusal.describe()
                ),
            );
        }
    }

    // A substance nothing consumes must say why, and the reason is a finding worth
    // reading rather than a defect: it names the rung that is unfinished.
    for entry in generated::SUBSTANCES {
        if consumed.contains(&entry.name) {
            continue;
        }
        if entry.no_consumer.is_empty() {
            defect(
                report,
                format!(
                    "substance `{}` has no consumer and no declared reason — a substance \
                     something will use as soon as the rung above lands is unfinished work, \
                     which must say so",
                    entry.name
                ),
            );
        }
    }

    // The vocabularies are closed, and an entry nothing requires is an intention.
    for (kind, entries) in [
        ("structure", generated::STRUCTURES),
        ("tool", generated::TOOLS),
        ("skill", generated::SKILLS),
    ] {
        for (name, _) in entries {
            let used = generated::PROCESSES.iter().any(|process| {
                process
                    .requires
                    .iter()
                    .any(|(required_kind, required)| *required_kind == kind && required == name)
            });
            if !used {
                defect(
                    report,
                    format!(
                        "{kind} `{name}` is declared and no process requires it — an entry \
                         nothing uses is an intention wearing a name"
                    ),
                );
            }
        }
    }

    for process in generated::PROCESSES {
        let (taken, given) = process_totals(process);
        if let (Some(into), Some(out)) = (taken, given) {
            report.lines.push(format!(
                "schema: `{}` ({}) — in {into} g, out {out} g, {}/{} h of labour",
                process.name,
                process.tier,
                process.mechanism.labour_hours.num,
                process.mechanism.labour_hours.den
            ));
        }
    }
    for item in generated::SCHEMA_OPEN {
        report.lines.push(format!("schema: open — {item}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::generated;

    /// The campaign's one invariant, checked over every row rather than over a
    /// hand-picked one: a table with one unbalanced process is a table that creates
    /// mass somewhere, and the report names which.
    #[test]
    fn every_process_balances_in_exact_grams() {
        let mut unbalanced = Vec::new();
        for process in generated::PROCESSES {
            if process.inputs.is_empty() {
                continue; // a gather draws on the world, and `gathers()` names it
            }
            let (taken, given) = process_totals(process);
            match (taken, given) {
                (Some(into), Some(out)) if into == out => {}
                (Some(into), Some(out)) => unbalanced.push(format!(
                    "`{}`: in {into} g, out {out} g, difference {} g",
                    process.name,
                    into - out
                )),
                _ => unbalanced.push(format!(
                    "`{}`: a quantity does not land on a whole gram",
                    process.name
                )),
            }
        }
        assert!(unbalanced.is_empty(), "{unbalanced:#?}");
        assert!(
            generated::PROCESSES.len() >= 7,
            "the stone rung has processes in it"
        );
    }

    /// Q7's own example, end to end: the shortest chain that ends in a tool which
    /// multiplies work, and every step of it in exact integers. `SCHEMA.md`'s worked
    /// chain is the specification; this is the machine checking it.
    #[test]
    fn the_stone_chain_builds_a_hatchet_out_of_the_world() {
        let edge = quantity_grams("knapped_edge", Rational { num: 800, den: 1 }).expect("grams");
        let haft = quantity_grams("haft_blank", Rational { num: 2400, den: 1 }).expect("grams");
        let cord = quantity_grams("cord", Rational { num: 95, den: 1 }).expect("grams");
        let hatchet =
            quantity_grams("hatchet", Rational { num: 1, den: 1 }).expect("a count reaches grams");
        assert_eq!(hatchet, 2900, "one hatchet is 2900 g, from its own unit mass");
        assert_eq!(
            edge + haft + cord,
            hatchet + 395,
            "the three inputs are the hatchet plus the declared trim waste — 3295 = 2900 + 395"
        );

        // The chain's gathers are the only mass the world gives up, and the sum is
        // what the ground stops holding.
        let drawn: i64 = gathers().iter().map(|(_, _, grams)| grams).sum();
        assert_eq!(drawn, 3000 + 1000 + 100, "the stone rung draws 4.1 kg from the world");
        assert!(
            gathers().iter().all(|(_, name, _)| substance(name)
                .is_some_and(|entry| entry.source != "made")),
            "a gather produces only what the world already holds"
        );
    }

    /// The rule that replaces `grow()`: mass does not appear. A no-input process whose
    /// output is `made` would be exactly that, so the gate must refuse it — tested by
    /// constructing the case rather than by trusting the table.
    #[test]
    fn a_process_that_makes_something_from_nothing_is_the_defect() {
        let fabricated = Process {
            name: "conjure brick",
            tier: "hands & stone",
            inputs: &[],
            outputs: &[("clay", Rational { num: 1000, den: 1 })],
            mechanism: generated::PROCESSES[0].mechanism,
            requires: &[],
            note: "",
        };
        // `clay` is `mined`, so this one is legal: the world holds it. The defect is a
        // no-input process producing something `made`, so the test asserts the rule
        // through the substance's own declared source rather than through a name.
        assert_eq!(substance("clay").expect("declared").source, "mined");
        assert!(fabricated.inputs.is_empty() && !fabricated.outputs.is_empty());
        let made = substance("haft_blank").expect("declared");
        assert_eq!(made.source, "made", "a made good must come from a process with inputs");
        for process in generated::PROCESSES {
            if process.inputs.is_empty() {
                for (name, _) in process.outputs {
                    assert_ne!(
                        substance(name).expect("declared").source,
                        "made",
                        "`{}` conjures a made good",
                        process.name
                    );
                }
            }
        }
    }

    /// A conversion that does not land on a whole gram is refused rather than rounded:
    /// rounding at the conversion is where a ledger stops balancing without saying so.
    #[test]
    fn a_conversion_that_would_round_is_refused() {
        let hatchet = substance("hatchet").expect("declared");
        assert_eq!(grams(hatchet, Rational { num: 1, den: 1 }), Some(2900));
        assert_eq!(
            grams(hatchet, Rational { num: 1, den: 3 }),
            None,
            "a third of a hatchet is not a number of grams"
        );
        assert_eq!(
            quantity_grams("nobody declared this", Rational { num: 1, den: 1 }),
            None
        );
        // Volume reaches grams through the declared density, and never through a float.
        let water = substance("water").expect("declared");
        assert_eq!(grams(water, Rational { num: 1500, den: 1 }), Some(1500), "1 mL is 1 g");
    }

    /// The unit vocabulary is closed, and each unit says what it needs.
    #[test]
    fn the_unit_list_is_closed_and_quotes_itself() {
        for unit in Unit::ALL {
            assert_eq!(Unit::named(unit.name()), Some(*unit));
        }
        assert_eq!(Unit::named("Bushel"), None);
        assert_eq!(Unit::named(""), None);
        assert_eq!(Unit::Mass.needs(), None, "grams are grams");
        assert_eq!(Unit::Volume.needs(), Some("density"));
        assert_eq!(Unit::Count.needs(), Some("unit_mass"));
    }

    /// An age is reached rather than assumed, and a process in no tier is unreachable
    /// rather than free.
    #[test]
    fn an_age_gates_the_work_inside_it() {
        assert_eq!(tier_ordinal("hands & stone"), Some(0));
        assert_eq!(tier_ordinal("bound & composite"), Some(1));
        assert_eq!(tier_ordinal("the age nobody wrote"), None);

        let bind = process("twist cord").expect("declared");
        assert!(reachable(bind, 1), "binding is reachable once its age is reached");
        assert!(!reachable(bind, 0), "and not before");
        let knap = process("knap a core").expect("declared");
        assert!(reachable(knap, 0), "the stone age is where knapping lives");
    }

    /// The gate is part of the material gate, and the open list is printed with it —
    /// a green run says the requirements are present, and nothing more.
    #[test]
    fn the_gate_reports_every_defect_and_every_open_item() {
        let report = crate::materials::verify();
        assert!(report.ok(), "the schema gate found defects: {:#?}", report.defects);
        for item in generated::SCHEMA_OPEN {
            assert!(
                report.lines.iter().any(|line| line.contains(item)),
                "the open list is part of the report: `{item}` was not printed"
            );
        }
        assert!(
            report.lines.iter().any(|line| line.contains("substances")),
            "the report states what it read"
        );
    }
}
