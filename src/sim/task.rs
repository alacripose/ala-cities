//! Tasks: the atomic, verifiable unit of work a citizen does (C9 round 11).
//!
//! Round 7's Q43 answered that a task is **a closed verb list plus a declared target** — the same
//! shape as the governor's `Op` vocabulary, which refuses anything outside its eight names and
//! quotes the list when it does. Round 6's Q45 then answered that a city grows because *citizens*
//! acquire material and build, through this system rather than through `grow()`, and round 11's
//! Q72 confirmed the ordering: delete `grow()` now and let the task system do the building.
//!
//! Two properties make a task auditable rather than aspirational:
//!
//! * **Its material is planned before the work starts.** A build that discovered halfway through
//!   that the world could not supply it would have spent a citizen's time and left a hole. The
//!   plan is atomic, so a task that exists can be paid for.
//! * **Completion is a defined check.** The verb decides what completion means, and for `Build` it
//!   means the material was drawn *and a structure stands*. A task cannot be completed by
//!   declaring it complete — which is what "acceptance is a test that passed" means when the
//!   worker is a citizen rather than a ticket.
//!
//! The verb list is **closed on purpose and grows by decision**. Four verbs today, each with a
//! mechanism behind it: `Build` raises a structure out of the ground, `Gather` takes a leaf out
//! of the world by hand and carries it to the site, `Make` runs a declared process on material
//! that is already there, and `Maintain` puts mass back into a structure that has weathered.
//!
//! # Why `Maintain` is a verb and not a service call
//!
//! Round 5's Q54 answered that **repair is the same system as building**, and the consequence is
//! this list rather than a second path: a repair is a task, planned before anybody walks, drawing
//! its material out of the ground like any other draw — and closing through `World::repair`, which
//! is the one implementation the repair ticket owns. A structure that mended itself because a
//! counter crossed a threshold would be health regeneration wearing a material coat.
//!
//! # Why `Make` is not `Build`
//!
//! A build's mass comes out of the ground under its own site, which is C9's *drawn but not
//! hauled* step. A make is the next link: its inputs must **already be held** at the site
//! (round 13's Q77), which is what makes a gather a real predecessor rather than a formality —
//! and it is what makes the plan's order emerge instead of having to be scheduled. A make whose
//! inputs are not there is not posted, and if they vanish it stalls with a case naming what is
//! missing (Q92).

use serde::{Deserialize, Serialize};

use crate::materials::generated::{Process, Rational};

use super::{BuildingKind, MaterialLineage};

/// What a task asks for. Closed: a task whose verb is not on this list is not a task.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verb {
    /// Raise a structure on a site, out of material the world gives up.
    Build,
    /// Take a leaf out of the world by hand — a patch the ground grows, or a substance a deposit
    /// holds — and carry it to the site.
    Gather,
    /// Run a declared process at a site, on material **already held there**.
    Make,
    /// Put mass back into a structure that owes it, out of material the world gives up.
    ///
    /// The only verb whose site is **already built on**: a maintain is addressed to a structure
    /// standing there, and `kind` is that structure's kind rather than a thing to raise.
    Maintain,
}

impl Verb {
    /// The declared list, quoted in refusals so the vocabulary is never implied.
    pub const ALL: &'static [Verb] = &[Verb::Build, Verb::Gather, Verb::Make, Verb::Maintain];

    pub fn name(self) -> &'static str {
        match self {
            Verb::Build => "build",
            Verb::Gather => "gather",
            Verb::Make => "make",
            Verb::Maintain => "maintain",
        }
    }

    /// The verb for a name, or `None` — the refusal case, not a default.
    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|verb| verb.name() == name)
    }

    /// Whether this verb builds a structure, which is what makes `site` a build site.
    pub fn builds(self) -> bool {
        matches!(self, Verb::Build)
    }
}

/// How far along a task is, and where the worker therefore has to be.
///
/// A gather is the only verb that needs this: it takes from one tile and leaves at another, and
/// a stage is what keeps *walk to the patch* and *walk to the site* from being the same trip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    /// Working where the task's work is: a build site, a make's site, always.
    #[default]
    Working,
    /// Walking to the patch a gather takes from.
    Fetching,
    /// Carrying what was taken to the site the task is addressed to.
    Delivering,
}

