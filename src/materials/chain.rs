//! Reducing a wanted good to what the world must give up for it (C9 phase 3).
//!
//! The process table says what each step costs. This module answers the question a
//! city actually asks: **to get one hatchet, what has to come out of the ground, and
//! in what order does the work happen?** It is the table applied rather than
//! described, and it is deliberately pure — no world, no clock, no citizen — so the
//! answer is a function of the declaration alone and can be checked against
//! `SCHEMA.md`'s worked chain line by line.
//!
//! # The rules it follows
//!
//! * **A leaf is the world.** Reduction stops at a substance the world already holds:
//!   either a process with no inputs (*a gather* — `gather timber`) or a substance
//!   whose declared source is `mined` or `salvaged` (*an extraction* — the deposit
//!   take that `sim` already performs). A substance with no route to either is
//!   **refused by name**, because a plan that cannot reach the ground is a plan that
//!   would have to invent mass.
//! * **Integer arithmetic, and rounding is declared rather than hidden.** A process
//!   runs a whole number of times, so making 2900 g of hatchet where one run yields
//!   2900 g is one run — and asking for 3000 g is *two*, with the 2800 g left over
//!   **reported as surplus**. Rounding up silently would be mass appearing; not
//!   rounding would be a plan that cannot be carried out.
//! * **Dependencies come first.** The steps are returned in the order the work has to
//!   happen in, so a caller can post them one at a time and watch the chain build
//!   itself (Q19's atomic, verifiable work at every step).
//! * **A cycle is refused, not survived.** A table where A is made from B and B from A
//!   is a table that makes nothing, and the refusal names the cycle rather than
//!   recursing until the stack says so.
//!
//! # What it does not decide
//!
//! **Which recipe, when a substance has more than one.** Today the first declared
//! process that produces a substance wins, and that is a policy gap rather than a
//! decision: the city will eventually choose between recipes by what it has, what it
//! costs and what it knows (phase 6's progressions). The rule is named here so the
//! first second producer arrives as a question rather than silently changing which
//! branch a plan takes.

use std::collections::BTreeMap;

use crate::materials::generated::{Process, Rational};
use crate::materials::schema::{quantity_grams, substance, Unit};

/// How a plan's leaf reaches the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Route {
    /// A process with no inputs: a person takes it from a surface the world grows.
    Gather,
    /// A substance the ground holds: the deposit take `sim` performs.
    Extract,
}

impl Route {
    pub fn name(self) -> &'static str {
        match self {
            Route::Gather => "gather",
            Route::Extract => "extract",
        }
    }
}

/// One leaf of a plan: a substance, how much of it, and how it comes out of the world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Leaf {
    pub substance: &'static str,
    pub grams: i64,
    pub route: Route,
    /// The gather process that takes it, when there is one.
    pub process: Option<&'static str>,
}

/// One step of work, in the order it has to happen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub process: &'static str,
    /// How many times the process runs. A whole number, always.
    pub runs: i64,
    /// Grams the step's inputs come to, in total.
    pub input_g: i64,
    /// Grams the step's outputs come to, in total. Equal to `input_g` by the table's
    /// own balance — and checked here rather than assumed, because this is the module
    /// that would turn an unbalanced row into a city that creates matter.
    pub output_g: i64,
}

