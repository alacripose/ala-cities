//! The mass ledger (C9 phase 2).
//!
//! Round 1's whole premise is that **the world itself and its resources** are the only
//! source, and round 9's Q26 answered that conservation is true *by construction* — mass is a
//! field on every object — and **audited** by a ledger that `verify.exe` reads back as claims.
//! This is the audit half. Phase 1 made the ground able to say what it holds and what it has
//! given up; this makes the total checkable.
//!
//! # Why a ledger at all, when the mass is already on the objects
//!
//! Because two facts that must agree are worth more than one fact that is trusted. A
//! structure's mass, a stockpile's contents and a tile's extraction can each be right while
//! their *sum* is wrong — a transfer that credits one side and forgets the other is exactly
//! the bug this catches, and it is the bug that makes "nothing from nothing" quietly false
//! while every individual number looks fine.
//!
//! # Double entry, and why the total is always zero
//!
//! **Mass is never created, so every movement has two sides.** Extracting from a tile debits
//! `ground:iron_ore` — the ground's account goes *down*, because the world has taken from it —
//! and credits `stock:iron_ore`. Smelting moves it from `stock:iron_ore` to `stock:steel`.
//! Waste moves it from a stock to `waste:...`. Every one of those has two sides, so a world
//! where every movement went through [`Ledger::convert`] shows a **total of exactly zero** —
//! not a tolerance, not a percentage, zero.
//!
//! Which is what makes the check sharp: **a nonzero total means some movement had one side.**
//! That is the whole audit — and it is why [`Ledger::holdings`] is a *reading* of where the
//! mass is, while [`Ledger::findings`] reports only the imbalance, which is the defect.
//!
//! There is no float anywhere in this file, and that is the point rather than a style: a ledger
//! that balances to within a rounding error cannot tell conservation from a bug, and this
//! campaign exists because the difference was invisible before.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A structure's mass at level 1, in grams, by the kind's declared name.
///
/// **A recorded deviation, not a second home.** These numbers belong in
/// `tools/materials/declare.py`'s substance and structure tables, beside every other declared
/// number, and they live here only because those tables do not exist yet — the same note
/// `materials::geology` carries for its deposits, for the same reason. Until then they are
/// declared once, on one screen, and a reader can see all of them at a time.
///
/// The provenance of the first row is the record itself: round 3's Q31 arithmetic is built on
/// *"a house is ~100 t of material"*, which is what makes one worked tile of ore worth about a
/// house and a half. The rest are declared game numbers scaled from it by the same reasoning
/// that gave the kinds their [NS]-free ratios, and they are marked as declared rather than
/// sourced: no source states what a game factory weighs.
pub const STRUCTURE_MASS_G: &[(&str, i64)] = &[
    ("home", 100_000_000),
    ("shop", 150_000_000),
    ("factory", 400_000_000),
    ("power plant", 1_200_000_000),
];

/// The mass of a structure of `kind` at `level`, in grams, or `None` for a kind this table
/// does not declare.
///
/// Level scales mass **linearly and exactly** — a level-3 home is three level-1 homes' worth of
/// material, in integers, with no growth curve to round. A curve would be a float, and a float
/// is where this ledger stops balancing without saying so.
pub fn structure_mass_g(kind: &str, level: u8) -> Option<i64> {
    let (_, base) = STRUCTURE_MASS_G.iter().find(|(name, _)| *name == kind)?;
    Some(base * level.max(1) as i64)
}

/// An account holding mass the world has taken **out of** the ground. Negative as it depletes:
/// the ground is the side that goes down, which is what makes extraction auditable rather than
/// a number the city asserts about itself.
pub fn ground_account(substance: &str) -> String {
    format!("ground:{substance}")
}

/// An account holding mass standing **in** the city as a structure.
pub fn structure_account(family: &str) -> String {
    format!("structure:{family}")
}

/// An account holding mass that is still in the world but no longer a working structure — a
/// retired building's remains. Retirement preserves the material as well as the history, so a
/// demolition moves mass here instead of destroying it.
pub fn ruin_account(family: &str) -> String {
    format!("ruin:{family}")
}

/// An account holding loose mass **at a site**: what stands on a tile waiting to be worked or
/// carried away (C9 round 13, Q77).
///
/// The substance is part of the key because a site holds more than one thing — a works holds
/// timber, stone and fibre at once — and *"where is the iron"* has to be answerable (Q82).
/// Q77 named the account `site:<tile>`; the substance is the reading of it, exactly as
/// `ground:iron_ore` qualifies the ground.
pub fn site_account(tile: u32, substance: &str) -> String {
    format!("site:{tile}:{substance}")
}