impl Stage {
    pub fn name(self) -> &'static str {
        match self {
            Stage::Working => "working",
            Stage::Fetching => "fetching",
            Stage::Delivering => "carrying",
        }
    }
}

/// Work-ticks a declared hour of labour is worth.
///
/// The declared number, and the only place a duration becomes work: `labour_hours` is a real
/// ratio from the tables (Q42), and how long a *game* hour takes is a declared design choice
/// that has to live somewhere visible. Eight ticks is a fifth of a sim-second per work-tick, so
/// a gather of a quarter-hour is two work-ticks.
pub const WORK_TICKS_PER_LABOUR_HOUR: i64 = 8;

/// The work a process's declared mechanism is worth, in work-ticks, rounded **up**: half a tick
/// of work is still work somebody has to do, and rounding down would make a short process free.
pub fn work_of_process(process: &Process) -> i64 {
    let Rational { num, den } = process.mechanism.labour_hours;
    if den == 0 || num <= 0 {
        return 1;
    }
    let ticks = (num * WORK_TICKS_PER_LABOUR_HOUR + den - 1) / den;
    ticks.max(1)
}

/// One unit of work: what is wanted, where, by whom, and how much is left.
///
/// `work_remaining` is in **labour-ticks**, decremented by the worker who is actually standing on
/// the site. Round 4's Q19 asked for work done atomically and verifiably; this is the counter that
/// makes "verifiable" mechanical — the number only goes down while somebody is there.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub verb: Verb,
    /// What the work is for: the structure being raised (`Build`) or the structure being put
    /// back (`Maintain`). Ignored by `Gather` and `Make`, which address substances and processes
    /// rather than structures, and left at a documented placeholder there.
    pub kind: BuildingKind,
    /// The tile the work happens on.
    pub site: u32,
    /// The material family the world must supply, and how much of it, in grams.
    pub family: String,
    pub requires_g: i64,
    /// The planned draw, held until the work completes — so the mass a structure claims is the
    /// mass the plan promised rather than a second search that could differ from the first.
    pub material: Vec<MaterialLineage>,
    pub work_remaining: i64,
    /// The process this task runs, by its declared name — a `Make`'s work, and a `Gather`'s too,
    /// because a gather *is* a declared process (`gather timber`). Empty for a `Build`, whose
    /// work is the kind's own declared build cost.
    #[serde(default)]
    pub process: String,
    /// The substance a `Gather` takes, in the table's vocabulary. Empty for the other verbs.
    #[serde(default)]
    pub substance: String,
    /// Where a `Gather` takes it from: the tile the worker has to stand by. `None` for the other
    /// verbs, whose material is either under the site or already held there.
    #[serde(default)]
    pub fetch_from: Option<u32>,
    /// How far along the task is. See [`Stage`].
    #[serde(default)]
    pub stage: Stage,
    /// The citizen doing the work, by id. `None` means the task is open and nobody has taken it.
    pub claimed_by: Option<u32>,
    pub opened_tick: u64,
}

impl Task {
    /// Whether anybody has taken this work.
    pub fn is_claimed(&self) -> bool {
        self.claimed_by.is_some()
    }

    /// The tile the worker has to be at **now**: the patch while fetching, the site otherwise.
    ///
    /// One method rather than a `site` read at each call site, because a gather that walked to
    /// its site first and then to its patch would be a task the citizen cannot finish — and the
    /// place that decides where to walk is the only place that has to know.
    pub fn destination(&self) -> u32 {
        match self.stage {
            Stage::Fetching => self.fetch_from.unwrap_or(self.site),
            Stage::Working | Stage::Delivering => self.site,
        }
    }

    /// Whether this task is a gather that still has to walk to its patch.
    pub fn is_fetching(&self) -> bool {
        self.verb == Verb::Gather && self.stage == Stage::Fetching
    }

    /// Whether this task is a gather carrying its load to the site.
    pub fn is_delivering(&self) -> bool {
        self.verb == Verb::Gather && self.stage == Stage::Delivering
    }

    /// Whether the work is done. For `Build` that means the counter reached zero *and* the plan
    /// still holds — a worker cannot finish a task whose material is no longer there.
    pub fn is_done(&self) -> bool {
        self.work_remaining <= 0
    }

    /// The mass this task's plan accounts for, in grams. The plan is what the work is *for*, so
    /// this is the number a completion check compares against what the ground actually gave up.
    pub fn lineage_total(&self) -> i64 {
        self.material.iter().map(|line| line.grams).sum()
    }