/// A plan: what the world gives up, what the work is, and what is left over.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    /// What was asked for, in grams, by substance.
    pub wanted: Vec<(&'static str, i64)>,
    /// The world's own contributions, gathered or extracted.
    pub leaves: Vec<Leaf>,
    /// The work, dependencies first.
    pub steps: Vec<Step>,
    /// What a step produces beyond the target and which therefore stays in stock —
    /// mass that is real, declared, and not silently discarded.
    pub surplus: Vec<(&'static str, i64)>,
    /// Everything a step produces that is not its target: offcuts, flakes, dust. Named
    /// rather than judged — whether anything uses it is the substance table's business.
    pub by_products: Vec<(&'static str, i64)>,
}

impl Plan {
    /// Every gram the world has to give up, in total.
    pub fn drawn_g(&self) -> i64 {
        self.leaves.iter().map(|leaf| leaf.grams).sum()
    }

    /// Whether the plan needs no work at all: the world already holds the substance.
    pub fn is_bare_leaf(&self) -> bool {
        self.steps.is_empty()
    }

    /// The plan as a person reads it.
    pub fn describe(&self) -> String {
        let mut lines = vec![format!(
            "plan: {} from {} g of the world, {} step(s)",
            self.wanted
                .iter()
                .map(|(name, grams)| format!("{grams} g {name}"))
                .collect::<Vec<_>>()
                .join(" + "),
            self.drawn_g(),
            self.steps.len()
        )];
        for leaf in &self.leaves {
            lines.push(format!(
                "  {} {} g of {}",
                leaf.route.name(),
                leaf.grams,
                leaf.substance
            ));
        }
        for step in &self.steps {
            lines.push(format!(
                "  run {} ×{} ({} g in, {} g out)",
                step.process, step.runs, step.input_g, step.output_g
            ));
        }
        lines.join("\n")
    }
}

/// Why a good cannot be planned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The substance is not declared at all.
    Undeclared(String),
    /// Declared, and nothing produces it and the world does not hold it.
    NoRoute {
        substance: String,
        source: &'static str,
    },
    /// The table loops: this substance is reachable from itself.
    Cycle(Vec<String>),
    /// A row's numbers do not land on whole grams.
    NotExact(String),
}

impl Refusal {
    pub fn describe(&self) -> String {
        match self {
            Refusal::Undeclared(name) => {
                format!("`{name}` is not a declared substance")
            }
            Refusal::NoRoute { substance, source } => format!(
                "`{substance}` cannot be reached: it is `{source}`, no process produces it, and \
                 the ground does not hold it — a plan that cannot reach the world would have to \
                 invent mass"
            ),
            Refusal::Cycle(path) => format!(
                "the process table loops: {} — a substance reachable from itself makes nothing",
                path.join(" → ")
            ),
            Refusal::NotExact(name) => format!(
                "`{name}` has a quantity that does not land on a whole gram, so the plan cannot \
                 be carried out in whole units"
            ),
        }
    }
}

/// A substance's declared source, or `None` when it is not in the table.
fn source_of(name: &str) -> Option<&'static str> {
    substance(name).map(|entry| entry.source)
}

/// The first declared process that produces `substance`, if any.
///
/// **The first, and that is a named policy gap rather than a decision** — see this
/// module's header. When a second producer exists, this is the line that has to learn
/// to choose.
fn producer<'a>(processes: &'a [Process], substance: &str) -> Option<&'a Process> {
    processes
        .iter()
        .find(|process| process.outputs.iter().any(|(name, _)| *name == substance))
}

/// The grams a process produces of `substance` in one run.
fn output_grams(process: &Process, substance: &str) -> Option<i64> {
    process
        .outputs
        .iter()
        .find(|(name, _)| *name == substance)
        .and_then(|(_, amount)| quantity_grams(substance, *amount))
}

/// How many whole runs it takes to produce at least `grams`, rounded **up**.
///
/// Written out rather than using `div_ceil`, which the pinned toolchain does not have:
/// this crate compiles against the toolchain in `rust-toolchain.toml`, and a plan's
/// arithmetic is not worth a version bump.
fn runs_for(grams: i64, per_run: i64) -> i64 {
    (grams + per_run - 1) / per_run
}