/// An account holding mass **in a carrier**: a person's arms, or a vehicle's bed (Q77, Q82).
///
/// The carrier is named rather than counted, so *"the iron is in a wagon, on the road, south of
/// the kiln"* is a sentence the ledger can support — and an abandoned carrier is a holding whose
/// owner is missing rather than a number that went nowhere.
pub fn carried_account(carrier: u32, substance: &str) -> String {
    format!("carried:carrier-{carrier}:{substance}")
}

/// The prefix every site holding shares, so a reader can ask a tile what it is holding without
/// knowing the substances in advance.
pub fn site_prefix(tile: u32) -> String {
    format!("site:{tile}:")
}

/// The prefix every carrier's holding shares.
pub fn carried_prefix(carrier: u32) -> String {
    format!("carried:carrier-{carrier}:")
}

/// The world's mass audit: where the mass is, and the one reconciliation a city can fail.
///
/// # Why this is not a zero total
///
/// [`Ledger`] balances at zero when every movement it recorded had two sides, and that is the
/// right check for a *transfer*. A whole world is not a transfer: taking iron out of the ground
/// debits the ground and credits nothing, because what it credits is **the city's own standing
/// material** — a different account, derived from a different fact, that only balances once the
/// city has actually built something out of it. So the world's invariant is a reconciliation:
///
/// > everything the world holds equals everything it has taken out of its own ground.
///
/// Which leaves exactly one way to fail, and it is the failure this campaign exists to name:
/// [`MassAudit::loose_g`] going **negative** means the city stands on material nobody dug up.
/// That is not a warning about a number; that is `grow()` — material from nothing — printed as a
/// quantity.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MassAudit {
    /// Where the mass is: the ground's depletion as a negative, the standing and ruined material
    /// as positives. A reading, not a verdict.
    pub ledger: Ledger,
    /// Total mass debited from the ground, in grams, as a positive number.
    pub extracted_g: i64,
    /// Total mass standing in the city — structures, and the remains of retired ones.
    pub standing_g: i64,
    /// Total mass **held loose in the city's own hands**: at sites and in carriers (Q77).
    ///
    /// Separate from `standing_g` on purpose. A structure is material that has been *spent* on
    /// something that stands; a holding is material that has been taken and not yet spent, and
    /// the difference is exactly what a builder's yard is. Both are the city's, both came out of
    /// the ground, and both therefore count against what the ground gave up.
    pub held_g: i64,
    /// Structures that could not enter the audit at all, because their mass or their material
    /// family could not be derived.
    ///
    /// Reported rather than skipped in silence, for the reason the v1 → v2 migration reports a
    /// structure with no material claim instead of assuming one: a building that weighs nothing
    /// enters the ledger as nothing and **hides itself**, and an audit that can quietly lose a
    /// building is an audit that will.
    pub defects: Vec<String>,
}

impl MassAudit {
    /// Material the world has taken out of its ground and **not** put anywhere, in grams —
    /// neither standing as a structure nor held at a site. Real and legitimate: it is material
    /// spent as waste, or simply gone from the world's books the way spent mass is.
    ///
    /// Negative is the defect, and its size is the amount of material that was never dug up.
    pub fn loose_g(&self) -> i64 {
        self.extracted_g - self.standing_g - self.held_g
    }

    /// Whether every gram the city is holding came out of its own ground — the same question as
    /// [`Self::conserves`], asked of the holdings rather than of the structures.
    pub fn holdings_are_paid_for(&self) -> bool {
        self.held_g <= self.extracted_g - self.standing_g
    }

    /// Whether every gram standing in this world came out of its own ground.
    pub fn conserves(&self) -> bool {
        self.loose_g() >= 0
    }

    /// Where the mass is, in a stable order — the ground's debits and the city's credits.
    pub fn holdings(&self) -> Vec<(&str, i64)> {
        self.ledger.holdings()
    }

    /// What the ledger says about one account, in grams.
    pub fn of(&self, account: &str) -> i64 {
        self.ledger.of(account)
    }