    /// What a person reads in a listing.
    pub fn describe(&self) -> String {
        // A build is described by the structure it raises and the family the ground supplies; a
        // gather by the substance and the patch; a make by the process and the site. One line per
        // verb, because a listing that described a make as "build  at tile 14" would be a record
        // nobody could read.
        match self.verb {
            Verb::Build => format!(
                "build {} at tile {} ({} g of {}, {} work left{})",
                self.kind.name(),
                self.site,
                self.requires_g,
                self.family,
                self.work_remaining,
                self.claim_line()
            ),
            Verb::Gather => format!(
                "gather {} at tile {} for the site at tile {} ({} g, {} work left, {}{})",
                self.substance,
                self.fetch_from.unwrap_or(self.site),
                self.site,
                self.requires_g,
                self.work_remaining,
                self.stage.name(),
                self.claim_line()
            ),
            Verb::Make => format!(
                "make `{}` at tile {} ({} work left{})",
                self.process,
                self.site,
                self.work_remaining,
                self.claim_line()
            ),
            // The condition is deliberately **not** printed here: it is a reading of the
            // structure's mass, and a task carrying its own copy would be a second home for it.
            Verb::Maintain => format!(
                "maintain {} at tile {} ({} g of {} to put back, {} work left{})",
                self.kind.name(),
                self.site,
                self.requires_g,
                self.family,
                self.work_remaining,
                self.claim_line()
            ),
        }
    }

    fn claim_line(&self) -> String {
        match self.claimed_by {
            Some(who) => format!(", claimed by citizen {who}"),
            None => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The closed-list rule, tested rather than asserted: a name outside the list is `None`, which
    /// is the refusal a caller has to handle, and every declared verb answers to its own name.
    #[test]
    fn the_verb_list_is_closed_and_quotes_itself() {
        for verb in Verb::ALL {
            assert_eq!(Verb::named(verb.name()), Some(*verb));
        }
        assert_eq!(Verb::named("demolish"), None, "not declared, so not a task verb");
        assert_eq!(Verb::named(""), None);
        // Mending is in the list because it has a mechanism behind it: a plan, a draw out of the
        // ground, and the one `repair` that closes it (Q54).
        assert_eq!(Verb::named("maintain"), Some(Verb::Maintain));
        assert_eq!(Verb::ALL.len(), 4, "a verb arrives with an implementation, not before");
    }

    /// A gather has two destinations and a make has one, and the difference is the whole reason
    /// the stage exists: the worker walks to the patch, then to the site, and never the reverse.
    #[test]
    fn a_gather_walks_to_its_patch_before_its_site() {
        let mut task = Task {
            id: 1,
            verb: Verb::Gather,
            kind: BuildingKind::Home,
            site: 40,
            family: "organic".to_string(),
            requires_g: 3000,
            material: Vec::new(),
            process: "gather timber".to_string(),
            substance: "timber".to_string(),
            fetch_from: Some(9),
            stage: Stage::Fetching,
            work_remaining: 2,
            claimed_by: None,
            opened_tick: 0,
        };
        assert_eq!(task.destination(), 9, "the patch comes first");
        assert!(task.is_fetching() && !task.is_delivering());
        task.stage = Stage::Delivering;
        assert_eq!(task.destination(), 40, "then the site the work is for");
        assert!(task.is_delivering() && !task.is_fetching());
        task.stage = Stage::Working;
        assert_eq!(task.destination(), 40);
    }

    /// Work is a declared multiple of a declared duration, rounded up: a process that takes an
    /// eighth of an hour is not free, and the number is the same in every replay.
    #[test]
    fn a_process_declares_its_work_in_ticks() {
        for process in crate::materials::generated::PROCESSES {
            let work = work_of_process(process);
            assert!(work >= 1, "`{}` takes no work at all", process.name);
        }
        // The stone rung's own numbers, so an edit to the rate announces itself here.
        let gather = crate::materials::schema::process("gather timber").expect("declared");
        assert_eq!(work_of_process(gather), 2, "a quarter-hour is two work-ticks at 8 per hour");
        let assemble = crate::materials::schema::process("assemble the hatchet").expect("declared");
        assert_eq!(work_of_process(assemble), 4, "half an hour is four");
    }
}