/// Grams in and out for one run of a process, or `None` when a row cannot reach grams.
fn run_totals(process: &Process) -> Option<(i64, i64)> {
    let total = |side: &[(&'static str, Rational)]| {
        side.iter()
            .try_fold(0i64, |sum, (name, amount)| Some(sum + quantity_grams(name, *amount)?))
    };
    Some((total(process.inputs)?, total(process.outputs)?))
}

/// Reduce a wanted amount of a substance to the world's own material and the work
/// between them, using the table the game ships.
pub fn plan(substance: &str, grams: i64) -> Result<Plan, Refusal> {
    plan_with(crate::materials::generated::PROCESSES, substance, grams)
}

/// The same reduction against a given table — which is what makes the cycle check
/// testable, because a table that loops cannot be the one that ships.
pub fn plan_with(processes: &[Process], substance: &str, grams: i64) -> Result<Plan, Refusal> {
    let mut plan = Plan::default();
    let mut stack: Vec<String> = Vec::new();
    reduce(processes, substance, grams, &mut plan, &mut stack)?;
    Ok(plan)
}

/// Add `grams` to a named line of the plan, merging with anything already there.
fn add(into: &mut Vec<(&'static str, i64)>, name: &'static str, grams: i64) {
    if grams == 0 {
        return;
    }
    if let Some(line) = into.iter_mut().find(|(existing, _)| *existing == name) {
        line.1 += grams;
        return;
    }
    into.push((name, grams));
}

fn reduce(
    processes: &[Process],
    substance: &str,
    grams: i64,
    plan: &mut Plan,
    stack: &mut Vec<String>,
) -> Result<(), Refusal> {
    if grams <= 0 {
        return Ok(());
    }
    let Some(entry) = crate::materials::schema::substance(substance) else {
        return Err(Refusal::Undeclared(substance.to_string()));
    };
    let declared = entry.name;

    // A leaf: the world already holds it, so there is no work between the want and the
    // ground. Which leaf it is decides who does the taking — a gather is a process, an
    // extraction is the deposit take.
    match producer(processes, declared) {
        Some(process) if !process.inputs.is_empty() => {
            // A transformation: recurse into its inputs, then record the step.
            if stack.iter().any(|open| open == declared) {
                stack.push(declared.to_string());
                return Err(Refusal::Cycle(stack.clone()));
            }
            stack.push(declared.to_string());

            let per_run = output_grams(process, declared).ok_or_else(|| {
                Refusal::NotExact(format!("{} produces {declared}", process.name))
            })?;
            if per_run <= 0 {
                return Err(Refusal::NotExact(format!(
                    "{} produces no {declared}",
                    process.name
                )));
            }
            // Whole runs, rounded **up**, with the remainder named as surplus.
            let runs = runs_for(grams, per_run);
            let produced = runs * per_run;
            add(&mut plan.surplus, declared, produced - grams);

            for (name, amount) in process.inputs {
                let needed = quantity_grams(name, *amount)
                    .ok_or_else(|| Refusal::NotExact((*name).to_string()))?
                    .checked_mul(runs)
                    .ok_or_else(|| Refusal::NotExact((*name).to_string()))?;
                reduce(processes, name, needed, plan, stack)?;
            }
            for (name, amount) in process.outputs {
                if *name == declared {
                    continue;
                }
                let made = quantity_grams(name, *amount)
                    .ok_or_else(|| Refusal::NotExact((*name).to_string()))?
                    .checked_mul(runs)
                    .ok_or_else(|| Refusal::NotExact((*name).to_string()))?;
                add(&mut plan.by_products, name, made);
            }

            let (input_g, output_g) = run_totals(process)
                .ok_or_else(|| Refusal::NotExact(process.name.to_string()))?;
            plan.steps.push(Step {
                process: process.name,
                runs,
                input_g: input_g * runs,
                output_g: output_g * runs,
            });
            stack.pop();
            Ok(())
        }
        // A gather: a process with no inputs, which is the world handing over what it
        // already grows.
        Some(process) => {
            let per_run = output_grams(process, declared).ok_or_else(|| {
                Refusal::NotExact(format!("{} produces {declared}", process.name))
            })?;
            let runs = runs_for(grams, per_run);
            let produced = runs * per_run;
            add(&mut plan.surplus, declared, produced - grams);
            let (input_g, output_g) = run_totals(process)
                .ok_or_else(|| Refusal::NotExact(process.name.to_string()))?;
            debug_assert_eq!(input_g, 0, "a gather takes nothing in");
            plan.steps.push(Step {
                process: process.name,
                runs,
                input_g: 0,
                output_g: output_g * runs,
            });
            plan.leaves.push(Leaf {
                substance: declared,
                grams,
                route: Route::Gather,
                process: Some(process.name),
            });
            Ok(())
        }
        // No process produces it. Mined and salvaged substances are taken from the
        // ground; anything else has no route and is refused by name.
        None => match declared {
            _ if matches!(source_of(declared), Some("mined") | Some("salvaged")) => {
                plan.leaves.push(Leaf {
                    substance: declared,
                    grams,
                    route: Route::Extract,
                    process: None,
                });
                Ok(())
            }
            _ => Err(Refusal::NoRoute {
                substance: declared.to_string(),
                source: source_of(declared).unwrap_or("undeclared"),
            }),
        },
    }
}

/// The substances whose declared unit is a count, and the tools among them.
///
/// A tool is a counted good, which is what makes "one hatchet" a number the ledger can
/// carry — see [`Unit::Count`] and the declared unit mass it needs.
pub fn counted_goods() -> Vec<&'static str> {
    crate::materials::generated::SUBSTANCES
        .iter()
        .filter(|entry| entry.unit == Unit::Count.name())
        .map(|entry| entry.name)
        .collect()
}

/// Every substance a process can produce, in a stable order.
pub fn producible() -> BTreeMap<&'static str, &'static str> {
    crate::materials::generated::PROCESSES
        .iter()
        .flat_map(|process| {
            process
                .outputs
                .iter()
                .map(move |(name, _)| (*name, process.name))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::generated;

    /// `SCHEMA.md`'s worked chain, as the machine reads it: one hatchet comes out of
    /// three armfuls of the world, and every number matches the document.
    #[test]
    fn one_hatchet_reduces_to_the_three_gathers_the_schema_states() {
        let plan = plan("hatchet", 2900).expect("the stone rung reaches the ground");
        assert!(!plan.is_bare_leaf(), "a hatchet is made, not taken");

        let mut leaves: Vec<(&str, i64, Route)> = plan
            .leaves
            .iter()
            .map(|leaf| (leaf.substance, leaf.grams, leaf.route))
            .collect();
        leaves.sort_by_key(|(name, grams, _)| (*name, *grams));
        assert_eq!(
            leaves,
            vec![
                ("plant_fibre", 100, Route::Gather),
                ("stone", 1000, Route::Gather),
                ("timber", 3000, Route::Gather),
            ],
            "the chain's leaves are the schema's own three gathers"
        );
        assert_eq!(plan.drawn_g(), 4100, "4.1 kg of the world, exactly");

        // The work, in order: **dependencies first**, which is the property that lets a
        // caller post one step at a time. The order among steps that do not depend on
        // each other is not asserted, because nothing depends on it — the two gathers
        // may come in either order and the plan is the same plan.
        let process_names: Vec<&str> = plan.steps.iter().map(|step| step.process).collect();
        assert_eq!(
            process_names.last().copied(),
            Some("assemble the hatchet"),
            "the thing asked for is made last: {process_names:?}"
        );
        for (index, step) in plan.steps.iter().enumerate() {
            let process = crate::materials::schema::process(step.process).expect("declared");
            for (input, _) in process.inputs {
                // A substance this plan makes must be made **before** it is consumed.
                let maker = plan.steps.iter().position(|earlier| {
                    crate::materials::schema::process(earlier.process)
                        .is_some_and(|p| p.outputs.iter().any(|(name, _)| name == input))
                });
                match maker {
                    Some(maker) => assert!(
                        maker < index,
                        "`{input}` is consumed by `{}` at step {index} but made at step {maker}",
                        step.process
                    ),
                    // Not made by the plan, so it has to come out of the ground here.
                    None => assert!(
                        plan.leaves.iter().any(|leaf| {
                            leaf.substance == *input && leaf.route == Route::Extract
                        }),
                        "`{input}` is neither made by the plan nor taken from the world"
                    ),
                }
            }
        }
        assert!(
            plan.steps.iter().all(|step| step.runs == 1),
            "one of each: {:#?}",
            plan.steps
        );
        assert!(
            plan.surplus.is_empty(),
            "one run yields exactly one hatchet's worth: {:#?}",
            plan.surplus
        );

        // Every declared loss is in the plan rather than missing from it.
        let mut by_products: Vec<(&str, i64)> = plan.by_products.clone();
        by_products.sort();
        assert_eq!(
            by_products,
            vec![
                ("fibre_dust", 5),
                ("stone_flakes", 200),
                ("timber_offcuts", 600),
                ("trim_waste", 395),
            ],
            "a loss is an output, so a plan has to carry it"
        );
    }

    /// Rounding happens in whole runs, and what is left over is **declared**. Two
    /// hatchets' worth is two runs; one gram more is still two runs, and the surplus
    /// says how much.
    #[test]
    fn a_run_is_whole_and_the_remainder_is_named_as_surplus() {
        let two = plan("hatchet", 5800).expect("planned");
        assert!(
            two.steps
                .iter()
                .filter(|step| step.process == "assemble the hatchet")
                .all(|step| step.runs == 2),
            "5800 g is two hatchets: {:#?}",
            two.steps
        );
        assert!(two.surplus.is_empty(), "two runs yield exactly 5800 g");

        let over = plan("hatchet", 2901).expect("planned");
        assert_eq!(
            over.surplus,
            vec![("hatchet", 2899)],
            "one gram past a whole hatchet is a second run, and the rest stays as stock"
        );
        assert_eq!(over.drawn_g(), 4100 * 2, "and the world gives up twice what it did");
    }

    /// A substance the ground holds and nothing makes is an **extraction**, not a
    /// refusal: sand is mined, so a plan for it is a hole in the ground.
    #[test]
    fn a_mined_substance_is_taken_rather_than_made() {
        let plan = plan("clay", 5000).expect("clay is mined");
        assert_eq!(
            plan.leaves,
            vec![Leaf {
                substance: "clay",
                grams: 5000,
                route: Route::Extract,
                process: None
            }]
        );
        assert!(plan.steps.is_empty(), "nothing is made, so there is no work");
        assert_eq!(plan.drawn_g(), 5000);
    }

    /// A substance with no route to the world is refused by name, with its source
    /// quoted — a plan that cannot reach the ground would have to invent mass.
    #[test]
    fn a_substance_with_no_route_to_the_world_is_refused() {
        let refusal = plan("nobody declared this", 10).expect_err("undeclared");
        assert_eq!(refusal, Refusal::Undeclared("nobody declared this".to_string()));
        assert!(refusal.describe().contains("not a declared substance"));

        // `water` is declared `gathered` with no gather process and no deposit: it is
        // the honest example of a substance whose consumer does not exist yet.
        let refusal = plan("water", 1000).expect_err("no route");
        let described = refusal.describe();
        assert!(described.contains("cannot be reached"), "{described}");
        assert!(described.contains("gathered"), "{described}");
        assert!(described.contains("would have to invent mass"), "{described}");
    }

    /// A looping table is refused **by name**, and the refusal carries the path — which
    /// is the difference between a message somebody can act on and a stack overflow.
    /// Constructed rather than shipped, because a table that loops cannot be the one
    /// that ships.
    #[test]
    fn a_looping_table_is_refused_with_the_cycle_named() {
        let mechanism = generated::PROCESSES[0].mechanism;
        // Declared substances, so the loop is reached through the real table rather
        // than refused as an undeclared name: `cord` is made from `haft_blank` and
        // `haft_blank` from `cord`, which is a table that makes nothing.
        let looped: Vec<Process> = vec![
            Process {
                name: "cord from a haft",
                tier: "bound & composite",
                inputs: &[("haft_blank", Rational { num: 1000, den: 1 })],
                outputs: &[("cord", Rational { num: 1000, den: 1 })],
                mechanism,
                requires: &[],
                note: "",
            },
            Process {
                name: "a haft from cord",
                tier: "bound & composite",
                inputs: &[("cord", Rational { num: 1000, den: 1 })],
                outputs: &[("haft_blank", Rational { num: 1000, den: 1 })],
                mechanism,
                requires: &[],
                note: "",
            },
        ];
        let refusal = plan_with(&looped, "cord", 1000).expect_err("the loop is refused");
        match refusal {
            Refusal::Cycle(path) => {
                assert!(path.len() >= 2, "the path names where it closed: {path:?}");
                assert!(path[0] == path[path.len() - 1], "and closes on itself: {path:?}");
            }
            other => panic!("expected a cycle, got {other:?}"),
        }
        assert!(
            Refusal::Cycle(vec!["cord".into(), "cord".into()])
                .describe()
                .contains("loops"),
            "the refusal reads as a sentence"
        );
    }

    /// The table that ships is acyclic and every leaf it reaches is either a gather or
    /// an extraction — the property that makes "nothing from nothing" true of the
    /// *plans* as well as of the rows.
    #[test]
    fn every_plan_the_table_can_make_ends_in_the_world() {
        for (name, process) in producible() {
            let plan = plan(name, 1000)
                .unwrap_or_else(|refusal| panic!("`{name}` ({process}) cannot be planned: {}", refusal.describe()));
            assert!(
                !plan.leaves.is_empty(),
                "`{name}` was made without the world giving anything up"
            );
            for leaf in &plan.leaves {
                assert!(
                    substance(leaf.substance)
                        .is_some_and(|entry| entry.source != "made"),
                    "a leaf must be something the world holds: `{}`",
                    leaf.substance
                );
            }
            // A step's totals balance — except a **gather**, whose other side is the
            // world rather than a row, and which is therefore a leaf by definition.
            for step in &plan.steps {
                if step.input_g == 0 {
                    assert!(
                        plan.leaves.iter().any(|leaf| {
                            leaf.process == Some(step.process) && leaf.route == Route::Gather
                        }),
                        "`{}` takes nothing in and is not a declared gather",
                        step.process
                    );
                    continue;
                }
                assert_eq!(
                    step.input_g, step.output_g,
                    "step `{}` does not balance",
                    step.process
                );
            }
        }
    }

    /// The counted goods are the ones a person reads as objects rather than as
    /// quantities, and every one of them declares its unit mass.
    #[test]
    fn every_counted_good_declares_a_unit_mass() {
        let counted = counted_goods();
        assert_eq!(counted, vec!["hatchet"], "the stone rung's one tool");
        for name in counted {
            let entry = substance(name).expect("declared");
            assert!(
                entry.unit_mass.num > 0,
                "`{name}` is counted and weighs nothing, so it cannot enter the ledger"
            );
        }
    }
}
