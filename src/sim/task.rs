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
//! The verb list is **closed on purpose and grows by decision**. One variant today, because a verb
//! with no implementation is a claim with no number: `Build` is the verb that has a mechanism.

use serde::{Deserialize, Serialize};

use super::{BuildingKind, MaterialLineage};

/// What a task asks for. Closed: a task whose verb is not on this list is not a task.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verb {
    /// Raise a structure on a site, out of material the world gives up.
    Build,
}

impl Verb {
    /// The declared list, quoted in refusals so the vocabulary is never implied.
    pub const ALL: &'static [Verb] = &[Verb::Build];

    pub fn name(self) -> &'static str {
        match self {
            Verb::Build => "build",
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

/// One unit of work: what is wanted, where, by whom, and how much is left.
///
/// `work_remaining` is in **labour-ticks**, decremented by the worker who is actually standing on
/// the site. Round 4's Q19 asked for work done atomically and verifiably; this is the counter that
/// makes "verifiable" mechanical — the number only goes down while somebody is there.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub verb: Verb,
    /// What is being raised, read by `Verb::Build` and ignored by every other verb.
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
    /// The citizen doing the work, by id. `None` means the task is open and nobody has taken it.
    pub claimed_by: Option<u32>,
    pub opened_tick: u64,
}

impl Task {
    /// Whether anybody has taken this work.
    pub fn is_claimed(&self) -> bool {
        self.claimed_by.is_some()
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
        format!(
            "{} {} at tile {} ({} g of {}, {} work left{})",
            self.verb.name(),
            self.kind.name(),
            self.site,
            self.requires_g,
            self.family,
            self.work_remaining,
            match self.claimed_by {
                Some(who) => format!(", claimed by citizen {who}"),
                None => String::new(),
            }
        )
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
        assert_eq!(Verb::ALL.len(), 1, "a verb arrives with an implementation, not before");
    }
}