    /// The audit as findings a person can read, in the shape `verify.exe` reports in.
    ///
    /// The mass that was never dug up is named **with its amount**, because a finding a reader
    /// cannot size is a finding nobody can act on.
    ///
    /// One finding, and deliberately not two: [`Ledger::findings`] is *not* appended here. Its
    /// zero-total check is a transfer's invariant, and a world's standing mass has its other side
    /// in the ground's depletion rather than in the ledger — the reconciliation above already
    /// states it. Reporting both would print one defect twice, and a report that names the same
    /// thing in two voices is how a reader learns to skim past it.
    pub fn findings(&self) -> Vec<String> {
        let mut findings = self.defects.clone();
        if self.conserves() {
            return findings;
        }
        findings.push(format!(
            "the city stands on {} g of material the ground never gave up: {} g has been taken \
             out, {} g is standing and {} g is held, so something here was not built from \
             anything",
            -self.loose_g(),
            self.extracted_g,
            self.standing_g,
            self.held_g
        ));
        findings
    }
}

/// Where the world's mass is, in exact grams, by account.
///
/// `BTreeMap` rather than `HashMap` so a listing is stable: a report that reorders itself
/// between runs is a report nobody can diff, and diffing is how this repo tracks everything
/// else. Account names are **qualified** — `ground:iron_ore` versus `stock:iron_ore` — because
/// the two sides of a transfer must not be the same key, or the movement would cancel itself
/// and the ledger would be checking nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ledger {
    entries: BTreeMap<String, i64>,
}

impl Ledger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record **one side** of a movement of `grams` on `account`. Positive credits, negative
    /// debits.
    ///
    /// This is the low-level half of [`Self::convert`] and it is deliberately one-sided, so
    /// that calling it without its partner *is* the bug the audit catches. Prefer `convert`.
    ///
    /// Going negative is allowed on purpose: the ledger is an audit, not a guard, and a debit
    /// with no source behind it is precisely the finding it exists to surface — "more iron was
    /// spent than was ever extracted" is a sentence a person needs to read, not an error to
    /// swallow at the point of recording.
    pub fn record(&mut self, substance: &str, grams: i64) {
        *self.entries.entry(substance.to_string()).or_default() += grams;
    }

    /// A transfer with both sides in one call: `grams_in` leaves `from` and `grams_out`
    /// arrives at `to`.
    ///
    /// This is how every movement in the world should be recorded — extraction, a process, a
    /// repair, a demolition — because it is the shape that cannot forget its other half.
    ///
    /// Returns the defect when the two sides disagree, because a transfer that does not
    /// conserve is a process the schema's gate should have refused — so reaching here with an
    /// imbalance means a defect got past it, and saying so at the call site is the only place
    /// that is still cheap to notice.
    pub fn convert(
        &mut self,
        from: &str,
        grams_in: i64,
        to: &str,
        grams_out: i64,
    ) -> Option<String> {
        if grams_in != grams_out {
            return Some(format!(
                "`{from}` → `{to}` does not conserve: {grams_in} g in, {grams_out} g out, \
                 a difference of {} g",
                grams_in - grams_out
            ));
        }
        self.record(from, -grams_in);
        self.record(to, grams_out);
        None
    }

    /// What the ledger says about one account, in grams.
    pub fn of(&self, account: &str) -> i64 {
        self.entries.get(account).copied().unwrap_or(0)
    }

    /// The whole world's balance. **Zero means every movement had two sides.**
    pub fn total_g(&self) -> i64 {
        self.entries.values().sum()
    }

    /// Whether the world has conserved what it has. The one predicate a verifier needs.
    pub fn balances(&self) -> bool {
        self.total_g() == 0
    }

    /// Everything the ledger knows, account and grams, in a stable order.
    pub fn entries(&self) -> impl Iterator<Item = (&str, i64)> {
        self.entries.iter().map(|(name, grams)| (name.as_str(), *grams))
    }

    /// Where the mass currently is: every account holding a nonzero amount.
    ///
    /// A *reading*, not a defect. A healthy world shows the ground's depletion as a negative
    /// and the stocks and standing structures as positives, and the two sides of the same mass
    /// are both visible — which is exactly what makes the total auditable.
    pub fn holdings(&self) -> Vec<(&str, i64)> {
        self.entries().filter(|(_, grams)| *grams != 0).collect()
    }

    /// The ledger as findings a person can read — the shape `verify.exe` reports in.
    ///
    /// Only the imbalance is a finding. "There is iron in a stockpile" is not news.
    pub fn findings(&self) -> Vec<String> {
        let mut findings = Vec::new();
        if !self.balances() {
            findings.push(format!(
                "the world's mass does not balance: {} g unaccounted for across {} account(s) \
                 — some movement had only one side, which is the one thing that cannot happen",
                self.total_g(),
                self.entries.len()
            ));
        }
        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A structure's mass is declared once and scales exactly, with no kind left undeclared —
    /// a building whose mass nobody declared would enter the ledger at zero and hide itself.
    #[test]
    fn every_structure_kind_declares_a_mass() {
        for kind in [
            crate::sim::BuildingKind::Home,
            crate::sim::BuildingKind::Shop,
            crate::sim::BuildingKind::Factory,
            crate::sim::BuildingKind::PowerPlant,
        ] {
            let mass = structure_mass_g(kind.name(), 1)
                .unwrap_or_else(|| panic!("{} has no declared mass", kind.name()));
            assert!(mass > 0, "{} weighs nothing", kind.name());
            assert_eq!(
                structure_mass_g(kind.name(), 3),
                Some(mass * 3),
                "level scales mass exactly, in integers"
            );
        }
        assert_eq!(structure_mass_g("a kind nobody declared", 1), None);
    }

    /// A structure's mass is derived, never stored, so the two facts cannot drift — and the
    /// level floor is what keeps a level-0 building from entering the ledger as nothing at all.
    #[test]
    fn level_zero_still_weighs_what_a_building_weighs() {
        assert_eq!(structure_mass_g("home", 0), structure_mass_g("home", 1));
    }

    /// Two tables declared separately — geology's deposit masses and this file's structure
    /// masses — have to agree about **what a tile is worth in buildings**, and until this test
    /// existed nothing made them.
    ///
    /// It pins what the tables support and **names what they do not**: the record's Q31 figure
    /// is *"one worked tile of ore is worth roughly a house and a half"*, and that is the
    /// **recovered metal**, after a ~45 % recovery the ore's own grade supplies. Neither the
    /// recovery ratio nor the ore grade is sourced yet — both are `[NS]` in
    /// `tools/materials/SOURCES.md` — so asserting the figure here would be inventing the very
    /// number that table exists to mark as missing. What is derivable is asserted; what is not
    /// is named.
    #[test]
    fn one_worked_tile_of_ore_is_worth_several_houses_of_rock() {
        let iron = crate::materials::geology::DEPOSIT_KINDS
            .iter()
            .position(|kind| kind.substance == "iron_ore")
            .expect("iron ore is a declared deposit kind");
        let tile = crate::materials::geology::full_tile_mass_g(iron);
        let home = structure_mass_g("home", 1).expect("declared");

        assert_eq!(tile, 345_600_000, "128 m³ of ore at 2.7 t/m³, exact");
        assert_eq!(tile / home, 3, "one full tile is three houses of ore, before recovery");
        assert!(
            tile > home,
            "a tile that could not build a house would make mining pointless"
        );
        // The record's "house and a half" is this number *after* recovery, and recovery is `[NS]`.
        // When it lands, the assertion to add is `tile * recovery / 1_000_000_000 ≈ home * 1.5`.
    }

    /// A structure the audit could not weigh is reported **even in a world that otherwise
    /// balances**, because a defect that only shows up when something else is already wrong is a
    /// defect that waits for the worst moment to appear.
    #[test]
    fn an_unweighable_structure_is_reported_by_a_balanced_audit() {
        let audit = MassAudit {
            defects: vec!["structure 3 could not be weighed".to_string()],
            ..Default::default()
        };
        assert!(audit.conserves(), "a held-nothing world conserves trivially");
        assert_eq!(audit.findings(), vec!["structure 3 could not be weighed"]);
    }

    #[test]
    fn a_quiet_world_balances_at_zero() {
        let ledger = Ledger::new();
        assert!(ledger.balances());
        assert_eq!(ledger.total_g(), 0);
        assert!(ledger.findings().is_empty(), "an empty ledger has nothing to report");
    }

    /// The property the campaign exists to check, end to end: what is taken from the ground
    /// appears as a holding, moves through the city without ever being created, and leaves only
    /// as something the world can point at.
    #[test]
    fn the_ground_goes_down_exactly_as_the_city_goes_up() {
        let mut ledger = Ledger::new();

        // Extraction is two-sided: the ground is debited, the stock credited.
        assert!(ledger
            .convert("ground:iron_ore", 345_600_000, "stock:iron_ore", 345_600_000)
            .is_none());
        assert!(ledger.balances(), "a two-sided movement cannot unbalance the world");
        assert_eq!(ledger.of("ground:iron_ore"), -345_600_000);
        assert_eq!(ledger.of("stock:iron_ore"), 345_600_000);
        assert_eq!(
            ledger.holdings().len(),
            2,
            "the same mass is visible on both sides, which is what makes it auditable"
        );

        // Smelting moves it again, and the world is no richer for it.
        assert!(ledger
            .convert("stock:iron_ore", 345_600_000, "stock:steel", 345_600_000)
            .is_none());
        assert!(ledger.balances());
        assert_eq!(ledger.of("stock:iron_ore"), 0);
        assert_eq!(ledger.of("stock:steel"), 345_600_000);

        // Waste is mass that has left the city but not the world, and it still balances.
        assert!(ledger.convert("stock:steel", 400, "waste:steel", 400).is_none());
        assert!(ledger.balances(), "nothing was invented and nothing was lost");
        assert!(ledger.findings().is_empty());
    }

    /// A holding is a different account from the ground it came from, and neither name is a
    /// prefix of the other in the wrong direction: two keys that could collide would cancel the
    /// movement the ledger is checking.
    #[test]
    fn a_site_and_a_carrier_hold_under_their_own_names() {
        assert_eq!(site_account(41, "timber"), "site:41:timber");
        assert_eq!(carried_account(3, "timber"), "carried:carrier-3:timber");
        assert_ne!(site_account(41, "timber"), site_account(41, "stone"));
        assert_ne!(site_account(41, "timber"), carried_account(41, "timber"));
        assert!(site_account(41, "timber").starts_with(&site_prefix(41)));
        assert!(carried_account(3, "timber").starts_with(&carried_prefix(3)));
        assert!(!site_prefix(4).starts_with(&site_prefix(41)));

        // Mass held at a site is the city's, and it is not standing: the two totals move
        // independently, which is what makes "taken but not yet spent" a readable state.
        let audit = MassAudit { extracted_g: 3000, standing_g: 1000, held_g: 2000, ..Default::default() };
        assert!(audit.conserves());
        assert_eq!(audit.loose_g(), 0, "everything taken is standing or held");
        assert!(audit.holdings_are_paid_for());

        let unpaid = MassAudit { extracted_g: 3000, standing_g: 1000, held_g: 3000, ..Default::default() };
        assert!(!unpaid.conserves(), "a holding larger than the take is the defect");
        assert!(!unpaid.holdings_are_paid_for());
    }

    /// A one-sided movement is the bug this whole file exists to catch, and it must break the
    /// total — otherwise the audit is decoration.
    #[test]
    fn a_one_sided_movement_is_the_defect() {
        let mut ledger = Ledger::new();
        ledger.record("stock:steel", 400);
        assert!(!ledger.balances(), "400 g appeared from nowhere");
        let findings = ledger.findings();
        assert!(findings.len() == 1, "one finding, not one per account: {findings:?}");
        assert!(findings[0].contains("does not balance"), "{findings:?}");
        assert!(findings[0].contains("400 g"), "{findings:?}");
    }

    /// A conversion that does not conserve is refused and recorded as the defect it is.
    #[test]
    fn a_conversion_that_does_not_conserve_is_a_defect_not_a_rounding() {
        let mut ledger = Ledger::new();
        let defect = ledger
            .convert("iron_ore", 1000, "steel", 999)
            .expect("a gram is a defect");
        assert!(defect.contains("1000 g in, 999 g out"), "{defect}");
        assert!(defect.contains("difference of 1 g"), "{defect}");
        assert_eq!(ledger.total_g(), 0, "a refused conversion changes nothing");
        assert!(ledger.balances());
    }

    /// Two runs of the same movements list the same accounts in the same order, so a defect
    /// report can be diffed.
    #[test]
    fn a_listing_is_stable_between_runs() {
        let build = || {
            let mut ledger = Ledger::new();
            for (account, grams) in [
                ("stock:steel", 10),
                ("ground:iron_ore", -20),
                ("stock:clay", 5),
                ("ground:iron_ore", 3),
            ] {
                ledger.record(account, grams);
            }
            ledger.entries().map(|(name, _)| name.to_string()).collect::<Vec<_>>()
        };
        assert_eq!(build(), build());
        assert_eq!(
            build(),
            vec!["ground:iron_ore", "stock:clay", "stock:steel"],
            "alphabetical, and stable"
        );
    }
}
