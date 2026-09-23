//! The city.
//!
//! Everything here is a pure function of `(seed, tick, input events)`. No wall
//! clock, no floating-point randomness, no hidden global state: the headless
//! verifier re-runs this exact code with no window attached, and a save
//! replays to the same city.

pub mod citizen;
pub mod road;
pub mod rng;
pub mod task;
pub mod terrain;

use serde::{Deserialize, Serialize};

use crate::gov::RetirementReason;
use crate::materials::chain as material_chain;
use crate::materials::effects as material_effects;
use crate::materials::geology as material_geology;
use crate::materials::schema as material_schema;
use crate::materials::ledger as material_ledger;
use crate::materials::surface as material_surface;
use crate::materials::generated::REPAIR_CEILING;
use crate::materials::generated as material_generated;
use crate::materials::world::{self as material_world, Level};
use citizen::{Citizen, CitizenState};
use task as sim_task;
use sim_task::Task;
use rng::Pcg32;
use road::RoadGraph;

/// The save format this build writes.
///
/// a152 replaced "`serde(default)` and derive the material quietly" with a **format
/// version and a named migration, with the derivation reported rather than silent**:
/// deriving what a structure is made of is a decision about the historical record, and
/// a decision that happens without a report is a decision nobody can check. v1 is every
/// save written before the world had materials; v2 carries them.
pub const FORMAT_VERSION: u32 = 2;

/// The condition a structure is in when it is new. 1.0 is as-built.
fn condition_as_built() -> f32 {
    1.0
}

/// The material a structure was built from, as its `MAT-*` ticket claims it.
///
/// Strings rather than the enums they resolve from, on purpose: the claim is recorded in
/// the material table's own vocabulary, so a reader compares it against
/// `materials::world` **by name**, and a renamed family is then a mismatch rather than a
/// silent equivalence.
///
/// a155 is why this is split from [`Building::condition`]: this field is the as-built
/// claim and it never changes, because a claim that weathers would make `verify.exe`
/// report the city's own ageing as a contradiction. What decays is the condition; what
/// the frame shows is a level derived from it, and a drawn level that disagrees with the
/// condition is a renderer finding rather than a claim contradiction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialClaim {
    pub part: String,
    pub family: String,
    pub anchor: String,
    pub level: String,
}

impl MaterialClaim {
    /// The claim for a structure of this kind, read off the declared world mapping.
    ///
    /// The **body** part is the claim: a structure has several parts and `world::PARTS`
    /// keeps every one of them declared, so what a single ticket records is the part that
    /// makes the structure what it is — and the rest stay checkable against the table by
    /// name. This is a reading of a175's "carrying family, anchor and level", recorded as
    /// a reading and overridable if the claim should be per part instead.
    pub fn of_kind(kind: BuildingKind) -> Option<Self> {
        let parts = material_world::parts_of(kind.part_prefix());
        let body = parts
            .iter()
            .find(|part| part.level == Level::Body)
            .or_else(|| parts.first())?;
        Some(Self {
            part: body.part.to_string(),
            family: body.family.as_str().to_string(),
            anchor: body.hue.as_str().to_string(),
            level: body.level.as_str().to_string(),
        })
    }

    /// The claim as a ticket states it.
    pub fn describe(&self) -> String {
        format!("{}/{}/{} on {}", self.family, self.anchor, self.level, self.part)
    }
}

/// What a v1 → v2 migration derived, and from what.
///
/// The report exists because the derivation is a decision: it says how many structures
/// needed a material invented for them and which materials were chosen, so a reader can
/// disagree with the choice instead of discovering it by noticing that every old building
/// happens to be ceramic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Migration {
    pub from: u32,
    pub to: u32,
    /// `(building id, the claim derived for it)`, in the order they were derived.
    pub derived: Vec<(u32, String)>,
}

impl Migration {
    pub fn describe(&self) -> String {
        if self.derived.is_empty() {
            return format!(
                "save format v{} → v{}: no structures needed a material derived; the \
                 tables and the road graph were already there",
                self.from, self.to
            );
        }
        let mut by_claim: Vec<(String, usize)> = Vec::new();
        for (_, claim) in &self.derived {
            match by_claim.iter_mut().find(|(name, _)| *name == *claim) {
                Some((_, count)) => *count += 1,
                None => by_claim.push((claim.clone(), 1)),
            }
        }
        let tallies: Vec<String> = by_claim
            .iter()
            .map(|(claim, count)| format!("{count} × {claim}"))
            .collect();
        format!(
            "save format v{} → v{}: derived the as-built material of {} structure(s) from \
             their own kinds against the declared mapping — {}. This is a derivation, not a \
             measurement: the structures were built before the world had materials, and a \
             claim recorded from here on is a claim.",
            self.from,
            self.to,
            self.derived.len(),
            tallies.join(", ")
        )
    }
}

/// Simulation rate. Fixed, and never negotiated with the renderer.
pub const SIM_HZ: u32 = 20;
/// Sim-ticks in one game day. At 1x, a day is two seconds.
pub const TICKS_PER_DAY: u64 = 40;
pub const DAYS_PER_MONTH: u64 = 30;
pub const MONTHS_PER_YEAR: u64 = 12;
/// How long construction takes, in ticks.
pub const BUILD_TICKS: u64 = 40;
/// Tiles within this distance of the camera focus are simulated as full agents.
/// Stored in the save so a replay reproduces the same level-of-detail split.
pub const LOD_RADIUS: i32 = 48;
/// How far a worker can reach from where they stand (C9 phase 3).
///
/// One declared number, used by both halves of the question: the router walks a citizen to the
/// end of the road, and this is how far past it their hands reach. Making the two agree is what
/// stops a tile from being **postable but unworkable** — a site three tiles off a road could be
/// posted under the old one-tile check and never worked, which is a stall with no case against
/// it.
pub const WORK_REACH: u32 = 3;

/// The good the first rung ends in (Q7's hatchet, Q102's first-tier rung).
///
/// A named constant rather than a literal, because the audit's "does this city have one yet"
/// question and the plan that makes it have to agree about what *one* is.
pub const RUNG_GOOD: &str = "hatchet";

/// A power plant lights this many road-distance tiles.
pub const POWER_RADIUS: u32 = 14;
pub const POWER_PER_PLANT: u32 = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terrain {
    Ground,
    Water,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Zone {
    None,
    Residential,
    Commercial,
    Industrial,
}

impl Zone {
    pub fn name(self) -> &'static str {
        match self {
            Zone::None => "unzoned",
            Zone::Residential => "residential",
            Zone::Commercial => "commercial",
            Zone::Industrial => "industrial",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub terrain: Terrain,
    pub zone: Zone,
    pub road: bool,
    /// Index into [`World::buildings`] for the structure standing here.
    pub building: Option<u32>,
    pub powered: bool,
    /// How much of this tile's deposit has been taken out, in grams (C9 phase 1).
    ///
    /// The deposit itself is **derived** from the world's seed — `materials::geology` — and
    /// never stored; this is the delta, which is round 9's Q59 rule applied to the ground.
    /// It defaults to zero, and that default is *correct* rather than convenient: a save
    /// written before this field existed has extracted nothing from a field it did not know
    /// about. That is why the format does not need a version bump for it — a default that
    /// would be a lie needs a migration, and this one would not be.
    #[serde(default)]
    pub extracted_g: i64,
    /// How much of this tile's **surface** patch has been taken, in grams (C9, Q102).
    ///
    /// The patch itself is derived from the seed — `materials::surface` — and never stored. Two
    /// deltas rather than one because the two are different resources on the same tile: a stand
    /// of timber can stand over a seam of stone, and working one says nothing about the other.
    ///
    /// This is the patch's **take total**, not its shortfall: it never goes down when the patch
    /// grows back, so the mass audit cannot read regrowth as material the city never dug.
    #[serde(default)]
    pub harvested_g: i64,
    /// The sim-day of the last take from this patch, which is what makes regrowth a function of
    /// elapsed time rather than of ticks (C9 round 9, Q61).
    #[serde(default)]
    pub harvested_day: u64,
    /// Whether this patch has been **stripped to the soil** (Q108): the remainder is soil rather
    /// than a standing patch, so it grows nothing back until it is planted. Planting is a task
    /// this rung does not build; the flag is what makes stripping a decision rather than a
    /// thing you can do forever without noticing.
    #[serde(default)]
    pub stripped: bool,
}

impl Tile {
    /// The family this tile's surface presents as (C9 phase 1).
    ///
    /// Derived rather than stored, like the deposit: a tile's surface is a reading of what is
    /// already on it, so there is no second copy to drift. `soil` for bare ground is the
    /// declared reading — `DEFERRED_SURFACES` says *"`soil` is the road shoulder today"*, so a
    /// plausible unpaved surface is a shoulder rather than an invention.
    pub fn surface_family(&self) -> &'static str {
        match self.terrain {
            Terrain::Water => "water",
            Terrain::Ground if self.road => "road",
            Terrain::Ground => "soil",
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            terrain: Terrain::Ground,
            zone: Zone::None,
            road: false,
            building: None,
            powered: false,
            extracted_g: 0,
            harvested_g: 0,
            harvested_day: 0,
            stripped: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingKind {
    Home,
    Shop,
    Factory,
    PowerPlant,
}

impl BuildingKind {
    pub fn name(self) -> &'static str {
        match self {
            BuildingKind::Home => "home",
            BuildingKind::Shop => "shop",
            BuildingKind::Factory => "factory",
            BuildingKind::PowerPlant => "power plant",
        }
    }

    /// The prefix its parts carry in the declared world mapping (`home.walls`, …).
    ///
    /// Separate from [`BuildingKind::name`] because the mapping's names are identifiers
    /// while the display name is prose: "power plant" is not a prefix, `power` is.
    pub fn part_prefix(self) -> &'static str {
        match self {
            BuildingKind::Home => "home",
            BuildingKind::Shop => "shop",
            BuildingKind::Factory => "factory",
            BuildingKind::PowerPlant => "power",
        }
    }

    pub fn zone(self) -> Zone {
        match self {
            BuildingKind::Home => Zone::Residential,
            BuildingKind::Shop => Zone::Commercial,
            BuildingKind::Factory => Zone::Industrial,
            BuildingKind::PowerPlant => Zone::None,
        }
    }

    pub fn build_cost(self) -> i64 {
        match self {
            BuildingKind::Home => 120,
            BuildingKind::Shop => 240,
            BuildingKind::Factory => 400,
            BuildingKind::PowerPlant => 1_800,
        }
    }

    pub fn capacity(self) -> u32 {
        match self {
            BuildingKind::Home => 8,
            BuildingKind::Shop => 6,
            BuildingKind::Factory => 12,
            BuildingKind::PowerPlant => 0,
        }
    }
}

/// One structure's material provenance: where its mass actually came from (round 11's Q75).
///
/// The record asked for **tile lineage** rather than a fungible total, and this is what that
/// buys: a ruin knows what it is made of and where that came from, salvage can be reasoned about
/// without a second accounting of the whole world, and "this house is 100 t of brick" has a
/// witness per gram instead of an assertion.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialLineage {
    /// The tile the ground gave it up from.
    pub tile: u32,
    /// The substance as it came out of the ground — ore, not steel; the substance vocabulary is
    /// what the ground speaks, and processing is a later link in the chain that will have its own
    /// rows here rather than rewriting these.
    pub substance: String,
    /// How much of it, in grams, exactly.
    pub grams: i64,
}

/// What a tile's **surface patch** stands right now (C9, Q102): what grows there, and how much
/// of it is there to take.
///
/// `stripped` is carried rather than recomputed by the caller because it is a *reading of the
/// tile*, not a second opinion about it — see `materials::surface` for why stripping is what
/// stops a renewable from renewing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Standing {
    pub substance: &'static str,
    pub mass_g: i64,
    pub stripped: bool,
}

/// Whether a worker standing on one tile is at work on another: [`WORK_REACH`] in both axes.
///
/// A free function rather than a method because the work loop holds the task list mutably while
/// it asks the question, and a borrow of the whole world to compare two tile indices would be a
/// worse trade than a width parameter.
pub fn within_reach(width: u32, standing: u32, target: u32) -> bool {
    let (sx, sy) = (standing % width, standing / width);
    let (tx, ty) = (target % width, target / width);
    (sx as i64 - tx as i64).abs() <= WORK_REACH as i64
        && (sy as i64 - ty as i64).abs() <= WORK_REACH as i64
}

/// The grams one run of a gather process yields of a substance, or `None` when that process does
/// not produce it at all — which is a table defect rather than a shortage.
fn gather_per_run(process: &material_generated::Process, substance: &str) -> Option<i64> {
    process
        .outputs
        .iter()
        .find(|(name, _)| *name == substance)
        .and_then(|(name, amount)| material_schema::quantity_grams(name, *amount))
}

/// A refusal to build, because the world could not supply the mass.
///
/// Round 11's Q74 answered that a refusal **files a case** rather than passing quietly: the
/// shortfall is a thing that happened to the world, and the world needs to be able to say what
/// it wanted, what it found, and from what kind of material. This type is the claim; the case is
/// filed by whoever asked — and if nobody asks, nothing was built either way.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialShortfall {
    pub site: u32,
    /// What was wanted: a substance (`timber`), a family (`ceramic`), or the name of the thing
    /// that refused — a process that cannot run, or a rung that cannot be planned.
    pub family: String,
    pub wanted_g: i64,
    pub found_g: i64,
    /// The sentence a defect would print, when the refusal is a defect rather than a shortage.
    /// A refusal a reader cannot act on is a refusal nobody can act on (Q74).
    pub note: String,
}

impl MaterialShortfall {
    /// A refusal naming what was wanted and how much of it was there.
    pub fn of(site: u32, name: impl Into<String>, wanted_g: i64, found_g: i64) -> Self {
        Self { site, family: name.into(), wanted_g, found_g, note: String::new() }
    }

    /// The same refusal, with the defect that caused it named — used where the reason is a broken
    /// row rather than an empty world.
    pub fn noting(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }
}

/// Where the city is short of material, aggregated by district and family — the reading a case
/// is sampled from (round 11's Q74).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Starved {
    pub district: u32,
    pub family: String,
    /// The **substance** the refusal names, when it can name one — a leaf's shortage is a
    /// substance (`timber`), not the family it presents as (`organic`), and a reader who cannot
    /// size a refusal cannot act on it (Q74). Empty for a structure's family-level shortage, whose
    /// substance is genuinely several.
    #[serde(default)]
    pub substance: String,
    /// The defect behind the refusal, when there was one: a process that does not balance, or a
    /// rung that cannot be planned. Empty for a plain shortage.
    #[serde(default)]
    pub note: String,
    /// Total wanted across every refusal folded into this reading, in grams.
    pub wanted_g: i64,
    /// Total the world could supply, in grams.
    pub found_g: i64,
    /// How many refusals this reading stands for.
    pub count: u32,
}

impl Starved {
    /// The shortfall this reading stands for, in grams.
    pub fn missing_g(&self) -> i64 {
        (self.wanted_g - self.found_g).max(0)
    }

    /// The case key, so repeated refusals update one reading instead of filing a thousand cases —
    /// the same dedupe the case engine already does, keyed by what is actually short.
    pub fn case_key(&self) -> String {
        format!("materials:{}:{}:{}", self.district, self.family, self.substance)
    }

    /// What the refusal is short of: the substance when one is named, the family otherwise.
    pub fn wanted_name(&self) -> &str {
        if self.substance.is_empty() {
            &self.family
        } else {
            &self.substance
        }
    }

    /// What a person reads. Round 11's Q74 asked for a refusal that says what it wanted and what
    /// it found; round 5's Q36 said the government owns what nobody owns, so the objective names
    /// the shortage as work rather than as a complaint.
    pub fn objective(&self) -> String {
        format!(
            "{} site(s) in district {} want {} g of {} and the world holds {} g: {} g short{}",
            self.count,
            self.district,
            self.wanted_g,
            self.wanted_name(),
            self.found_g,
            self.missing_g(),
            if self.note.is_empty() {
                String::new()
            } else {
                format!(" — {}", self.note)
            }
        )
    }
}

impl MaterialShortfall {
    /// The shortfall in grams — the number a case has to carry.
    pub fn missing_g(&self) -> i64 {
        (self.wanted_g - self.found_g).max(0)
    }

    /// What a person reads, in the shape the report already uses for a claim the world does not
    /// support.
    pub fn describe(&self) -> String {
        format!(
            "a structure at tile {} needs {} g of {} and the world holds {} g within reach: \
             {} g short",
            self.site,
            self.wanted_g,
            self.family,
            self.found_g,
            self.missing_g()
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Building {
    pub id: u32,
    pub kind: BuildingKind,
    pub tile: u32,
    pub level: u8,
    pub built_tick: u64,
    pub ready_tick: u64,
    /// A demolished building is retired, never removed: the record of what the
    /// city used to be is part of the city.
    pub retired: Option<RetirementReason>,
    pub retired_tick: Option<u64>,
    pub powered: bool,
    pub occupants: u32,
    /// The material this structure was built from (a155, a175). `None` means the save
    /// predates the claim: the v1 → v2 migration fills it, and every reader treats a
    /// `None` in a v2 save as a **defect** rather than as "an unknown material", because
    /// an unknown material is exactly what a missing report would hide.
    #[serde(default)]
    pub material_as_built: Option<MaterialClaim>,
    /// Present condition, 1.0 being as-built. What decays; never the claim above.
    #[serde(default = "condition_as_built")]
    pub condition: f32,
    /// The tiles this structure's mass came out of (round 11's Q75).
    ///
    /// **Empty is a placement that made no material claim** — a direct `place_building`, which is
    /// the raw primitive the world state uses and not the way a structure is built. It is not
    /// reported as a defect by the mass audit, because the audit has one job and it is mass; but
    /// it is visible rather than hidden, since [`Building::lineage_g`] returns zero against a
    /// non-zero derived mass. Filling it is the task system's business, and a structure whose
    /// lineage is present but **disagrees** with its derived mass is a defect.
    #[serde(default)]
    pub material_from: Vec<MaterialLineage>,
    /// Whether this structure is known to owe a repair, so a crossed floor files
    /// **one** ticket rather than one per tick. Cleared by the repair that raises
    /// the condition back over the floor, not by the ticket being opened.
    #[serde(default)]
    pub repair_filed: bool,
}

impl Building {
    /// This structure's mass, in grams, **derived from what it is** rather than stored.
    ///
    /// Round 9's Q59 rule — a save stores the seed and its deltas, and everything derivable is
    /// derived — applies to a building's mass exactly as it applies to the ground's deposit:
    /// there is one fact (this is a home, it is level 2, it claims ceramic) and the mass is a
    /// reading of it. A stored mass could disagree with the level, and the disagreement would be
    /// invisible until the ledger failed for a reason nobody could trace.
    ///
    /// `None` means the kind has no declared mass, which is a defect rather than a zero: a
    /// structure that weighs nothing enters the ledger as nothing and hides itself, so the
    /// caller is made to decide what that means instead of being handed a zero that looks fine.
    pub fn mass_g(&self) -> Option<i64> {
        material_ledger::structure_mass_g(self.kind.name(), self.level)
    }

    /// The mass this structure's recorded lineage accounts for, in grams. Zero means no material
    /// claim was made, not that it weighs nothing — [`Building::mass_g`] is the mass.
    pub fn lineage_g(&self) -> i64 {
        self.material_from.iter().map(|line| line.grams).sum()
    }

    /// The account this structure's mass sits in: standing structures, or — once retired — the
    /// remains, because a demolition preserves the material as surely as it preserves the record.
    pub fn mass_account(&self, family: &str) -> String {
        match self.retired {
            None => material_ledger::structure_account(family),
            Some(_) => material_ledger::ruin_account(family),
        }
    }

    /// The appearance family this structure's material belongs to, read off its claim, falling
    /// back to the kind's declared body part. `None` only when neither knows — which the v1 → v2
    /// migration already treats as a defect to report rather than a material to guess.
    pub fn material_family(&self) -> Option<&str> {
        self.material_as_built
            .as_ref()
            .map(|claim| claim.family.as_str())
            .or_else(|| {
                // The same derivation `MaterialClaim::of_kind` makes, read for its family: the
                // declared **body** part, which is the part that makes the structure what it is.
                let parts = material_world::parts_of(self.kind.part_prefix());
                parts
                    .iter()
                    .find(|part| part.level == Level::Body)
                    .or_else(|| parts.first())
                    .map(|part| part.family.as_str())
            })
    }

    /// The colour the frame draws this structure in: its own claimed material.
    ///
    /// Lives here rather than in the frame so it is **checkable** — a rule that only a
    /// running window could exercise is a rule that rots. The claim is world state, so
    /// this is a lookup; the fallback derives the kind's declared body material (the same
    /// derivation the v1 → v2 migration makes) rather than inventing a token, so a drawn
    /// colour is always one the table names.
    pub fn drawn_colour(&self) -> Option<[f32; 4]> {
        let claim = self
            .material_as_built
            .clone()
            .or_else(|| MaterialClaim::of_kind(self.kind))?;
        crate::materials::colour_of(&claim.family, &claim.anchor, &claim.level, 1.0)
    }

    /// The colour its ruin is drawn in: the same family and anchor, read one level down.
    ///
    /// `materials::world::RULES` already declares this — *"the retired structure's own
    /// parts, read at `deep`"* — and the frame drew every ruin the same grey before this.
    pub fn ruin_colour(&self, alpha: f32) -> Option<[f32; 4]> {
        let claim = self
            .material_as_built
            .clone()
            .or_else(|| MaterialClaim::of_kind(self.kind))?;
        crate::materials::colour_of(&claim.family, &claim.anchor, "deep", alpha)
    }

    pub fn is_ready(&self, tick: u64) -> bool {
        self.retired.is_none() && tick >= self.ready_tick
    }

    pub fn capacity(&self) -> u32 {
        self.kind.capacity() + self.kind.capacity() * self.level.saturating_sub(1) as u32
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Economy {
    /// City funds. Fiction — labelled as fiction everywhere it is shown.
    pub credits: i64,
    pub tax_residential: f32,
    pub tax_commercial: f32,
    pub tax_industrial: f32,
    pub loan_balance: i64,
    pub month_income: i64,
    pub month_expense: i64,
    pub lifetime_income: i64,
    pub lifetime_expense: i64,
    pub months_in_debt: u32,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Demand {
    pub residential: f32,
    pub commercial: f32,
    pub industrial: f32,
}

impl Demand {
    pub fn for_zone(&self, zone: Zone) -> f32 {
        match zone {
            Zone::Residential => self.residential,
            Zone::Commercial => self.commercial,
            Zone::Industrial => self.industrial,
            Zone::None => 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Clock {
    pub tick: u64,
}

impl Clock {
    pub fn day(&self) -> u64 {
        self.tick / TICKS_PER_DAY
    }

    /// One-based, because "month 0" is not a month.
    pub fn month(&self) -> u64 {
        (self.day() / DAYS_PER_MONTH) % MONTHS_PER_YEAR + 1
    }

    pub fn year(&self) -> u64 {
        self.day() / (DAYS_PER_MONTH * MONTHS_PER_YEAR) + 1
    }

    /// True on the single tick that closes a month, so income posts once.
    pub fn closes_month(&self) -> bool {
        self.tick > 0 && self.tick.is_multiple_of(TICKS_PER_DAY * DAYS_PER_MONTH)
    }

    pub fn label(&self) -> String {
        format!("Y{} M{:02} D{:02}", self.year(), self.month(), self.day())
    }

    /// Sim-seconds elapsed. The stall bound is expressed in these, so it holds
    /// at every game speed.
    pub fn sim_seconds(&self) -> f64 {
        self.tick as f64 / SIM_HZ as f64
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Stats {
    pub population: u32,
    pub homes: u32,
    pub shops: u32,
    pub factories: u32,
    pub power_plants: u32,
    pub retired_buildings: u32,
    pub jobs: u32,
    pub employed: u32,
    pub unemployed: u32,
    pub road_tiles: u32,
    pub powered_tiles: u32,
    pub unpowered_zoned: u32,
    pub commute_failures: u32,
    pub full_agents: u32,
    pub brownout: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    pub width: u32,
    pub height: u32,
    pub seed: u64,
    pub tiles: Vec<Tile>,
    pub buildings: Vec<Building>,
    pub citizens: Vec<Citizen>,
    pub economy: Economy,
    pub demand: Demand,
    pub clock: Clock,
    pub stats: Stats,
    pub rng: Pcg32,
    /// Where the camera is looking, in tiles. Part of the saved state because
    /// level-of-detail is driven by it, and a replay must split identically.
    pub focus: (i32, i32),
    pub next_building_id: u32,
    pub next_citizen_id: u32,
    /// Open and claimed work (round 11's Q43).
    #[serde(default)]
    pub tasks: Vec<Task>,
    #[serde(default = "first_task_id")]
    pub next_task_id: u32,
    /// Where the city is short of material, for the case engine to read (round 11's Q74).
    ///
    /// Aggregated rather than appended, so it is bounded by districts × families instead of
    /// growing with every refused tick.
    #[serde(default)]
    pub starved: Vec<Starved>,
    /// Where the city's **loose mass** is: what stands at a site and what a carrier is holding
    /// (C9 round 13, Q77).
    ///
    /// A `Ledger` rather than a second table of stocks, because a holding is an account like any
    /// other and the audit is the thing that has to read it: `site:<tile>:<substance>` and
    /// `carried:carrier-<id>:<substance>` are the two names Q77 settled, and mass moves between
    /// them, the ground and a structure without ever having one side. Nothing here is a second
    /// source of truth — it is the world's own mass, written down where it is standing.
    #[serde(default)]
    pub holdings: material_ledger::Ledger,
    /// Which save format wrote this. Absent (0) is a v1 save: it predates the world
    /// having materials at all.
    #[serde(default)]
    pub format_version: u32,
    /// What the migration derived on the way in, if it ran. Skipped in the save: this is
    /// a report about reading a file, not part of the world.
    #[serde(skip)]
    pub migration: Option<Migration>,

    /// Derived from `tiles`. Rebuilt after any road or load, never serialised:
    /// two sources of truth for where a road is would be one too many.
    #[serde(skip)]
    pub roads: RoadGraph,
    #[serde(skip)]
    road_access: Vec<bool>,
}

impl World {
    pub fn new(width: u32, height: u32, seed: u64) -> Self {
        let mut tiles = vec![Tile::default(); (width * height) as usize];
        for y in 0..height {
            for x in 0..width {
                let n = terrain::fbm(x as f32 * 0.045, y as f32 * 0.045, seed, 4);
                tiles[(y * width + x) as usize].terrain = if n < 0.34 {
                    Terrain::Water
                } else {
                    Terrain::Ground
                };
            }
        }

        // The map edge carries a road, so a new city has something to connect
        // to instead of starting from nowhere.
        for y in 0..height {
            let tile = &mut tiles[(y * width) as usize];
            if tile.terrain == Terrain::Ground {
                tile.road = true;
            }
        }

        let mut world = Self {
            width,
            height,
            seed,
            tiles,
            buildings: Vec::new(),
            citizens: Vec::new(),
            tasks: Vec::new(),
            next_task_id: first_task_id(),
            starved: Vec::new(),
            holdings: material_ledger::Ledger::new(),
            economy: Economy {
                credits: 25_000,
                tax_residential: 0.09,
                tax_commercial: 0.09,
                tax_industrial: 0.09,
                ..Default::default()
            },
            demand: Demand {
                residential: 0.6,
                commercial: 0.25,
                industrial: 0.35,
            },
            clock: Clock::default(),
            format_version: FORMAT_VERSION,
            migration: None,
            stats: Stats::default(),
            rng: Pcg32::new(seed),
            focus: (width as i32 / 2, height as i32 / 2),
            next_building_id: 1,
            next_citizen_id: 1,
            roads: RoadGraph::default(),
            road_access: Vec::new(),
        };
        world.rebuild_derived();
        world
    }

    /// Rebuild everything derived from the tiles. Must be called after load.
    pub fn rebuild_derived(&mut self) {
        self.roads = RoadGraph::rebuild(self.width, self.height, &self.tiles);

        // Road access: within two tiles of a road, and on ground.
        let mut access = vec![false; self.tiles.len()];
        for tile in 0..self.tiles.len() as u32 {
            if self.tiles[tile as usize].terrain != Terrain::Ground {
                continue;
            }
            if self.tiles[tile as usize].road {
                access[tile as usize] = true;
                continue;
            }
            access[tile as usize] = self
                .roads
                .nearest_node_within(self.width, self.height, tile, 2)
                .is_some();
        }
        self.road_access = access;
        self.recount();
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }

    pub fn index(&self, x: u32, y: u32) -> u32 {
        y * self.width + x
    }

    pub fn coords(&self, tile: u32) -> (u32, u32) {
        (tile % self.width, tile / self.width)
    }

    pub fn has_road_access(&self, tile: u32) -> bool {
        self.road_access.get(tile as usize).copied().unwrap_or(false)
    }

    pub fn tile(&self, tile: u32) -> &Tile {
        &self.tiles[tile as usize]
    }

    pub fn building(&self, index: u32) -> &Building {
        &self.buildings[index as usize]
    }

    /// The structure standing on a tile, if any.
    ///
    /// Added because an **id is not an index**: [`World::place_building`] returns an id,
    /// ids start at 1 and survive demolition, while an index is a position in a vector
    /// that shrinks and grows. Reading a claim through one when you have the other is a
    /// bug that waits until the second structure exists before showing itself.
    pub fn building_on(&self, tile: u32) -> Option<&Building> {
        self.tile(tile).building.map(|index| self.building(index))
    }

    // ---------------------------------------------------------------------
    // Player actions. Each returns a description of what actually changed,
    // which the caller turns into a ticket. Nothing here decides whether the
    // action was *authorised* -- that is the governor's job, and it happens
    // before these are ever called.
    // ---------------------------------------------------------------------

    /// The tiles a straight leg between two tiles passes through, without
    /// changing anything. The client asks for these *before* building so it can
    /// see what already existed — which is what a regression check compares
    /// against afterwards.
    pub fn leg_tiles(&self, from: u32, to: u32) -> Vec<u32> {
        let (x0, y0) = self.coords(from);
        let (x1, y1) = self.coords(to);
        let mut tiles = Vec::new();
        let (mut x, mut y) = (x0 as i32, y0 as i32);
        loop {
            tiles.push(self.index(x as u32, y as u32));
            if x == x1 as i32 && y == y1 as i32 {
                break;
            }
            if x != x1 as i32 {
                x += (x1 as i32 - x).signum();
            } else {
                y += (y1 as i32 - y).signum();
            }
        }
        tiles
    }

    /// Lay road along a straight leg between two tiles. Returns the tiles that
    /// became road (the set the caller must read back later to validate).
    pub fn lay_road(&mut self, from: u32, to: u32) -> Vec<u32> {
        let mut laid = Vec::new();
        for tile in self.leg_tiles(from, to) {
            let t = &mut self.tiles[tile as usize];
            if t.terrain == Terrain::Ground && !t.road {
                t.road = true;
                laid.push(tile);
            }
        }
        self.rebuild_derived();
        laid
    }

    /// Zone a tile. Returns true when the zone actually changed.
    pub fn set_zone(&mut self, tile: u32, zone: Zone) -> bool {
        let t = &mut self.tiles[tile as usize];
        if t.terrain != Terrain::Ground || t.road || t.building.is_some() {
            return false;
        }
        if t.zone == zone {
            return false;
        }
        t.zone = zone;
        true
    }

    // ---------------------------------------------------------------------
    // The ground's material (C9 phase 1). What a tile holds is derived; what
    // has been taken out is stored. Nothing else about a deposit is state.
    // ---------------------------------------------------------------------

    /// Travel speed over a tile's surface, as a factor on the road graph's own step cost.
    ///
    /// This is §8.24's deferred `SURFACE_SPEED`, which had **no consumer** until a tile could
    /// say what it presents as: the graph only ever inserted tiles where `road` was true, so
    /// every node was already the table's own `road: 1.0` and every other entry was
    /// unreachable. Now a bare tile presents as soil (0.6) and water as itself (0.0) —
    /// *impassable rather than slow*, which is what the declaration says.
    ///
    /// It panics on an undeclared family rather than defaulting, for the same reason
    /// `effects::part_price` does: a speed that silently becomes zero is a road nobody can
    /// drive on, and a speed that silently becomes one is a swamp nobody can feel.
    pub fn surface_speed(&self, tile: u32) -> f32 {
        let family = self.tiles[tile as usize].surface_family();
        material_generated::SURFACE_SPEED
            .iter()
            .find(|(name, _)| *name == family)
            .map(|(_, speed)| *speed)
            .unwrap_or_else(|| panic!("`{family}` has no declared surface speed"))
    }

    /// What this tile still holds, in grams — derived from the seed and net of what has
    /// been taken out. `None` where the ground is barren or worked out.
    pub fn deposit_at(&self, tile: u32) -> Option<material_geology::Deposit> {
        let (x, y) = self.coords(tile);
        let found = material_geology::deposit(self.seed, x, y)?;
        let remaining = found.mass_g - self.tiles[tile as usize].extracted_g;
        (remaining > 0).then_some(material_geology::Deposit {
            mass_g: remaining,
            ..found
        })
    }

    /// Take up to `grams` out of a tile, recording the delta and returning **only what was
    /// actually there**.
    ///
    /// Returning the take rather than the request is the whole point: a caller that asked
    /// for more than the ground held must not be handed a number that conservation would
    /// contradict. Padding the answer or refusing silently would both break the ledger, and
    /// the ledger is what this campaign exists to make checkable.
    pub fn extract(&mut self, tile: u32, grams: i64) -> i64 {
        let Some(found) = self.deposit_at(tile) else {
            return 0;
        };
        let taken = grams.min(found.mass_g).max(0);
        self.tiles[tile as usize].extracted_g += taken;
        taken
    }

    /// What this tile's **surface patch** stands right now, or `None` where the ground grows
    /// nothing, where it has been paved or built over, or where the patch is stripped to the
    /// soil.
    ///
    /// Derived like the deposit — seed plus the tile's own delta — with Q108's regrowth folded
    /// in as a function of elapsed **sim-days**, never of ticks.
    pub fn surface_at(&self, tile: u32) -> Option<Standing> {
        let (x, y) = self.coords(tile);
        let found = material_surface::surface(self.seed, x, y)?;
        let state = self.tiles[tile as usize];
        // A patch the city paved or built on is not a stand of timber any more. Saying so here
        // is cheaper than letting a gather take a road's worth of brush out of a road.
        if state.terrain != Terrain::Ground || state.road || state.building.is_some() {
            return None;
        }
        let rate = material_surface::tile_regrowth_g_per_day(found.kind, found.base_g);
        let mass_g = material_surface::standing_g(
            found.base_g,
            rate,
            state.harvested_g,
            state.harvested_day,
            state.stripped,
            self.clock.day(),
        );
        (mass_g > 0).then_some(Standing {
            substance: material_surface::kind_substance(found.kind),
            mass_g,
            stripped: state.stripped,
        })
    }

    /// Take up to `grams` off a tile's surface patch, recording the delta and returning **only
    /// what was actually growing there** — the same rule [`Self::extract`] follows, for the same
    /// reason.
    ///
    /// This is a *gather* (Q102): no mine, no structure, just hands on a patch. Mass enters the
    /// world here — `SCHEMA.md` says gathering is the one place it legitimately does — and the
    /// tile's `harvested_g` is the take total the audit reads back.
    pub fn harvest(&mut self, tile: u32, grams: i64) -> i64 {
        let Some(standing) = self.surface_at(tile) else {
            return 0;
        };
        let (x, y) = self.coords(tile);
        let Some(found) = material_surface::surface(self.seed, x, y) else {
            return 0;
        };
        let state = self.tiles[tile as usize];
        let rate = material_surface::tile_regrowth_g_per_day(found.kind, found.base_g);
        let day = self.clock.day();
        debug_assert_eq!(standing.mass_g, material_surface::standing_g(found.base_g, rate, state.harvested_g, state.harvested_day, state.stripped, day));
        let take = material_surface::take(
            found.base_g,
            rate,
            state.harvested_g,
            state.harvested_day,
            state.stripped,
            day,
            grams,
        );
        let updated = &mut self.tiles[tile as usize];
        updated.harvested_g = take.harvested_g;
        updated.harvested_day = day;
        updated.stripped = take.stripped;
        take.taken_g
    }

    /// What one account is holding, in grams — a site holding or a carrier's load.
    pub fn holding_of(&self, account: &str) -> i64 {
        self.holdings.of(account)
    }

    /// Credit loose mass to a holding: what a gather has just taken, or what a process has just
    /// produced.
    ///
    /// **One-sided on purpose, and it is not the bug** the ledger exists to catch: the ground's
    /// side of a gather is the tile's own delta rather than an account, and the audit reconciles
    /// the two. A move *between* holdings is [`Self::move_holding`], which cannot forget a side.
    pub fn credit_holding(&mut self, account: &str, grams: i64) {
        self.holdings.record(account, grams);
    }

    /// Move held mass from one account to another, both sides in one call. `None` means it
    /// happened; `Some(defect)` means the source could not cover it.
    ///
    /// A holding that cannot cover a move is refused rather than overdrawn: unlike the ground,
    /// which is allowed to run out, a holding going negative would be mass that nobody has — and
    /// the ledger would report it as a finding instead of preventing it.
    pub fn move_holding(&mut self, from: &str, to: &str, grams: i64) -> Option<String> {
        if grams < 0 {
            return Some(format!("a move of {grams} g is not a move"));
        }
        let held = self.holdings.of(from);
        if held < grams {
            return Some(format!(
                "`{from}` holds {held} g and cannot give {grams} g: the move would make mass \
                 that nobody has"
            ));
        }
        self.holdings.convert(from, grams, to, grams)
    }

    /// The world's mass audit, assembled from what is actually here (C9 phase 2).
    ///
    /// Two sides, and nothing else: the ground's depletion, per tile, at the substance its own
    /// deposit derives to, and the mass standing in the city, per structure, at the material it
    /// claims. That is the audit round 9's Q26 asked for — `verify.exe` reads it back, and the
    /// number it produces is the whole claim of this campaign: **the city is made of what it dug
    /// up, or the audit says by how much it is not.**
    ///
    /// It is *derived*, like everything else on this side of the boundary: nothing about it is
    /// stored, so it cannot drift from the world, and a replay of the same save computes the
    /// same accounts. Compiled with `grow()` still in place, a grown city reads as a named
    /// positive quantity of material nobody dug — and that is not a false alarm. It is `grow()`
    /// itself, printed.
    pub fn mass_audit(&self) -> material_ledger::MassAudit {
        let mut ledger = material_ledger::Ledger::new();
        let mut extracted_g = 0;
        let mut standing_g = 0;

        // The ground, from what each tile has given up — read back through the same derivation
        // that produced it, so the substance cannot be remembered wrongly. **Both halves of the
        // ground**: the seam under the tile, and the patch growing on it. They are two resources
        // on one tile, and a tile that has given up neither is a tile with nothing to report.
        for (index, tile) in self.tiles.iter().enumerate() {
            let (x, y) = (index as u32 % self.width, index as u32 / self.width);
            if tile.extracted_g != 0 {
                match material_geology::deposit(self.seed, x, y) {
                    // Taken from a tile whose deposit no longer derives: the take is still real,
                    // so it is recorded against the ground itself rather than dropped.
                    None => ledger
                        .record(&material_ledger::ground_account("unattributed"), -tile.extracted_g),
                    Some(deposit) => {
                        let substance = material_geology::kind_substance(deposit.kind);
                        ledger.record(&material_ledger::ground_account(substance), -tile.extracted_g);
                    }
                }
                extracted_g += tile.extracted_g;
            }
            if tile.harvested_g != 0 {
                match material_surface::surface(self.seed, x, y) {
                    None => ledger
                        .record(&material_ledger::ground_account("unattributed"), -tile.harvested_g),
                    Some(found) => {
                        let substance = material_surface::kind_substance(found.kind);
                        ledger.record(&material_ledger::ground_account(substance), -tile.harvested_g);
                    }
                }
                extracted_g += tile.harvested_g;
            }
        }

        // The city's **loose mass**: what stands at a site and what a carrier is holding. It is
        // the city's, it came out of the ground, and leaving it out of the audit would report a
        // city that is holding three tonnes of timber as three tonnes nobody dug.
        let mut held_g = 0;
        for (account, grams) in self.holdings.holdings() {
            ledger.record(account, grams);
            held_g += grams;
        }

        // The city, standing and in ruins, at the mass each structure derives to. A structure
        // that cannot be weighed or attributed is **reported, never skipped**: it would otherwise
        // enter the audit as nothing and hide itself, and an audit that can lose a building
        // quietly is an audit nobody can trust to find the first real one.
        let mut defects = Vec::new();
        for building in &self.buildings {
            let mass = building.mass_g();
            let family = building.material_family();
            let (Some(mass), Some(family)) = (mass, family) else {
                defects.push(format!(
                    "structure {} ({}, level {}) could not be weighed or attributed: {} — it is \
                     not in the mass audit at all",
                    building.id,
                    building.kind.name(),
                    building.level,
                    if mass.is_none() {
                        "its kind declares no mass"
                    } else {
                        "no material claim and no declared body part"
                    }
                ));
                continue;
            };
            ledger.record(&building.mass_account(family), mass);
            standing_g += mass;
        }

        material_ledger::MassAudit { ledger, extracted_g, standing_g, held_g, defects }
    }

    /// Plan where a structure's material would come from, **without touching the ground**.
    ///
    /// Round 11's Q73 answered *nearest matching family*: tiles are searched outward from the
    /// site for a deposit whose declared family is the structure's own claimed family — a ceramic
    /// home takes stone, sand or clay and never iron ore. That is what makes "built out of
    /// something" checkable rather than merely claimed; without it, a brick house raised on an
    /// iron seam would balance in tonnes and be nonsense in substance.
    ///
    /// The plan is **atomic on purpose**: it is computed in full and only then drawn, so a build
    /// that could not be covered never half-mines the map. A caller that finds three tiles of clay
    /// and is one short would otherwise leave three holes and nothing standing on them, which is
    /// exactly the half-finished thing a shortfall exists to refuse.
    ///
    /// Ties are broken by tile index, because two tiles the same distance away must be chosen the
    /// same way on every platform and in every replay — a nearest-neighbour search with an
    /// arbitrary order is the quietest way to lose determinism there is.
    pub fn plan_material(
        &self,
        site: u32,
        family: &str,
        need_g: i64,
    ) -> Result<Vec<MaterialLineage>, MaterialShortfall> {
        let shortfall = |found_g: i64| MaterialShortfall::of(site, family, need_g, found_g);
        if need_g <= 0 {
            return Ok(Vec::new());
        }

        let (sx, sy) = self.coords(site);
        let mut candidates: Vec<(i64, u32, &'static str)> = Vec::new();
        for index in 0..self.tiles.len() as u32 {
            let Some(deposit) = self.deposit_at(index) else {
                continue;
            };
            let substance = material_geology::kind_substance(deposit.kind);
            if material_geology::kind_family(deposit.kind) != family {
                continue;
            }
            let (x, y) = (index % self.width, index / self.width);
            let (dx, dy) = (
                x as i64 - sx as i64,
                y as i64 - sy as i64,
            );
            candidates.push((dx * dx + dy * dy, index, substance));
        }
        // Nearest first, and a stable tie-break so the plan is a function of the world alone.
        candidates.sort_by_key(|(distance, index, _)| (*distance, *index));

        let mut plan = Vec::new();
        let mut covered = 0;
        for (_, index, substance) in candidates {
            if covered >= need_g {
                break;
            }
            let available = self.deposit_at(index).map(|d| d.mass_g).unwrap_or(0);
            let take = available.min(need_g - covered);
            if take <= 0 {
                continue;
            }
            covered += take;
            plan.push(MaterialLineage {
                tile: index,
                substance: substance.to_string(),
                grams: take,
            });
        }
        if covered < need_g {
            return Err(shortfall(covered));
        }
        Ok(plan)
    }

    /// Draw a plan's mass out of the ground, returning what was actually taken.
    ///
    /// It takes the plan rather than recomputing it, so the mass a structure claims is the mass
    /// the plan promised: a second search could differ from the first, and a lineage that does not
    /// describe the hole it came from is worse than no lineage at all.
    pub fn draw_material(&mut self, plan: &[MaterialLineage]) -> Vec<MaterialLineage> {
        plan.iter()
            .map(|line| MaterialLineage {
                tile: line.tile,
                substance: line.substance.clone(),
                grams: self.extract(line.tile, line.grams),
            })
            .collect()
    }

    /// Build a structure **out of material the world actually gives up**, or refuse and say why.
    ///
    /// This is the shape round 11's Q72 asked for: nothing is placed that the world cannot pay
    /// for, and the payment is recorded per tile. The refusal is returned rather than swallowed so
    /// the caller can file a case — Q74's answer — because a city that stopped growing with no
    /// reason on the record is the complaint this campaign opened with.
    pub fn build_with_material(
        &mut self,
        tile: u32,
        kind: BuildingKind,
    ) -> Result<u32, MaterialShortfall> {
        // An undeclared kind is a defect, not a shortfall: the world cannot be asked to supply
        // material for a structure whose own mass nobody declared, and panicking *names* it in the
        // same spirit as an undeclared surface family, rather than refusing in a way that reads as
        // "the ground was empty".
        let mass = material_ledger::structure_mass_g(kind.name(), 1)
            .unwrap_or_else(|| panic!("{} declares no mass, so it cannot be built", kind.name()));
        // The family it takes is the family it will **claim**, read from the same reading that
        // fills `material_as_built` — one source, so a structure cannot be built out of one
        // material and claim another, which is the substance-level version of the same defect the
        // ledger exists to catch in grams.
        let claim = MaterialClaim::of_kind(kind).unwrap_or_else(|| {
            panic!("{} has no declared material claim, so its material is unknown", kind.name())
        });
        let family = claim.family;
        let plan = self.plan_material(tile, &family, mass)?;
        let drawn = self.draw_material(&plan);
        let built = self
            .place_building(tile, kind)
            .ok_or_else(|| MaterialShortfall::of(tile, family.clone(), mass, 0))?;
        if let Some(index) = self.tiles[tile as usize].building {
            self.buildings[index as usize].material_from = drawn;
        }
        Ok(built)
    }

    /// Open a build task on a site, **or refuse and say why** (round 11's Q72).
    ///
    /// The material is planned here, before any citizen is asked to work, and the plan travels
    /// with the task: a task that exists can be paid for, and one that cannot is not opened at
    /// all. That is the whole difference between this and the `grow()` it replaces — `grow()`
    /// decided that a building should exist and made one; this decides that a building should
    /// exist and then asks the world whether it can supply it, which is a question the world is
    /// allowed to answer no.
    pub fn post_build_task(
        &mut self,
        site: u32,
        kind: BuildingKind,
    ) -> Result<u32, MaterialShortfall> {
        let mass = material_ledger::structure_mass_g(kind.name(), 1)
            .unwrap_or_else(|| panic!("{} declares no mass, so it cannot be built", kind.name()));
        let claim = MaterialClaim::of_kind(kind).unwrap_or_else(|| {
            panic!("{} has no declared material claim, so its material is unknown", kind.name())
        });
        let plan = self.plan_material(site, &claim.family, mass)?;
        let id = self.next_task_id;
        self.next_task_id += 1;
        self.tasks.push(sim_task::Task {
            id,
            verb: sim_task::Verb::Build,
            kind,
            site,
            family: claim.family,
            requires_g: mass,
            material: plan,
            // The kind's own declared build cost is the work, so how long a structure takes is
            // the same declared number the player's build already pays.
            work_remaining: kind.build_cost(),
            // A build draws from the ground under its own site: no process, no patch, and the
            // stage that matters to a gather is a stage a build is never in.
            process: String::new(),
            substance: String::new(),
            fetch_from: None,
            stage: sim_task::Stage::Working,
            claimed_by: None,
            opened_tick: self.clock.tick,
        });
        Ok(id)
    }

    /// Where a substance is held by the world, **nearest to `site`** — a patch the ground grows,
    /// or a seam it holds — among the tiles a worker can actually stand beside.
    ///
    /// Round 11's Q73 chose nearest-matching-family for a build's material and this is the same
    /// rule one link up: nearest first, ties by tile index so the choice is a function of the
    /// world alone, and **only tiles within work reach of a road**, because a patch nobody can
    /// stand at is a patch nobody can gather (Q102 takes these by hand, and a hand has to get
    /// there).
    pub fn plan_source(&self, site: u32, substance: &str) -> Option<u32> {
        let (sx, sy) = self.coords(site);
        let mut best: Option<((i64, u32), u32)> = None;
        for index in 0..self.tiles.len() as u32 {
            // **Reach first, substance second**, and the order is the cheap half of the search: a
            // tile with no road within reach cannot be gathered from whatever it holds, and this
            // costs a ring of road-graph lookups instead of deriving a patch the search will
            // throw away.
            if self
                .roads
                .nearest_node_within(self.width, self.height, index, WORK_REACH)
                .is_none()
            {
                continue;
            }
            let holds = self
                .surface_at(index)
                .is_some_and(|standing| standing.substance == substance)
                || self
                    .deposit_at(index)
                    .is_some_and(|deposit| material_geology::kind_substance(deposit.kind) == substance);
            if !holds {
                continue;
            }
            let (x, y) = self.coords(index);
            let distance = (x as i64 - sx as i64).pow(2) + (y as i64 - sy as i64).pow(2);
            if best.map(|(key, _)| (distance, index) < key).unwrap_or(true) {
                best = Some(((distance, index), index));
            }
        }
        best.map(|(_, index)| index)
    }

    /// Whether a worker standing on `standing` is at work on `target`.
    ///
    /// A declared reach rather than adjacency, and it is the same number the router uses: a
    /// worker walks the road to the end of the street and works the tile it reaches. Before this
    /// existed the check was one tile, while the planter, the router and `has_road_access` all
    /// worked within two or three — so a site two tiles off a road could be posted and never
    /// worked, which is a stall nobody can see.
    pub fn within_reach(&self, standing: u32, target: u32) -> bool {
        within_reach(self.width, standing, target)
    }

    /// Whether the site is holding what a make needs for one run of its process.
    ///
    /// The rule round 13's Q77 decision needs: **a process consumes material that is already
    /// held**, so a process nobody has supplied is not work — it is a wish. Checked at claim time
    /// (so nobody walks to a step they cannot do) and again at completion (because a holding can
    /// be emptied between the two).
    pub fn inputs_present(&self, process: &material_generated::Process, site: u32) -> bool {
        process.inputs.iter().all(|(name, amount)| {
            material_schema::quantity_grams(name, *amount).is_some_and(|grams| {
                self.holdings.of(&material_ledger::site_account(site, name)) >= grams
            })
        })
    }

    /// Whether a task can be started at all — Q74's rule applied to *beginning* work rather than
    /// to posting it: a build and a gather are always startable (the world plans their material
    /// before they are posted), and a make is startable when its site holds its inputs.
    pub fn task_can_start(&self, task: &Task) -> bool {
        if task.verb != sim_task::Verb::Make {
            return true;
        }
        material_schema::process(&task.process)
            .is_some_and(|process| self.inputs_present(process, task.site))
    }

    /// The site a city works at by hand when nobody owns a workshop: the first ground tile with
    /// road access and nothing on it, in tile order.
    ///
    /// Deterministic on purpose, and deliberately dull: *where the works is* has to be a function
    /// of the world, and the first road-reachable tile is one. A player-proposed site (Q87/Q98's
    /// tender) is the later answer, and this is the placeholder the record asks to be *named*
    /// rather than left to look like the answer.
    pub fn works_site(&self) -> Option<u32> {
        (0..self.tiles.len() as u32).find(|index| {
            let tile = self.tiles[*index as usize];
            tile.terrain == Terrain::Ground
                && !tile.road
                && tile.building.is_none()
                && tile.extracted_g == 0
                && tile.harvested_g == 0
                && self.has_road_access(*index)
        })
    }

    /// Post the work that makes **one hatchet** — the stone rung, end to end (Q66's first rung,
    /// Q102's first-tier kinds, Q7's own example of a tool fashioned from natural materials).
    ///
    /// The plan comes from `materials::chain`, so what has to come out of the ground and in what
    /// order is the declaration's answer rather than this function's opinion. Two kinds of task
    /// fall out of it: a **gather** per leaf, addressed to the patch the world holds it in, and a
    /// **make** per step, addressed to the site. Nothing here schedules them: a make is not
    /// claimable until its inputs are held, so the order the plan states is the order the world
    /// enforces.
    pub fn post_rung_tasks(&mut self, site: u32) -> Result<Vec<u32>, MaterialShortfall> {
        // One hatchet, in grams, from the table's own declared unit mass: a counted good without
        // one cannot enter a ledger, so this is a refusal rather than a default.
        let grams = material_schema::quantity_grams(RUNG_GOOD, material_generated::Rational { num: 1, den: 1 })
            .unwrap_or_else(|| {
                panic!("`{RUNG_GOOD}` declares no unit mass, so one of it is not an amount of mass")
            });
        let plan = match material_chain::plan(RUNG_GOOD, grams) {
            Ok(plan) => plan,
            // A rung that cannot be planned is a defect in the declaration rather than a shortage
            // in the world, and the refusal says which — named here so it reaches a case instead
            // of a silent nothing.
            Err(refusal) => {
                return Err(MaterialShortfall::of(site, RUNG_GOOD, grams, 0)
                    .noting(refusal.describe()))
            }
        };
        let mut posted = Vec::new();

        // A gather per leaf: one trip to one patch, carrying what it took to the site. `kind` is
        // the build verb's field and is left at its default here — a gather raises nothing, and a
        // placeholder that read as a structure would be worse than one that is documented as
        // ignored.

        for leaf in &plan.leaves {
            let Some(process) = material_schema::process(leaf.process.unwrap_or("")) else {
                return Err(MaterialShortfall::of(site, leaf.substance, leaf.grams, 0)
                    .noting(format!("the reduction names a gather process `{}` that the tables do not declare", leaf.process.unwrap_or(""))));
            };
            let Some(patch) = self.plan_source(site, leaf.substance) else {
                // A leaf the world cannot supply is a refusal with a substance on it (Q74), not a
                // task somebody fails at later — and a patch with no road within reach is that
                // same refusal, which is why the search is by *reachable* source.
                return Err(MaterialShortfall::of(site, leaf.substance, leaf.grams, 0)
                    .noting(format!(
                        "no tile within {WORK_REACH} tiles of a road holds `{}`",
                        leaf.substance
                    )));
            };
            // Whole runs of the declared gather, and the surplus that leaves over is real mass
            // the city keeps — the same arithmetic `chain` reports (one armful is 3000 g).
            let per_run = gather_per_run(process, leaf.substance).unwrap_or(0);
            let runs = if per_run > 0 { material_chain::runs_for(leaf.grams, per_run) } else { 1 };
            let take_g = if per_run > 0 { runs * per_run } else { leaf.grams };
            let id = self.next_task_id;
            self.next_task_id += 1;
            self.tasks.push(Task {
                id,
                verb: sim_task::Verb::Gather,
                kind: BuildingKind::Home,
                site,
                family: material_schema::substance(leaf.substance)
                    .map(|entry| entry.family.to_string())
                    .unwrap_or_default(),
                requires_g: take_g,
                material: Vec::new(),
                process: process.name.to_string(),
                substance: leaf.substance.to_string(),
                fetch_from: Some(patch),
                stage: sim_task::Stage::Fetching,
                work_remaining: sim_task::work_of_process(process) * runs,
                claimed_by: None,
                opened_tick: self.clock.tick,
            });
            posted.push(id);
        }

        // A make per step, dependencies already ordered by the reduction. A **gather** is a step
        // of the plan and not a make: it is already posted above as work in the world, and posting
        // it here as well would be a process that produces out of nothing — the one thing the
        // tables refuse.
        for step in &plan.steps {
            let Some(process) = material_schema::process(step.process) else {
                continue;
            };
            if process.inputs.is_empty() {
                continue;
            }
            let id = self.next_task_id;
            self.next_task_id += 1;
            self.tasks.push(Task {
                id,
                verb: sim_task::Verb::Make,
                kind: BuildingKind::Home,
                site,
                family: String::new(),
                requires_g: step.output_g,
                material: Vec::new(),
                process: step.process.to_string(),
                substance: String::new(),
                fetch_from: None,
                stage: sim_task::Stage::Working,
                work_remaining: sim_task::work_of_process(process) * step.runs,
                claimed_by: None,
                opened_tick: self.clock.tick,
            });
            posted.push(id);
        }
        Ok(posted)
    }

    /// The city's first want is a tool (Q7, Q99, Q102): post the rung's work if the city has none
    /// of it and nothing else is open.
    ///
    /// This is the owner-of-last-resort case (Q36) in its purest form — no citizen owns a works, so
    /// the city does — and it is what stands where a founding endowment would be (Q5 asked for hands
    /// instead). Returns whether work was posted.
    pub fn post_works_tasks(&mut self) -> bool {
        // **Where a city's first tool is wanted is a daily decision, not a per-tick one.** The
        // search reads the whole map — reach, patches, seams — so asking it every eight ticks would
        // be scanning the world two and a half times a second for an answer that changes when
        // somebody lays a road. One sim-day is the declared cadence (Q14's timescale, applied to a
        // decision rather than to a process), and the first tick is allowed so a city that already
        // can work starts immediately.
        if self.clock.tick != 0 && !self.clock.tick.is_multiple_of(TICKS_PER_DAY) {
            return false;
        }
        if !self.tasks.is_empty() {
            // One works at a time. A queue that grows faster than it drains is a queue nobody can
            // read, which is the same rule the case engine's dedupe follows.
            return false;
        }
        if self.holds(RUNG_GOOD) {
            return false;
        }
        let Some(site) = self.works_site() else {
            // No ground within reach of a road to work on. That is a refusal like any other, and
            // it is filed rather than passed over: a city that does nothing with no reason on the
            // record is the complaint this campaign opened with (`docs/GRILLING-C9.md`, *What
            // prompted it*).
            self.record_starved(&MaterialShortfall::of(0, "a works site", 0, 0).noting(format!(
                "every tile within {WORK_REACH} tiles of a road is paved, built on or already \
                 worked, so the city has nowhere to put its first rung"
            )));
            return false;
        };
        match self.post_rung_tasks(site) {
            Ok(_) => true,
            Err(shortfall) => {
                // A rung the world cannot supply is a case with a number: which substance, how
                // much, and where it was wanted.
                self.record_starved(&shortfall);
                false
            }
        }
    }

    /// Whether any holding anywhere holds a gram of `substance`.
    pub fn holds(&self, substance: &str) -> bool {
        self.holdings
            .entries()
            .any(|(account, grams)| grams > 0 && account.ends_with(&format!(":{substance}")))
    }

    /// How much of a substance is held in total, in grams, wherever it is.
    pub fn held_g(&self, substance: &str) -> i64 {
        self.holdings
            .entries()
            .filter(|(account, _)| account.ends_with(&format!(":{substance}")))
            .map(|(_, grams)| grams)
            .sum()
    }

    /// Has this site already got open work on it? One task per site, so two citizens cannot raise
    /// two buildings on one tile and the demand does not queue itself into a crowd.
    pub fn open_task_at(&self, site: u32) -> Option<&sim_task::Task> {
        self.tasks.iter().find(|task| task.site == site)
    }

    /// Where the city is **short of material**, deduped by district and family, in the shape the
    /// case engine reads.
    ///
    /// Round 11's Q74 answered that a refusal files a case rather than passing quietly, so this is
    /// the record the cases are sampled from. It is aggregated, not appended: four hundred blocked
    /// builds wanting clay in one district is **one** reading with a count, exactly as a case
    /// already reads its count.
    pub fn starved(&self) -> &[Starved] {
        &self.starved
    }

    /// Note a refusal, folding it into an existing reading for the same district and family.
    fn record_starved(&mut self, shortfall: &MaterialShortfall) {
        // A refusal names either a **substance** (a leaf the world cannot supply: `timber`) or a
        // **family** (a structure's material: `ceramic`). Keeping the two apart is what makes
        // "no timber" readable as timber rather than as the family it presents as — and what
        // keeps two refusals about different substances from folding into one reading.
        let named = material_schema::substance(&shortfall.family);
        let (family, substance) = match named {
            Some(entry) => (entry.family.to_string(), shortfall.family.clone()),
            None => (shortfall.family.clone(), String::new()),
        };
        let district = crate::gov::district_of(self.width, shortfall.site);
        if let Some(existing) = self.starved.iter_mut().find(|s| {
            s.district == district && s.family == family && s.substance == substance
        }) {
            existing.count += 1;
            existing.wanted_g += shortfall.wanted_g;
            existing.found_g += shortfall.found_g;
            return;
        }
        self.starved.push(Starved {
            district,
            family,
            substance,
            note: shortfall.note.clone(),
            wanted_g: shortfall.wanted_g,
            found_g: shortfall.found_g,
            count: 1,
        });
    }

    /// Post build work for zoned ground that is powered, connected and wanted.
    ///
    /// This is what stands where `grow()` stood. It makes **no building** — it makes *work*, and
    /// the building is the consequence of a citizen doing it with material the ground gave up.
    /// Deleting `grow()` is round 6's Q45 answer and round 11's Q72 confirmation: the sim no
    /// longer decides that a structure exists, it decides that the city wants one built.
    pub fn post_demand_tasks(&mut self) {
        if self.stats.brownout {
            // No power, no growth. Posting work the city cannot light would file its own case.
            return;
        }
        let mut posted = 0;
        for tile in 0..self.tiles.len() as u32 {
            if posted >= 4 {
                break;
            }
            let t = self.tiles[tile as usize];
            if t.zone == Zone::None || t.building.is_some() || t.road || !t.powered {
                continue;
            }
            if self.open_task_at(tile).is_some() || !self.has_road_access(tile) {
                continue;
            }
            if self.demand.for_zone(t.zone) < 0.20 {
                continue;
            }
            let kind = match t.zone {
                Zone::Residential => BuildingKind::Home,
                Zone::Commercial => BuildingKind::Shop,
                Zone::Industrial => BuildingKind::Factory,
                Zone::None => continue,
            };
            match self.post_build_task(tile, kind) {
                Ok(_) => {
                    posted += 1;
                    // Spending the demand is what stops one busy tick from zoning an entire map.
                    match t.zone {
                        Zone::Residential => {
                            self.demand.residential = (self.demand.residential - 0.12).max(0.0)
                        }
                        Zone::Commercial => {
                            self.demand.commercial = (self.demand.commercial - 0.12).max(0.0)
                        }
                        Zone::Industrial => {
                            self.demand.industrial = (self.demand.industrial - 0.12).max(0.0)
                        }
                        Zone::None => {}
                    }
                }
                // A blocked task is a refusal with a number, not a silence. The work is not
                // posted, so the queue never fills with jobs nobody can do.
                Err(shortfall) => self.record_starved(&shortfall),
            }
        }
    }

    /// Idle hands take the nearest open work, and walk to it.
    ///
    /// Round 4's Q29(b) answered that citizens decide for themselves; round 5's Q36 said the
    /// government is the **owner of last resort** for what nobody owns — a road, a service, a
    /// build nobody has commissioned. A build task is exactly that case, so the city's own
    /// unemployed take it. Utility-scored choice with declared weights (Q53) arrives with the
    /// citizen model that can score anything; today the choice is nearest-first, and this comment
    /// is the record that it is a placeholder rather than the answer.
    pub fn claim_tasks(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        for index in 0..self.citizens.len() {
            let (ready, state, free, home) = {
                let c = &self.citizens[index];
                (c.ready, c.state, c.work.is_none() && c.task.is_none(), c.home)
            };
            // An idle resident is one at home or between jobs: `Unemployed` is documented in the
            // citizen model as "a home but no work", which is exactly the labour a city has free
            // to build itself with, and requiring `AtHome` alone would have excluded every one.
            if !ready
                || !free
                || !matches!(state, CitizenState::AtHome | CitizenState::Unemployed)
            {
                continue;
            }
            let Some(home) = home else { continue };
            let home_tile = self.buildings[home as usize].tile;
            let (hx, hy) = (home_tile % self.width, home_tile / self.width);
            let mut best: Option<((i64, u32), u32)> = None;
            for task in self.tasks.iter() {
                if task.is_claimed() {
                    continue;
                }
                // A **make whose site is not holding its inputs is not work yet**: nobody walks
                // to a step they cannot take. This is Q74's rule applied to starting rather than
                // to posting, and it is what makes the plan's order emerge — a later step becomes
                // claimable exactly when an earlier one has delivered (round 13's Q77).
                if !self.task_can_start(task) {
                    continue;
                }
                let target = task.destination();
                let (tx, ty) = (target % self.width, target / self.width);
                let dx = tx as i64 - hx as i64;
                let dy = ty as i64 - hy as i64;
                let distance = dx * dx + dy * dy;
                if best.map(|(key, _)| (distance, task.id) < key).unwrap_or(true) {
                    best = Some(((distance, task.id), task.id));
                }
            }
            let Some((_, task_id)) = best else { break };
            if let Some(task) = self.tasks.iter_mut().find(|task| task.id == task_id) {
                task.claimed_by = Some(self.citizens[index].id);
            }
            let citizen = &mut self.citizens[index];
            citizen.task = Some(task_id);
            // At home, so the planner's `AtHome` case routes them to the site — never straight
            // there, because a citizen that teleports is not a citizen. Parked with no plan, so
            // the planner picks them up on its next pass.
            citizen.state = CitizenState::AtHome;
            citizen.path.clear();
            citizen.path_cursor = 0;
            citizen.step_work = 0.0;
        }
    }

    /// The work a claimed task gets from the citizen standing on it, and what happens when it is
    /// finished.
    ///
    /// The clock is not read here on purpose, and the parameter-free shape is what keeps this a
    /// pure function of the world's own state: how much work a task has left is a number, not a
    /// number of seconds.
    ///
    /// The counter only moves while somebody is actually there — that is the "atomic and
    /// verifiable" round 4's Q19 asked for, expressed as a rule a test can check rather than as an
    /// intention. Completion draws the plan and raises the structure; a plan that no longer
    /// covers the mass (because someone else mined it) is a refusal with a case, not a building.
    pub fn work_tasks(&mut self) {
        let mut finished: Vec<u32> = Vec::new();
        let mut stalled: Vec<(u32, MaterialShortfall)> = Vec::new();
        // Refusals that do **not** stop the work: a gather that came back with less than the plan
        // asked for still delivered what the world had, and the shortage is a reading rather than
        // a failure. Recorded separately from `stalled` because the two do different things to
        // the task.
        let mut refusals: Vec<MaterialShortfall> = Vec::new();

        for task in self.tasks.iter_mut() {
            let Some(worker) = task.claimed_by else { continue };
            let Some(citizen) = self.citizens.iter().find(|c| c.id == worker) else {
                // The worker is gone. The work goes back to the queue rather than being lost.
                task.claimed_by = None;
                continue;
            };
            if citizen.state != CitizenState::AtWork || citizen.task != Some(task.id) {
                continue;
            }
            let standing = citizen.current_tile().unwrap_or(task.site);
            if !within_reach(self.width, standing, task.destination()) {
                // Standing somewhere else is not working on it — and a gather's **patch** is the
                // somewhere else that matters: the work is done at the patch, not at the site.
                continue;
            }
            task.work_remaining -= 1;
            if task.is_done() {
                finished.push(task.id);
            }
        }

        for id in finished {
            let Some(position) = self.tasks.iter().position(|task| task.id == id) else { continue };
            let task = self.tasks[position].clone();
            match (task.verb, task.stage) {
                (sim_task::Verb::Build, _) => {
                    let drawn = self.draw_material(&task.material);
                    let drawn_g: i64 = drawn.iter().map(|line| line.grams).sum();
                    let built = if drawn_g >= task.requires_g {
                        self.place_building(task.site, task.kind)
                    } else {
                        None
                    };
                    match built {
                        Some(_) => {
                            if let Some(index) = self.tiles[task.site as usize].building {
                                self.buildings[index as usize].material_from = drawn;
                            }
                        }
                        None => {
                            // Either the ground no longer covers the plan or the site stopped
                            // being buildable. Both are refusals with a number, and the case
                            // engine reads them.
                            stalled.push((id, MaterialShortfall::of(
                                task.site,
                                task.family.clone(),
                                task.requires_g,
                                drawn_g,
                            )));
                            continue;
                        }
                    }
                    self.release(id, CitizenState::ToHome);
                    self.tasks.remove(position);
                }
                // Taking from the world, by hand, into the carrier's arms (Q77's `carried:`).
                (sim_task::Verb::Gather, sim_task::Stage::Fetching) => {
                    let patch = task.fetch_from.unwrap_or(task.site);
                    let taken = self.take_from_world(&task);
                    if taken <= 0 {
                        // A patch that is bare — or a seam worked out — is not a failure, it is
                        // Q92's *refused until something changes*: the case names it and the work
                        // waits, because a patch grows back (Q108) and a mine does not.
                        stalled.push((id, MaterialShortfall::of(patch, task.substance.clone(), task.requires_g, 0)));
                        continue;
                    }
                    if taken < task.requires_g {
                        // A thin edge of a patch gives less than an armful. The take is real, so
                        // it is carried and delivered — and the shortage is a reading, because a
                        // gather that brought back what was there is not a failure.
                        refusals.push(MaterialShortfall::of(
                            patch,
                            task.substance.clone(),
                            task.requires_g,
                            taken,
                        ));
                    }
                    let worker = task.claimed_by.unwrap_or(0);
                    let account = material_ledger::carried_account(worker, &task.substance);
                    self.credit_holding(&account, taken);
                    if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
                        task.stage = sim_task::Stage::Delivering;
                    }
                    // Sent to the site with what they took: cleared so the planner routes the
                    // **next leg** rather than finishing the last one.
                    self.replan_for_task(id);
                }
                // Carrying it to the site, where it stays as a holding (Q77's `site:`).
                (sim_task::Verb::Gather, sim_task::Stage::Delivering) => {
                    let worker = task.claimed_by.unwrap_or(0);
                    let from = material_ledger::carried_account(worker, &task.substance);
                    let to = material_ledger::site_account(task.site, &task.substance);
                    // Whatever is actually in their arms, rather than what the plan asked for:
                    // the take was bounded by the patch, and the plan's number is a want.
                    let carried = self.holding_of(&from);
                    if carried <= 0 {
                        stalled.push((id, MaterialShortfall::of(task.site, task.substance.clone(), task.requires_g, 0)));
                        continue;
                    }
                    if let Some(defect) = self.move_holding(&from, &to, carried) {
                        // The worker is not carrying what the task says they are: a defect worth
                        // reading rather than a negative holding nobody notices.
                        stalled.push((
                            id,
                            MaterialShortfall::of(task.site, task.substance.clone(), task.requires_g, 0)
                                .noting(defect),
                        ));
                        continue;
                    }
                    self.release(id, CitizenState::ToHome);
                    self.tasks.remove(position);
                }
                // A gather is always fetching or carrying. A `Working` gather would be a task the
                // world posted wrong, and saying so is cheaper than a silent no-op.
                (sim_task::Verb::Gather, sim_task::Stage::Working) => {
                    stalled.push((
                        id,
                        MaterialShortfall::of(task.site, task.substance.clone(), task.requires_g, 0)
                            .noting("a gather in the working stage has no patch to take from"),
                    ));
                }
                // Running a declared process on material **already held at the site**.
                (sim_task::Verb::Make, _) => {
                    let Some(process) = material_schema::process(&task.process) else {
                        stalled.push((
                            id,
                            MaterialShortfall::of(task.site, task.process.clone(), task.requires_g, 0)
                                .noting(format!("`{}` is not a declared process", task.process)),
                        ));
                        continue;
                    };
                    if !self.inputs_present(process, task.site) {
                        // The inputs went somewhere between the claim and the finish. Q92: the
                        // task stalls with a case naming what is missing, rather than finishing
                        // into a holding that cannot pay for it.
                        let short = process
                            .inputs
                            .first()
                            .map(|(name, _)| (*name).to_string())
                            .unwrap_or_else(|| task.process.clone());
                        let want = process
                            .inputs
                            .first()
                            .and_then(|(name, amount)| material_schema::quantity_grams(name, *amount))
                            .unwrap_or(0);
                        let have = process
                            .inputs
                            .first()
                            .map(|(name, _)| self.holding_of(&material_ledger::site_account(task.site, name)))
                            .unwrap_or(0);
                        stalled.push((
                            id,
                            MaterialShortfall::of(task.site, short, want, have)
                                .noting("the site is no longer holding what the process takes in"),
                        ));
                        continue;
                    }
                    if let Some(defect) = self.run_process(&task) {
                        stalled.push((
                            id,
                            MaterialShortfall::of(task.site, task.process.clone(), task.requires_g, 0)
                                .noting(defect),
                        ));
                        continue;
                    }
                    self.release(id, CitizenState::ToHome);
                    self.tasks.remove(position);
                }
            }
        }

        // A stall is work that is still real: the task goes back to the queue with its work
        // intact, and the case stays open until the world changes (Q92 — *refused until something
        // changes*).
        for (id, shortfall) in stalled {
            if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
                task.claimed_by = None;
            }
            self.release(id, CitizenState::AtHome);
            self.record_starved(&shortfall);
        }
        for shortfall in refusals {
            self.record_starved(&shortfall);
        }
    }

    /// Take a gather's substance out of the world at the tile it stands at.
    ///
    /// Two routes, one rule: a **patch** the ground grows is harvested (Q102, by hand, no mine),
    /// and a **seam** is extracted — which is the same take `plan_material` already performs, done
    /// at the tile the worker is standing by rather than at the site.
    fn take_from_world(&mut self, task: &Task) -> i64 {
        let Some(tile) = task.fetch_from else { return 0 };
        let patches = self
            .surface_at(tile)
            .is_some_and(|standing| standing.substance == task.substance);
        if patches {
            return self.harvest(tile, task.requires_g);
        }
        let seam = self
            .deposit_at(tile)
            .is_some_and(|deposit| material_geology::kind_substance(deposit.kind) == task.substance);
        if seam {
            return self.extract(tile, task.requires_g);
        }
        0
    }

    /// Run a task's declared process at its site: consume what is held there, and produce what
    /// the row says, **both into the same holding**.
    ///
    /// The balance is re-checked here rather than assumed, because this is the function that
    /// would turn an unbalanced row into a city that creates matter — the same reasoning the
    /// emitting gate and `materials::schema` both follow. `Some(defect)` means nothing moved.
    fn run_process(&mut self, task: &Task) -> Option<String> {
        let process = material_schema::process(&task.process)?;
        if process.inputs.is_empty() {
            // A process that takes nothing in is a **gather**: the world hands it over, and the
            // only place that may happen is a patch or a seam, through a gather task. Running one
            // at a site would be `grow()` wearing a process name.
            return Some(format!(
                "`{}` takes nothing in, so it is a gather: it is taken from the world rather than \
                 run at a site",
                process.name
            ));
        }
        let (Some(into), Some(out)) = material_schema::process_totals(process) else {
            return Some(format!("`{}` has a quantity that does not reach grams", process.name));
        };
        if into != out {
            return Some(format!(
                "`{}` does not balance: {into} g in and {out} g out, so running it would make or \
                 lose {} g",
                process.name,
                (into - out).abs()
            ));
        }
        // One run of the row, exactly as declared. A make task is one run: whole runs are the rule
        // (`chain` states it for the plan), and a task that needed two would be two tasks.
        for (name, amount) in process.inputs {
            let Some(grams) = material_schema::quantity_grams(name, *amount) else {
                return Some(format!("`{name}` in `{}` does not reach grams", process.name));
            };
            let account = material_ledger::site_account(task.site, name);
            self.holdings.record(&account, -grams);
        }
        for (name, amount) in process.outputs {
            let Some(grams) = material_schema::quantity_grams(name, *amount) else {
                return Some(format!("`{name}` in `{}` does not reach grams", process.name));
            };
            let account = material_ledger::site_account(task.site, name);
            self.holdings.record(&account, grams);
        }
        None
    }

    /// A worker lets go of a task: their claim goes, and they are sent on from where they stand.
    fn release(&mut self, id: u32, state: CitizenState) {
        for citizen in self.citizens.iter_mut() {
            if citizen.task == Some(id) {
                citizen.task = None;
                citizen.state = state;
                // Parked with no plan, so the planner picks them up on its next pass — a released
                // worker walks home from wherever the work was rather than standing there.
                citizen.path.clear();
                citizen.path_cursor = 0;
                citizen.step_work = 0.0;
            }
        }
    }

    /// Send a worker on to the next leg of their task.
    ///
    /// The stage moved (a gather finished fetching), so the destination changed: clearing the path
    /// is what lets the planner route the **new** leg instead of finishing the old one, and the
    /// state stays `AtWork` so the planner knows to route rather than to retire them.
    fn replan_for_task(&mut self, id: u32) {
        for citizen in self.citizens.iter_mut() {
            if citizen.task == Some(id) {
                citizen.state = CitizenState::AtWork;
                citizen.path.clear();
                citizen.path_cursor = 0;
                citizen.step_work = 0.0;
            }
        }
    }

    /// Demolish whatever stands on a tile. The building is **retired**, not
    /// deleted, and the reason is recorded.
    pub fn demolish(&mut self, tile: u32, reason: RetirementReason) -> Option<u32> {
        let t = self.tiles[tile as usize];
        if let Some(building) = t.building {
            let b = &mut self.buildings[building as usize];
            b.retired = Some(reason);
            b.retired_tick = Some(self.clock.tick);
            let id = b.id;
            // Residents of a retired home leave rather than vanish silently.
            for citizen in self.citizens.iter_mut() {
                if citizen.home == Some(building) || citizen.work == Some(building) {
                    citizen.state = CitizenState::Unemployed;
                    citizen.path.clear();
                }
            }
            self.tiles[tile as usize].building = None;
            self.recount();
            return Some(id);
        }
        if t.road {
            self.tiles[tile as usize].road = false;
            self.rebuild_derived();
            return None;
        }
        if t.zone != Zone::None {
            self.tiles[tile as usize].zone = Zone::None;
            return None;
        }
        None
    }

    /// Place a structure directly (power plants are placed, not grown).
    pub fn place_building(&mut self, tile: u32, kind: BuildingKind) -> Option<u32> {
        let t = self.tiles[tile as usize];
        if t.terrain != Terrain::Ground || t.road || t.building.is_some() {
            return None;
        }
        if kind == BuildingKind::PowerPlant && !self.has_road_access(tile) {
            return None;
        }
        let id = self.next_building_id;
        self.next_building_id += 1;
        let index = self.buildings.len() as u32;
        self.buildings.push(Building {
            id,
            kind,
            tile,
            level: 1,
            built_tick: self.clock.tick,
            ready_tick: self.clock.tick + BUILD_TICKS,
            retired: None,
            retired_tick: None,
            powered: false,
            occupants: 0,
            repair_filed: false,
            // Every structure carries its claim, whoever placed it: the world state is
            // what `verify.exe` re-reads, and a structure without a claim would be
            // unverifiable rather than merely unrecorded. The `MAT-*` *ticket* is filed
            // where placement is a recorded act (the player's build), which is the same
            // distinction the record already makes between a claim and growth.
            material_as_built: MaterialClaim::of_kind(kind),
            condition: condition_as_built(),
            // Placement is not building: this primitive makes no material claim, and
            // `build_with_material` is what fills one in.
            material_from: Vec::new(),
        });
        self.tiles[tile as usize].building = Some(index);
        self.tiles[tile as usize].zone = Zone::None;
        self.recount();
        Some(id)
    }

    pub fn set_tax(&mut self, zone: Zone, rate: f32) -> f32 {
        let clamped = rate.clamp(0.0, 0.20);
        match zone {
            Zone::Residential => self.economy.tax_residential = clamped,
            Zone::Commercial => self.economy.tax_commercial = clamped,
            Zone::Industrial => self.economy.tax_industrial = clamped,
            Zone::None => {}
        }
        clamped
    }

    // ---------------------------------------------------------------------
    // The tick.
    // ---------------------------------------------------------------------

    pub fn tick(&mut self) {
        self.clock.tick += 1;

        if self.clock.tick.is_multiple_of(4) {
            self.recompute_power();
        }
        if self.clock.tick.is_multiple_of(8) {
            self.recompute_demand();
            self.post_demand_tasks();
            // The city's first want is a tool, and nobody owns a works: the owner of last resort
            // posts it (Q36), which is what stands where `grow()`'s endowment would have been.
            self.post_works_tasks();
            self.claim_tasks();
            self.work_tasks();
        }
        if self.clock.tick.is_multiple_of(16) {
            self.assign_jobs();
            self.recount();
        }
        if self.clock.closes_month() {
            self.post_month();
        }
        if self.clock.tick.is_multiple_of(TICKS_PER_DAY) {
            self.weather_structures();
        }

        self.step_citizens();
    }

    /// One sim-day of wear on every standing structure: its **weakest** part's
    /// declared rate off its condition, and the repair flag set when it crosses the
    /// declared floor.
    ///
    /// This is §7.4's deterioration effect, which was declared, given a field on
    /// every structure, and computed by nothing until here. Condition is what moves:
    /// the as-built claim never does (a155), so a decayed structure still says what it
    /// was built from and only its condition says what it is now.
    ///
    /// The rate comes from `materials::effects`, which is the same lookup the test
    /// asserting a home's rate goes through — so a declared rate and a charged rate
    /// cannot drift apart.
    pub fn weather_structures(&mut self) {
        for building in self.buildings.iter_mut() {
            if building.retired.is_some() {
                continue;
            }
            let rate = material_effects::decay_per_day(building.kind.part_prefix());
            if rate <= 0.0 {
                continue;
            }
            building.condition = (building.condition - rate).max(0.0);
            if material_effects::repair_is_due(building.condition) {
                building.repair_filed = true;
            }
        }
    }

    /// The structures that owe a repair, each with the numbers its ticket is filed
    /// with: the structure's index, its tile, its condition, and what the repair of
    /// *this* structure costs.
    ///
    /// The world reports; it does not file. Filing is the governor's act (a175), and
    /// the sim has no governor — so a due repair is a reading here and a ticket there,
    /// rather than a second ticket-writing path nobody asked for.
    pub fn repairs_due(&self) -> Vec<(u32, u32, f32, f32)> {
        self.buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| b.retired.is_none() && b.repair_filed)
            .map(|(index, b)| {
                (
                    index as u32,
                    b.tile,
                    b.condition,
                    material_effects::repair_cost(b.kind.part_prefix()),
                )
            })
            .collect()
    }

    /// Close a repair: the structure is restored to the declared ceiling and stops
    /// being due. Returns the condition that closed it, which is exactly what the
    /// ticket is closed by reading back.
    pub fn repair(&mut self, index: usize) -> Option<f32> {
        let building = self.buildings.get_mut(index)?;
        if building.retired.is_some() {
            return None;
        }
        building.condition = REPAIR_CEILING;
        building.repair_filed = false;
        Some(building.condition)
    }

    fn recompute_power(&mut self) {
        let plants: Vec<u32> = self
            .buildings
            .iter()
            .filter(|b| b.kind == BuildingKind::PowerPlant && b.retired.is_none())
            .map(|b| b.tile)
            .collect();

        for tile in self.tiles.iter_mut() {
            tile.powered = false;
        }
        for building in self.buildings.iter_mut() {
            building.powered = false;
        }

        let capacity = plants.len() as u32 * POWER_PER_PLANT;
        let consumers = self
            .buildings
            .iter()
            .filter(|b| b.retired.is_none() && b.kind != BuildingKind::PowerPlant)
            .count() as u32;
        let brownout = consumers > capacity;
        self.stats.brownout = brownout;

        if !brownout {
            // Flood outwards from each plant along the road network, then light
            // the tiles that touch a lit road. Power follows streets, which is
            // what makes a plant's placement a real decision.
            let mut lit: Vec<u32> = Vec::new();
            for tile in plants {
                if let Some(node) = self
                    .roads
                    .nearest_node_within(self.width, self.height, tile, 3)
                {
                    let mut frontier = vec![(node, 0u32)];
                    let mut seen = std::collections::HashSet::new();
                    while let Some((current, depth)) = frontier.pop() {
                        if !seen.insert(current) || depth > POWER_RADIUS {
                            continue;
                        }
                        let road_tile = self.roads.nodes[current as usize].tile;
                        lit.push(road_tile);
                        for &next in self.roads.neighbours(current) {
                            frontier.push((next, depth + 1));
                        }
                    }
                }
            }
            for road_tile in lit {
                let (x, y) = self.coords(road_tile);
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if self.in_bounds(nx, ny) {
                            let tile = self.index(nx as u32, ny as u32);
                            self.tiles[tile as usize].powered = true;
                        }
                    }
                }
            }
        }

        for building in self.buildings.iter_mut() {
            building.powered = self.tiles[building.tile as usize].powered;
        }
    }

    fn recount(&mut self) {
        let mut stats = Stats::default();
        stats.road_tiles = self.tiles.iter().filter(|t| t.road).count() as u32;
        stats.powered_tiles = self.tiles.iter().filter(|t| t.powered).count() as u32;
        stats.brownout = self.stats.brownout;

        for building in &self.buildings {
            if building.retired.is_some() {
                stats.retired_buildings += 1;
                continue;
            }
            match building.kind {
                BuildingKind::Home => stats.homes += 1,
                BuildingKind::Shop => stats.shops += 1,
                BuildingKind::Factory => stats.factories += 1,
                BuildingKind::PowerPlant => stats.power_plants += 1,
            }
            if building.kind != BuildingKind::PowerPlant {
                stats.jobs += building.capacity();
            }
        }

        stats.population = self.citizens.len() as u32;
        stats.employed = self
            .citizens
            .iter()
            .filter(|c| c.work.is_some() && c.state != CitizenState::Unemployed)
            .count() as u32;
        stats.unemployed = stats.population.saturating_sub(stats.employed);
        stats.unpowered_zoned = self
            .tiles
            .iter()
            .filter(|t| t.zone != Zone::None && !t.powered)
            .count() as u32;
        stats.commute_failures = self.stats.commute_failures;
        self.stats = stats;
    }

    fn recompute_demand(&mut self) {
        let jobs = self.stats.jobs as f32;
        let population = self.stats.population as f32;
        let shops = self.stats.shops as f32;
        let factories = self.stats.factories as f32;
        let commercial_capacity = shops * 10.0;

        // Tuned against the C1 playtest, not asserted as correct: the numbers
        // here are a starting point and the record says so.
        self.demand.residential =
            ((jobs - population * 0.55) / 40.0 + 0.25).clamp(0.0, 1.0);
        self.demand.commercial =
            ((population * 0.28 - commercial_capacity) / 40.0 + 0.05).clamp(0.0, 1.0);
        self.demand.industrial =
            ((commercial_capacity * 0.6 + 30.0 - factories * 14.0) / 40.0).clamp(0.0, 1.0);
    }

    // `grow()` stood here: a function that decided a building should exist and then made one.
    // Round 6's Q45 answered that a city grows because citizens acquire material and build, and
    // round 11's Q72 confirmed it — so the function is **deleted, not kept beside the task system**,
    // and what replaces it is `post_demand_tasks` (work, not buildings), `claim_tasks` (a citizen
    // who takes it) and `work_tasks` (labour that only counts while somebody is standing on the
    // site). The test `growth_is_work_now_and_the_audit_balances` is where the difference shows:
    // the audit read a grown city as material nobody dug up, and reads a built one as conserved.

    /// Settle new residents into ready homes, then find work for the jobless.
    fn assign_jobs(&mut self) {
        let tick = self.clock.tick;

        // Counts are gathered first and spawns applied afterwards: a building
        // list borrowed immutably cannot be spawning citizens into the same
        // world, and letting the compiler enforce that is cheaper than
        // remembering it.
        let mut residents: Vec<u32> = vec![0; self.buildings.len()];
        let mut workers: Vec<u32> = vec![0; self.buildings.len()];
        for citizen in &self.citizens {
            if let Some(home) = citizen.home {
                if let Some(slot) = residents.get_mut(home as usize) {
                    *slot += 1;
                }
            }
            if let Some(work) = citizen.work {
                if let Some(slot) = workers.get_mut(work as usize) {
                    *slot += 1;
                }
            }
        }

        let homes: Vec<(u32, u32)> = self
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| b.is_ready(tick) && b.kind == BuildingKind::Home)
            .map(|(index, b)| (index as u32, b.capacity()))
            .collect();
        for (index, capacity) in homes {
            let have = residents.get(index as usize).copied().unwrap_or(0);
            for _ in have..capacity {
                self.spawn_citizen(index);
            }
        }

        let mut openings: Vec<(u32, u32)> = Vec::new();
        for (index, building) in self.buildings.iter().enumerate() {
            if !building.is_ready(tick)
                || building.kind == BuildingKind::PowerPlant
                || building.kind == BuildingKind::Home
            {
                continue;
            }
            let capacity = building.capacity();
            let have = workers.get(index).copied().unwrap_or(0);
            if have < capacity {
                openings.push((index as u32, capacity - have));
            }
        }

        for opening in openings {
            let (building, mut slots) = opening;
            while slots > 0 {
                match self
                    .citizens
                    .iter()
                    .position(|c| c.work.is_none() && c.ready)
                {
                    Some(index) => {
                        self.citizens[index].work = Some(building);
                        self.citizens[index].state = CitizenState::AtHome;
                        self.citizens[index].path.clear();
                        self.citizens[index].path_cursor = 0;
                        self.citizens[index].step_work = 0.0;
                    }
                    None => break,
                }
                slots -= 1;
            }
        }

        for citizen in self.citizens.iter_mut() {
            citizen.ready = true;
        }
    }

    fn spawn_citizen(&mut self, home: u32) {
        let id = self.next_citizen_id;
        self.next_citizen_id += 1;
        self.citizens.push(Citizen::new(id, home, self.clock.tick));
    }

    fn post_month(&mut self) {
        let mut income = 0i64;
        let mut expense = self.stats.road_tiles as i64 * 2;

        for building in &self.buildings {
            if building.retired.is_some() || !building.powered {
                continue;
            }
            let rate = match building.kind {
                BuildingKind::Home => self.economy.tax_residential,
                BuildingKind::Shop => self.economy.tax_commercial,
                BuildingKind::Factory => self.economy.tax_industrial,
                BuildingKind::PowerPlant => 0.0,
            };
            income += (building.capacity() as f32 * rate * 12.0) as i64;
        }

        expense += self.stats.power_plants as i64 * 120;
        expense += self.economy.loan_balance / 40;

        self.economy.month_income = income;
        self.economy.month_expense = expense;
        self.economy.credits += income - expense;
        self.economy.lifetime_income += income;
        self.economy.lifetime_expense += expense;

        if self.economy.credits < 0 {
            self.economy.months_in_debt += 1;
        } else {
            self.economy.months_in_debt = 0;
        }
    }

    fn step_citizens(&mut self) {
        let tick = self.clock.tick;
        let focus = self.focus;
        let mut full = 0u32;
        let mut failures = self.stats.commute_failures;
        // Tiles are copied out before the loop: updating an agent must not need
        // a whole-`self` borrow while the citizen list is borrowed mutably.
        let building_tiles: Vec<u32> = self.buildings.iter().map(|b| b.tile).collect();
        let mut arrivals: Vec<u32> = Vec::new();

        for citizen in self.citizens.iter_mut() {
            let Some(home) = citizen.home else { continue };
            let Some(&home_tile) = building_tiles.get(home as usize) else {
                continue;
            };
            let (hx, hy) = (home_tile % self.width, home_tile / self.width);
            let distance = (hx as i32 - focus.0).abs() + (hy as i32 - focus.1).abs();
            let is_full = distance <= LOD_RADIUS;
            if is_full {
                full += 1;
            }

            // A parked agent carries a one-tile path and has nothing to walk.
            if citizen.path.len() <= 1 {
                continue;
            }

            // Full agents walk a step at a time; offscreen agents advance a
            // tile per tick. Both are functions of the world, never the clock.
            let speed = if is_full { 0.25 } else { 1.0 };
            citizen.step_work += speed;
            if citizen.step_work < 1.0 {
                continue;
            }
            citizen.step_work -= 1.0;
            if citizen.step_work < 0.0 {
                citizen.step_work = 0.0;
            }
            citizen.path_cursor += 1;
            if citizen.path_cursor + 1 >= citizen.path.len() {
                // Arrived. The agent parks on its destination instead of
                // vanishing: a citizen at work is still a citizen.
                let arrived = citizen.path[citizen.path.len() - 1];
                citizen.path = vec![arrived];
                citizen.path_cursor = 0;
                citizen.step_work = 0.0;
                citizen.state = match citizen.state {
                    CitizenState::ToWork => CitizenState::AtWork,
                    CitizenState::ToHome => CitizenState::AtHome,
                    other => other,
                };
                if citizen.state == CitizenState::AtWork {
                    if let Some(work) = citizen.work {
                        arrivals.push(work);
                    }
                }
            }
        }

        for work in arrivals {
            if let Some(building) = self.buildings.get_mut(work as usize) {
                building.occupants += 1;
            }
        }

        // Route planning happens in a second pass, because it borrows the
        // road graph and cannot run while the citizens are borrowed mutably.
        let tick_budget = if tick.is_multiple_of(4) { 64 } else { 0 };
        let mut planned = 0;
        for index in 0..self.citizens.len() {
            if planned >= tick_budget {
                break;
            }
            let (state, work, home, task, needs_plan, ready) = {
                let c = &self.citizens[index];
                // A parked agent (a one-tile path) is due a new plan; an agent
                // mid-route is not.
                (c.state, c.work, c.home, c.task, c.path.len() <= 1, c.ready)
            };
            if !needs_plan || !ready {
                continue;
            }
            let Some(home) = home else { continue };
            let from_tile = self.buildings[home as usize].tile;
            // Where this citizen is heading and where they are heading back to. A **task** takes
            // precedence over a job: work the city owes itself is addressed to a site, and a site
            // is a tile like any other, so the same routing walks a citizen to it.
            let (origin, to_tile, onwards) = match (state, task, work) {
                (CitizenState::AtHome, Some(task), _) => {
                    let Some(task) = self.tasks.iter().find(|t| t.id == task) else {
                        continue;
                    };
                    // Where the task's **current leg** is: a gather walks to its patch first and
                    // to its site second, and which one it is now is the task's own answer.
                    (from_tile, task.destination(), CitizenState::ToWork)
                }
                // A citizen on a task who is standing still is between legs: a gather that has
                // just taken from a patch walks to the site it is carrying it to. Routed from
                // where they stand, and left `AtWork` so their next arrival is at the site.
                (CitizenState::AtWork, Some(task), _) => {
                    let Some(task) = self.tasks.iter().find(|t| t.id == task) else {
                        continue;
                    };
                    let here = self.citizens[index].current_tile().unwrap_or(from_tile);
                    // Already on the leg's tile: there is nothing to walk, and the work loop owns
                    // them from here. Planning anyway would hand them a one-tile path and spend
                    // the tick budget re-deciding that every pass.
                    if within_reach(self.width, here, task.destination()) {
                        continue;
                    }
                    (here, task.destination(), CitizenState::ToWork)
                }
                (CitizenState::AtHome, None, Some(job)) => {
                    let Some(building) = self.buildings.get(job as usize) else { continue };
                    (from_tile, building.tile, CitizenState::ToWork)
                }
                (CitizenState::AtWork, _, Some(job)) => {
                    let Some(building) = self.buildings.get(job as usize) else { continue };
                    (
                        self.citizens[index].current_tile().unwrap_or(from_tile),
                        building.tile,
                        CitizenState::ToHome,
                    )
                }
                // A citizen whose task is finished walks home from wherever the work was, which is
                // how a worker released at a build site gets back rather than standing there.
                (CitizenState::ToHome, _, _) => (
                    self.citizens[index].current_tile().unwrap_or(from_tile),
                    from_tile,
                    CitizenState::AtHome,
                ),
                _ => continue,
            };
            // The same reach the work check uses, so a citizen is never routed to a tile they
            // cannot work from and never refused a tile they were routed to.
            let route = self
                .roads
                .route_between_tiles(self.width, self.height, origin, to_tile, WORK_REACH);
            planned += 1;
            match route {
                Some(path) => {
                    let citizen = &mut self.citizens[index];
                    citizen.path = path;
                    citizen.path_cursor = 0;
                    citizen.step_work = 0.0;
                    citizen.state = onwards;
                }
                None => {
                    // A commute with no route is a refusal, counted and
                    // surfaced as a case -- never a silent teleport.
                    failures += 1;
                    let citizen = &mut self.citizens[index];
                    citizen.path.clear();
                    citizen.path_cursor = 0;
                    citizen.step_work = 0.0;
                }
            }
        }

        self.stats.commute_failures = failures;
        self.stats.full_agents = full;
    }

    /// A save. `ron` rather than JSON because a 65k-tile world in JSON is a
    /// text file nobody wants to open.
    /// Write the world through a temporary file and rename it into place.
    ///
    /// An autosave can land mid-frame at an arbitrary moment, and a process
    /// killed while writing directly to the destination leaves a half-written
    /// file that parses as nothing. The rename is the part that is atomic, so a
    /// reader sees either the previous save or the new one, never a fragment.
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if self.format_version != FORMAT_VERSION {
            // Writing a save stamped with a version it was not written under would make
            // the format field a decoration. A world in memory is always current.
            return Err(std::io::Error::other(format!(
                "refusing to save: this world is stamped v{} and this build writes v{}",
                self.format_version, FORMAT_VERSION
            )));
        }
        // Compact on purpose. The record files are pretty because a reader has
        // to look at them; this snapshot is disposable, untracked, and rewritten
        // on a timer, so the only things that matter are bytes and milliseconds.
        let text = ron::ser::to_string(self).map_err(|e| std::io::Error::other(e.to_string()))?;
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        let tmp = std::path::PathBuf::from(tmp);
        std::fs::write(&tmp, text)?;
        match std::fs::rename(&tmp, path) {
            Ok(()) => Ok(()),
            Err(err) => {
                // Leaving the partial file behind would be misleading: the
                // destination is intact, and the tmp is not a save.
                let _ = std::fs::remove_file(&tmp);
                Err(err)
            }
        }
    }

    pub fn load(path: &std::path::Path) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let mut world: World =
            ron::from_str(&text).map_err(|e| std::io::Error::other(e.to_string()))?;
        world.migrate()?;
        world.rebuild_derived();
        Ok(world)
    }

    /// Bring a loaded save up to [`FORMAT_VERSION`], or refuse.
    ///
    /// Two refusals, and both are a152's rule rather than caution for its own sake: a save
    /// from a **newer** build cannot be read correctly by this one, so reading it anyway
    /// would be inventing state; and a **current** save missing a material claim is a
    /// defect to report, not a case to patch over, because the only reason it could be
    /// missing is that something wrote a v2 file without the thing v2 is for.
    pub fn migrate(&mut self) -> std::io::Result<()> {
        if self.format_version > FORMAT_VERSION {
            return Err(std::io::Error::other(format!(
                "this save is format v{} and this build understands up to v{}; a newer save \
                 read by an older build is invented state",
                self.format_version, FORMAT_VERSION
            )));
        }
        if self.format_version == FORMAT_VERSION {
            let missing: Vec<u32> = self
                .buildings
                .iter()
                .filter(|building| building.material_as_built.is_none())
                .map(|building| building.id)
                .collect();
            if !missing.is_empty() {
                return Err(std::io::Error::other(format!(
                    "a v{FORMAT_VERSION} save carries {} structure(s) with no as-built \
                     material ({}); that is a defect in the file, not a version to migrate",
                    missing.len(),
                    missing
                        .iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
            }
            return Ok(());
        }

        // v1 → v2. The named migration: every structure that predates the world having
        // materials gets the material its own kind declares, and **the derivation is
        // reported** rather than happening quietly (a152).
        let mut derived = Vec::new();
        for building in self.buildings.iter_mut() {
            if building.material_as_built.is_none() {
                building.material_as_built = MaterialClaim::of_kind(building.kind);
                if let Some(claim) = &building.material_as_built {
                    derived.push((building.id, claim.describe()));
                }
            }
        }
        let from = self.format_version;
        self.format_version = FORMAT_VERSION;
        self.migration = Some(Migration {
            from,
            to: FORMAT_VERSION,
            derived,
        });
        Ok(())
    }
}

/// The first task id. A save predating tasks starts here, and an absent `next_task_id` in an old
/// save defaults to it — the same reasoning as a tile with no extraction having extracted nothing.
fn first_task_id() -> u32 {
    1
}

#[cfg(test)]
pub mod tests_support {
    use super::{Terrain, Tile};

    /// Ground tiles, no water, no roads. Used by tests that need a clean grid.
    pub fn blank_tiles(width: u32, height: u32) -> Vec<Tile> {
        (0..width * height)
            .map(|_| Tile {
                terrain: Terrain::Ground,
                ..Default::default()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The claim of the whole campaign, as a test: **a city made by hand is made of what was dug
    /// up, to the gram.** A building placed on ground whose deposit was worked out first leaves
    /// the ledger at exactly zero, and the only way to move it is to move real mass.
    #[test]
    fn a_dug_city_balances_to_the_gram() {
        let mut world = World::new(32, 32, 7);
        for tile in world.tiles.iter_mut() {
            tile.terrain = Terrain::Ground;
            tile.road = false;
        }

        // Find a tile that actually holds something, and take all of it.
        let rich = (0..1024u32)
            .find(|tile| world.deposit_at(*tile).is_some())
            .expect("a 32×32 world has ore in it somewhere");
        let available = world.deposit_at(rich).expect("just found it").mass_g;
        assert_eq!(world.extract(rich, available), available, "the take is what was there");

        // The ground is debited by exactly that, at the substance it derives to.
        let audit = world.mass_audit();
        let ground: i64 = audit
            .holdings()
            .iter()
            .filter(|(account, _)| account.starts_with("ground:"))
            .map(|(_, grams)| *grams)
            .sum();
        assert_eq!(ground, -available, "the ground goes down by exactly the take");
        assert_eq!(audit.extracted_g, available);
        assert_eq!(audit.standing_g, 0, "nothing has been built yet");
        assert!(
            audit.conserves(),
            "material taken and not yet built is loose, not invented"
        );
        assert_eq!(audit.loose_g(), available, "all of it is loose so far");

        // Now stand something on the same tile. Its mass is derived, not stored — and the loose
        // remainder is the take minus the house, which is the shape of the whole economy.
        world.place_building(rich, BuildingKind::Home).expect("ground, no road, empty");
        let audit = world.mass_audit();
        let home = material_ledger::structure_mass_g("home", 1).expect("declared");
        assert_eq!(audit.of("structure:ceramic"), home, "a declared mass, read off the kind");
        assert_eq!(world.buildings[0].mass_g(), Some(home));
        assert_eq!(audit.extracted_g, available, "building does not change what was dug");
        assert_eq!(audit.standing_g, home);
        assert_eq!(
            audit.loose_g(),
            available - home,
            "the remainder is real material, still in the world"
        );
        assert!(
            audit.holdings().iter().any(|(account, grams)| account.starts_with("ground:") && *grams < 0),
            "and the ground it came from is still visibly debited"
        );
    }

    /// A structure that appeared without anything being dug up is the finding, and the finding
    /// is the *amount*: this is `grow()` printed rather than argued about.
    #[test]
    fn material_that_was_never_dug_up_is_reported_as_an_imbalance() {
        let mut world = World::new(32, 32, 7);
        for tile in world.tiles.iter_mut() {
            tile.terrain = Terrain::Ground;
            tile.road = false;
        }
        world.place_building(0, BuildingKind::Home).expect("empty ground");

        let audit = world.mass_audit();
        assert!(!audit.conserves(), "a house that grew out of nothing is not conservation");
        let mass = material_ledger::structure_mass_g("home", 1).expect("declared");
        assert_eq!(audit.loose_g(), -mass, "the defect is exactly the house, as a quantity");
        let findings = audit.findings();
        assert_eq!(findings.len(), 1, "one finding, naming the number: {findings:?}");
        assert!(findings[0].contains("never gave up"), "{findings:?}");
        assert!(findings[0].contains(&mass.to_string()), "the amount is named: {findings:?}");
    }

    /// A demolition moves mass to the ruins rather than destroying it, because retirement
    /// preserves the material as surely as it preserves the record.
    #[test]
    fn demolishing_moves_mass_to_the_ruins() {
        let mut world = World::new(32, 32, 7);
        for tile in world.tiles.iter_mut() {
            tile.terrain = Terrain::Ground;
            tile.road = false;
        }
        world.place_building(0, BuildingKind::Shop).expect("empty ground");
        let mass = material_ledger::structure_mass_g("shop", 1).expect("declared");
        assert_eq!(world.mass_audit().of("structure:ceramic"), mass);

        world.demolish(0, crate::gov::RetirementReason::Superseded);
        let audit = world.mass_audit();
        assert_eq!(audit.of("structure:ceramic"), 0, "it is no longer a standing structure");
        assert_eq!(audit.of("ruin:ceramic"), mass, "the material is still in the world");
        assert_eq!(
            audit.standing_g, mass,
            "demolition conserved the mass; only its account changed"
        );
    }

    /// A world at the size the game actually is, worked down to bare ground, for tests about
    /// material rather than about terrain.
    ///
    /// **Why not `small_city`:** a 32×32 world is one geology cell across, and a cell is one
    /// patch, so such a world holds **one substance everywhere** — seed 7's holds iron and nothing
    /// else. That is faithful to the derivation and it is exactly the world a ceramic home cannot
    /// be built on, which is why it is used *for that test* rather than as a general fixture.
    fn worked_city() -> World {
        let mut world = World::new(256, 256, 7);
        for tile in world.tiles.iter_mut() {
            tile.terrain = Terrain::Ground;
        }
        // A road down the left edge, so the tiles beside it are reachable and can be lit — the
        // same seed road `small_city` lays, at the size a city actually is.
        for y in 0..256u32 {
            world.tiles[(y * 256) as usize].road = true;
        }
        world.rebuild_derived();
        world
    }

    /// Total mass of a family's deposits in a world, in grams, and how many tiles hold it.
    fn family_totals(world: &World, family: &str) -> (u32, i64) {
        let mut tiles = 0;
        let mut mass = 0;
        for index in 0..world.tiles.len() as u32 {
            if let Some(deposit) = world.deposit_at(index) {
                if material_geology::kind_family(deposit.kind) == family {
                    tiles += 1;
                    mass += deposit.mass_g;
                }
            }
        }
        (tiles, mass)
    }

    /// The ground has to be able to supply what the city's own structures are made of, or the
    /// bootstrap is impossible for a reason nobody declared. Measured, per family, rather than
    /// assumed: this is the check that would have caught a world whose ceramic was thinner than
    /// one home before a player ever met it.
    #[test]
    fn a_game_sized_world_can_supply_every_family_its_structures_claim() {
        let world = worked_city();
        for kind in [BuildingKind::Home, BuildingKind::Shop, BuildingKind::Factory, BuildingKind::PowerPlant] {
            let claim = MaterialClaim::of_kind(kind).expect("declared claim");
            let (tiles, mass) = family_totals(&world, &claim.family);
            let needs = material_ledger::structure_mass_g(kind.name(), 1).expect("declared");
            assert!(
                tiles > 0 && mass >= needs,
                "{} needs {} g of {} and the world holds {} g across {} tiles",
                kind.name(),
                needs,
                claim.family,
                mass,
                tiles
            );
        }
    }

    fn small_city() -> World {
        let mut world = World::new(32, 32, 7);
        // Fill the water in, so tests exercise the city rather than the sea.
        for tile in world.tiles.iter_mut() {
            tile.terrain = Terrain::Ground;
        }
        for y in 0..32u32 {
            world.tiles[(y * 32) as usize].road = true;
        }
        world.rebuild_derived();
        world
    }

    #[test]
    fn a_new_world_has_a_seed_road_and_credits() {
        let world = small_city();
        assert!(world.stats.road_tiles >= 32);
        assert_eq!(world.economy.credits, 25_000);
        assert_eq!(world.stats.population, 0);
    }

    #[test]
    fn the_same_seed_builds_the_same_terrain() {
        let a = World::new(64, 64, 1234);
        let b = World::new(64, 64, 1234);
        for (x, y) in [(0u32, 0u32), (10, 10), (63, 63), (5, 40)] {
            let i = (y * 64 + x) as usize;
            assert_eq!(a.tiles[i].terrain, b.tiles[i].terrain);
        }
    }

    #[test]
    fn a_road_leg_lays_the_tiles_between_its_ends() {
        let mut world = small_city();
        let from = world.index(4, 4);
        let to = world.index(8, 4);
        let laid = world.lay_road(from, to);
        assert_eq!(laid.len(), 5, "expected five tiles from x=4 to x=8");
        for x in 4..=8u32 {
            assert!(world.tiles[world.index(x, 4) as usize].road);
        }
        assert!(world.roads.len() >= 5);
    }

    #[test]
    fn zoning_is_refused_on_roads_and_buildings() {
        let mut world = small_city();
        let road_tile = world.index(0, 5);
        assert!(!world.set_zone(road_tile, Zone::Residential));
        let free = world.index(4, 20);
        assert!(world.set_zone(free, Zone::Residential));
        assert_eq!(world.tiles[free as usize].zone, Zone::Residential);
    }

    /// §7.4's deterioration effect, reached through the clock rather than called
    /// directly: a declared rate off the condition every sim-day, the floor crossing
    /// that owes **one** repair, and the repair that settles it by the condition it
    /// wrote back.
    ///
    /// Through the tick on purpose. The defect this fixes was a field nothing
    /// touched, and a test that calls `weather_structures` itself would pass just as
    /// happily with the call missing from `tick`.
    #[test]
    fn a_structure_decays_a_day_at_a_time_and_owes_one_repair() {
        use crate::materials::generated::REPAIR_FLOOR;

        let mut world = small_city();
        let tile = world.index(4, 4);
        world.place_building(tile, BuildingKind::Home).expect("placed");
        let rate = material_effects::decay_per_day(BuildingKind::Home.part_prefix());
        let claim = world.buildings[0].material_as_built.clone();
        assert_eq!(world.buildings[0].condition, 1.0, "as-built");
        assert!(world.repairs_due().is_empty(), "nothing is owed on day one");

        // The tick before a day closes does nothing; the one that closes it wears the
        // structure by exactly one declared day.
        world.clock.tick = TICKS_PER_DAY - 2;
        world.tick();
        assert_eq!(world.buildings[0].condition, 1.0, "a day has not passed");
        world.clock.tick = TICKS_PER_DAY - 1;
        world.tick();
        assert!(
            (world.buildings[0].condition - (1.0 - rate)).abs() < 1e-6,
            "one sim-day of wear at the declared rate, got {}",
            world.buildings[0].condition
        );

        // Drive it to just above the floor and let a day cross it.
        world.buildings[0].condition = REPAIR_FLOOR + rate / 2.0;
        world.clock.tick = TICKS_PER_DAY * 2 - 1;
        world.tick();
        assert!(world.buildings[0].condition < REPAIR_FLOOR);
        let due = world.repairs_due();
        assert_eq!(due.len(), 1, "the structure owes a repair: {due:?}");
        assert_eq!(due[0].1, tile);
        assert!(
            (due[0].3 - material_effects::repair_cost("home")).abs() < 1e-6,
            "a repair is priced against the thing repaired"
        );

        // Still one, a day later: a crossed floor is a repair owed, not a repair per
        // day, which is what the flag on the structure is for.
        world.clock.tick = TICKS_PER_DAY * 3 - 1;
        world.tick();
        assert_eq!(world.repairs_due().len(), 1);

        // The repair settles it, and the condition is what closed it.
        let closed = world.repair(0).expect("a standing structure can be repaired");
        assert_eq!(closed, REPAIR_CEILING);
        assert!(world.repairs_due().is_empty());
        assert!(!world.buildings[0].repair_filed);
        assert_eq!(
            world.buildings[0].material_as_built, claim,
            "the condition moves and the as-built claim never does (a155)"
        );

        // A retired structure owes nothing: it is history, not a liability.
        world.demolish(tile, RetirementReason::Superseded);
        world.buildings[0].condition = 0.1;
        world.buildings[0].repair_filed = true;
        assert!(world.repairs_due().is_empty(), "a ruin is not repairable");
        assert!(world.repair(0).is_none());
    }

    /// A tile gives up only what it holds (C9 phase 1). The deposit is derived, the delta
    /// is stored, and the two agree with the geology module rather than with each other.
    #[test]
    fn a_tile_gives_up_only_what_it_holds_and_then_is_spent() {
        let mut world = World::new(256, 256, 0xC117_2026);
        let (tile, declared) = (0..256u32 * 256)
            .find_map(|tile| {
                world.deposit_at(tile).map(|found| (tile, found.mass_g))
            })
            .expect("this seed's map holds at least one deposit");

        // Nothing taken yet, so the reading is the derivation itself.
        let (x, y) = world.coords(tile);
        assert_eq!(
            declared,
            material_geology::deposit(world.seed, x, y).unwrap().mass_g,
            "an untouched tile reads exactly what the seed derives"
        );

        // Asking for more than is there takes only what is there.
        let over = declared + 1_000_000;
        assert_eq!(world.extract(tile, over), declared, "the request is not the take");
        assert!(
            world.deposit_at(tile).is_none(),
            "a worked-out tile holds nothing"
        );
        assert_eq!(world.extract(tile, 1000), 0, "a spent tile gives nothing more");
        assert_eq!(world.tiles[tile as usize].extracted_g, declared);

        // A part-worked tile reports the remainder, which is what a mine's rate limit reads.
        let other = find_another(&world, tile);
        let full = world.deposit_at(other).expect("found").mass_g;
        assert_eq!(world.extract(other, full / 3), full / 3);
        assert_eq!(world.deposit_at(other).expect("remains").mass_g, full - full / 3);

        fn find_another(world: &World, skip: u32) -> u32 {
            (0..256u32 * 256)
                .find(|tile| *tile != skip && world.deposit_at(*tile).is_some())
                .expect("a second deposit exists")
        }
    }

    /// §8.24 deferred `SURFACE_SPEED` for want of a consumer. This is the consumer, and the
    /// three readings a driver feels: a road at full speed, bare ground slower, water not slow
    /// but *impassable*.
    #[test]
    fn a_surface_reads_at_its_own_declared_speed() {
        let world = small_city();
        let road = world.index(0, 5);
        assert_eq!(world.tiles[road as usize].surface_family(), "road");
        assert_eq!(world.surface_speed(road), 1.0);

        let bare = world.index(4, 20);
        assert_eq!(world.tiles[bare as usize].surface_family(), "soil");
        assert_eq!(world.surface_speed(bare), 0.6);

        if let Some(water) = (0..world.tiles.len() as u32)
            .find(|tile| world.tiles[*tile as usize].terrain == Terrain::Water)
        {
            assert_eq!(
                world.surface_speed(water),
                0.0,
                "water is impassable rather than slow"
            );
        }
    }

    #[test]
    fn a_demolished_building_is_retired_not_deleted() {
        let mut world = small_city();
        let tile = world.index(4, 4);
        let id = world.place_building(tile, BuildingKind::Home).expect("placed");
        assert_eq!(world.buildings.len(), 1);
        let retired = world.demolish(tile, RetirementReason::Superseded);
        assert_eq!(retired, Some(id));
        assert_eq!(
            world.buildings.len(),
            1,
            "retirement must not remove the building from the record"
        );
        assert_eq!(
            world.buildings[0].retired,
            Some(RetirementReason::Superseded)
        );
        assert!(world.tiles[tile as usize].building.is_none());
        assert_eq!(world.stats.retired_buildings, 1);
    }

    #[test]
    fn power_follows_the_road_network() {
        let mut world = small_city();
        // A plant near the seed road, and a home far from it.
        let plant_tile = world.index(1, 3);
        world.place_building(plant_tile, BuildingKind::PowerPlant).expect("plant");
        let near = world.index(0, 3);
        let far = world.index(30, 30);
        world.buildings[0].ready_tick = 0;
        world.recompute_power();
        assert!(world.tiles[near as usize].powered, "tile beside the plant's road should be lit");
        assert!(
            !world.tiles[far as usize].powered,
            "a tile with no road within the radius must stay dark"
        );
    }

    /// Power and a road decide **whether the work is posted at all** — the same rule `grow()` had,
    /// moved one step earlier: the city will not ask anyone to build where it cannot light or
    /// reach the site, so the queue never fills with jobs that cannot be done.
    #[test]
    fn zoned_tiles_get_work_only_with_power_and_a_road() {
        let mut world = worked_city();
        world.place_building(world.index(1, 3), BuildingKind::PowerPlant).expect("plant");
        world.buildings[0].ready_tick = 0;
        // Beside the seed road: work expected.
        let near = world.index(1, 6);
        world.set_zone(near, Zone::Residential);
        // In the far corner with no road: no work, and no building either.
        let far = world.index(250, 250);
        world.set_zone(far, Zone::Residential);
        world.recompute_power();
        world.recompute_demand();
        for _ in 0..40 {
            world.clock.tick += 1;
            world.post_demand_tasks();
        }
        assert!(
            world.open_task_at(near).is_some(),
            "a powered, road-connected zoned tile should have work posted for it"
        );
        assert!(
            world.open_task_at(far).is_none(),
            "an unpowered tile with no road must not be given work"
        );
        assert!(
            world.tiles[near as usize].building.is_none(),
            "and work posted is not a building: the citizen has to do it"
        );
    }

    /// A structure is built **out of material the world gives up**, and the lineage says which
    /// tiles paid for it. This is the property that makes "built from something" a fact rather
    /// than a claim: the grams in the lineage are grams that left the ground.
    #[test]
    fn a_built_structure_is_paid_for_out_of_named_tiles() {
        let mut world = worked_city();
        let site = world.index(4, 4);
        let home = material_ledger::structure_mass_g("home", 1).expect("declared");
        let id = world
            .build_with_material(site, BuildingKind::Home)
            .unwrap_or_else(|short| panic!("a game-sized world should cover a home: {}", short.describe()));
        let index = world.tiles[site as usize].building.expect("the home stands on its site");
        assert_eq!(world.buildings[index as usize].id, id, "it returns the structure it built");

        let building = &world.buildings[index as usize];
        assert_eq!(building.lineage_g(), home, "the lineage accounts for the whole mass");
        assert!(
            !building.material_from.is_empty(),
            "a built structure records where its mass came from"
        );
        for line in &building.material_from {
            assert!(line.grams > 0, "a lineage line of nothing is not a line");
            let deposit = world
                .deposit_at(line.tile)
                .map(|d| (d.kind, d.mass_g))
                .map(|(kind, remaining)| (material_geology::DEPOSIT_KINDS[kind].substance, remaining));
            assert!(
                deposit.is_some(),
                "tile {} is not shown as dug, so the lineage names a hole that is not there",
                line.tile
            );
        }
        // The world holds the house and nothing else: exactly what it dug.
        let audit = world.mass_audit();
        assert!(audit.conserves(), "the material came out of the world's own ground");
        assert_eq!(audit.extracted_g, home, "and it took exactly what the house weighs");
        assert_eq!(audit.loose_g(), 0, "nothing is left lying around from a single build");
    }

    /// The material has to be the **right kind**: a structure only draws from deposits whose
    /// declared family is the family it claims. Without this, a brick home could be raised on an
    /// iron seam and the ledger would balance in tonnes while being nonsense in substance.
    #[test]
    fn a_structure_only_draws_from_its_own_family() {
        let world = worked_city();
        let site = world.index(4, 4);
        let claim = MaterialClaim::of_kind(BuildingKind::Home).expect("declared claim");
        let plan = world
            .plan_material(site, &claim.family, material_ledger::structure_mass_g("home", 1).unwrap())
            .expect("a 32×32 world should cover a home");
        for line in &plan {
            let deposit = world.deposit_at(line.tile).expect("a planned tile holds something");
            let substance = material_geology::kind_substance(deposit.kind);
            let family = material_geology::kind_family(deposit.kind);
            assert_eq!(
                family, claim.family,
                "tile {} is {} ({}), not {}",
                line.tile, substance, family, claim.family
            );
            assert_eq!(line.substance, substance, "and the line names what is there");
        }
    }

    /// A shortfall refuses **atomically**: the plan is computed without touching the ground, so an
    /// unobtainable build leaves the map exactly as it found it rather than half-mined.
    #[test]
    fn a_shortfall_refuses_without_mining_anything() {
        let world = worked_city();
        let before: i64 = world.tiles.iter().map(|tile| tile.extracted_g).sum();
        let site = world.index(4, 4);
        let shortfall = world
            .plan_material(site, "ceramic", i64::MAX / 2)
            .expect_err("no world holds that much clay");
        assert!(shortfall.found_g > 0, "it says what it did find: {}", shortfall.found_g);
        assert!(shortfall.missing_g() > 0);
        assert!(shortfall.describe().contains("short"), "{}", shortfall.describe());

        let after: i64 = world.tiles.iter().map(|tile| tile.extracted_g).sum();
        assert_eq!(after, before, "planning is a read; a refusal must not leave holes");
        assert_eq!(world.buildings.len(), 0);
    }

    /// The same world plans the same tiles in the same order — including which of two equidistant
    /// tiles is chosen. A nearest-neighbour search with an arbitrary order is the quietest way to
    /// lose a replay, so the tie-break is part of the answer.
    #[test]
    fn the_same_world_plans_the_same_tiles() {
        let plan = |world: &World| {
            world
                .plan_material(4 * 32 + 4, "ceramic", material_ledger::structure_mass_g("home", 1).unwrap())
                .expect("coverable")
                .iter()
                .map(|line| (line.tile, line.grams))
                .collect::<Vec<_>>()
        };
        assert_eq!(plan(&worked_city()), plan(&worked_city()));
    }

    /// **A world of iron cannot raise a brick house.** Seed 7 at 32×32 is one geology cell, so it
    /// holds metal everywhere and no ceramic at all — which makes it the honest test of the family
    /// rule: the plan does not exist, `build_with_material` refuses, and the refusal says what it
    /// wanted and what it found rather than building the house out of ore.
    #[test]
    fn a_ceramic_home_cannot_be_built_on_a_world_of_iron() {
        let mut world = small_city();
        let (ceramic_tiles, _) = family_totals(&world, "ceramic");
        assert_eq!(ceramic_tiles, 0, "this world is the fixture *because* it has no ceramic");
        let (metal_tiles, metal_mass) = family_totals(&world, "metal");
        assert!(metal_tiles > 0, "and it does hold metal, so the refusal is about substance");

        let site = world.index(4, 4);
        let shortfall = world
            .build_with_material(site, BuildingKind::Home)
            .expect_err("a home is ceramic and there is no ceramic");
        assert_eq!(shortfall.found_g, 0);
        assert!(shortfall.family.contains("ceramic"), "it names what it wanted: {}", shortfall.family);
        assert!(world.buildings.is_empty(), "and nothing was placed");
        assert_eq!(
            world.tiles.iter().map(|tile| tile.extracted_g).sum::<i64>(),
            0,
            "not even out of the metal, which is there and is the wrong material"
        );
        assert!(metal_mass > 0);
    }

    /// **What replaced the measured violation.** `grow()` made buildings out of nothing; a test
    /// used to pin that as a number, and this is the same scenario through the task system, where
    /// the audit no longer reports material that was never dug.
    ///
    /// The whole chain in one place: demand posts *work*, a citizen takes it, the work only
    /// advances while they are standing on the site, and completing it draws the plan and raises
    /// the structure with its lineage. Every stated property has an assertion, so "growth is work
    /// now" is a claim a reader can check rather than take.
    #[test]
    fn growth_is_work_now_and_the_audit_balances() {
        let mut world = worked_city();
        // The plant and the first home are **built**, not placed: the audit at the end is the whole
        // city's, so anything standing unpaid would (correctly) fail it. Paying for them is also a
        // live exercise of the same path the site's work takes.
        let plant = world.index(1, 3);
        world
            .build_with_material(plant, BuildingKind::PowerPlant)
            .unwrap_or_else(|short| panic!("a power plant must be payable: {}", short.describe()));
        let plant_index = world.tiles[plant as usize].building.expect("the plant");
        world.buildings[plant_index as usize].ready_tick = 0;
        world.recompute_power();

        let housing = world.index(1, 6);
        world
            .build_with_material(housing, BuildingKind::Home)
            .unwrap_or_else(|short| panic!("a home must be payable: {}", short.describe()));
        let home_index = world.tiles[housing as usize].building.expect("the home");
        world.buildings[home_index as usize].ready_tick = 0;
        for _ in 0..4 {
            world.clock.tick += 1;
            world.assign_jobs();
        }
        assert!(!world.citizens.is_empty(), "the home has residents to do the work");

        // A zoned, powered, connected tile. The demand posts **work**, and posts no building.
        let site = world.index(1, 8);
        world.set_zone(site, Zone::Residential);
        world.recompute_power();
        world.recompute_demand();
        world.post_demand_tasks();
        let task = world
            .open_task_at(site)
            .unwrap_or_else(|| {
                panic!(
                    "no work on the site: demand {}, powered {}, zone {:?}, road access {}, starved {:?}",
                    world.demand.residential,
                    world.tiles[site as usize].powered,
                    world.tiles[site as usize].zone,
                    world.has_road_access(site),
                    world.starved()
                )
            })
            .clone();
        assert_eq!(task.verb, sim_task::Verb::Build);
        assert_eq!(task.kind, BuildingKind::Home);
        assert!(!task.is_claimed(), "nobody is on the work until a citizen takes it");
        assert!(
            world.tiles[site as usize].building.is_none(),
            "posting work must not raise the building — that is the whole point"
        );
        assert_eq!(
            task.lineage_total(),
            material_ledger::structure_mass_g("home", 1).expect("declared"),
            "and the material it will take is planned before anyone is asked to work"
        );

        // A citizen takes it, and walks. The counter must not move while they are elsewhere.
        world.claim_tasks();
        let worker = world
            .tasks
            .iter()
            .find(|t| t.id == task.id)
            .and_then(|t| t.claimed_by)
            .expect("an idle resident takes work the city owes itself");
        let before = world.tasks[0].work_remaining;
        world.work_tasks();
        assert_eq!(
            world.tasks[0].work_remaining, before,
            "work nobody is standing on does not advance"
        );

        // Now stand them on the site, and work it out. This is the atomic, verifiable part: the
        // counter only turns while they are there.
        let index = world.citizens.iter().position(|c| c.id == worker).expect("the worker");
        world.citizens[index].state = CitizenState::AtWork;
        world.citizens[index].path = vec![site];
        world.citizens[index].path_cursor = 0;
        for _ in 0..before.max(1) {
            world.work_tasks();
        }
        assert!(world.tasks.is_empty(), "the work finished and retired itself");
        assert!(
            world.tiles[site as usize].building.is_some(),
            "and the structure stands, because the work was done on it"
        );
        let built = &world.buildings[world.tiles[site as usize].building.unwrap() as usize];
        assert!(
            built.lineage_g() == built.mass_g().expect("declared"),
            "its material came out of named tiles, to the gram"
        );

        let audit = world.mass_audit();
        assert!(
            audit.conserves(),
            "the city is made of what it dug, which is what grow() could never say"
        );
        assert!(audit.loose_g() >= 0);
    }

    // -----------------------------------------------------------------
    // The stone rung: gathering, carrying, and making (C9 phase 3, Q102/Q77/Q66)
    // -----------------------------------------------------------------

    /// A city with the works a first tool needs: the seed road, a road to a **timber patch** and
    /// to a **stone seam**, a power plant and an occupied home.
    ///
    /// Building the roads is the point rather than pageantry. Q102's gathers are taken by hand and
    /// Q84 settled that a person walks anywhere — but this sim walks its citizens along roads, so a
    /// patch with no road within [`WORK_REACH`] is a patch nobody can gather. Laying the road is
    /// the test saying that out loud instead of asserting a rung the world cannot reach.
    fn rung_world() -> World {
        let mut world = worked_city();

        // The nearest tile of each kind the rung needs, searched in tile order so the fixture is
        // the same world every run.
        let timber = (0..world.tiles.len() as u32)
            .find(|index| {
                world
                    .surface_at(*index)
                    .is_some_and(|standing| standing.substance == "timber" && standing.mass_g >= 3000)
            })
            .expect("the world grows timber somewhere");
        let seam = (0..world.tiles.len() as u32)
            .find(|index| {
                world.deposit_at(*index).is_some_and(|deposit| {
                    material_geology::kind_substance(deposit.kind) == "stone" && deposit.mass_g >= 1000
                })
            })
            .expect("the world holds stone somewhere");

        // A road from the seed road down the left edge to each patch, in two straight legs so the
        // tiles are orthogonally connected — a staircase is not a road, and the graph knows it.
        for patch in [timber, seam] {
            let (x, y) = world.coords(patch);
            let approach = world.index(x.saturating_sub(1), y);
            world.lay_road(world.index(0, 0), world.index(0, y));
            world.lay_road(world.index(0, y), approach);
        }
        world.rebuild_derived();

        // The works the rung is worked from: a plant (so the city can light itself) and a home
        // whose residents are the hands. Both are **built** rather than placed, so the audit at
        // the end is the whole city's.
        let plant = world.index(1, 3);
        world
            .build_with_material(plant, BuildingKind::PowerPlant)
            .unwrap_or_else(|short| panic!("a power plant must be payable: {}", short.describe()));
        let plant_index = world.tiles[plant as usize].building.expect("the plant");
        world.buildings[plant_index as usize].ready_tick = 0;
        world.recompute_power();

        let housing = world.index(1, 6);
        world
            .build_with_material(housing, BuildingKind::Home)
            .unwrap_or_else(|short| panic!("a home must be payable: {}", short.describe()));
        let home_index = world.tiles[housing as usize].building.expect("the home");
        world.buildings[home_index as usize].ready_tick = 0;
        for _ in 0..4 {
            world.clock.tick += 1;
            world.assign_jobs();
        }
        assert!(!world.citizens.is_empty(), "the home has residents to do the work");
        world
    }

    /// Idle residents take the rung's work and do it, standing where each leg of their task is.
    ///
    /// Routing is exercised by the tick in its own test; what this drives is the **mass path**:
    /// patch → carrier → site → process → holding, with the order enforced by what a task can
    /// start on rather than by a schedule.
    fn drive_the_rung(world: &mut World, rounds: usize) {
        for _ in 0..rounds {
            // A released worker is a resident at home again, as far as claiming is concerned.
            for index in 0..world.citizens.len() {
                if world.citizens[index].task.is_some() {
                    continue;
                }
                let Some(home) = world.citizens[index].home else { continue };
                let home_tile = world.buildings[home as usize].tile;
                let citizen = &mut world.citizens[index];
                citizen.state = CitizenState::AtHome;
                citizen.path = vec![home_tile];
                citizen.path_cursor = 0;
                citizen.step_work = 0.0;
                citizen.ready = true;
            }
            world.claim_tasks();
            // Stand every worker on the tile their task is on **now** — a gather's patch first,
            // then its site, which is the stage doing its job.
            let placements: Vec<(usize, u32)> = world
                .citizens
                .iter()
                .enumerate()
                .filter_map(|(index, citizen)| {
                    let id = citizen.task?;
                    let task = world.tasks.iter().find(|task| task.id == id)?;
                    Some((index, task.destination()))
                })
                .collect();
            for (index, tile) in placements {
                let citizen = &mut world.citizens[index];
                citizen.state = CitizenState::AtWork;
                citizen.path = vec![tile];
                citizen.path_cursor = 0;
            }
            world.work_tasks();
            if world.tasks.is_empty() {
                return;
            }
        }
    }

    /// The rung, end to end: the world's timber patch, its stone seam and its fibre are taken by
    /// hand, carried to the works, and turned into **one hatchet** — with every gram accounted
    /// for on the way. The chain's own number is the assertion: 4.1 kg of the world.
    #[test]
    fn the_stone_rung_makes_a_hatchet_out_of_the_world() {
        let mut world = rung_world();
        let site = world.works_site().expect("a road-reachable tile to work at");
        let posted = world
            .post_rung_tasks(site)
            .unwrap_or_else(|short| panic!("the rung must be payable: {}", short.describe()));
        assert!(posted.len() >= 3, "the plan has leaves to gather before anything is made");

        // Nothing is made before anything is gathered: a make is not even claimable until its site
        // holds what its process takes in. This is Q77's rule, and it is what makes the plan's
        // order emerge rather than having to be scheduled.
        let assemble = world
            .tasks
            .iter()
            .find(|task| task.process == "assemble the hatchet")
            .expect("the chain's last step is posted")
            .clone();
        assert!(!world.task_can_start(&assemble), "nothing is at the site yet");
        assert!(
            world.tasks.iter().any(|task| task.verb == sim_task::Verb::Gather && world.task_can_start(task)),
            "and the gathers are work from the first tick"
        );

        drive_the_rung(&mut world, 400);
        assert!(world.tasks.is_empty(), "the rung finished: {:?}", world.tasks.iter().map(|t| t.describe()).collect::<Vec<_>>());

        // The hatchet exists, at the works, and it is the good the chain names.
        let held = world.held_g(RUNG_GOOD);
        assert_eq!(held, 2900, "one hatchet is 2900 g of the world");

        // Everything the rung drew is still somewhere: 3000 g timber + 1000 g stone + 100 g fibre
        // is the chain's own 4.1 kg, and its by-products (offcuts, flakes, dust, trim) are
        // holdings rather than losses.
        let site_holding: i64 = world
            .holdings
            .entries()
            .filter(|(account, _)| account.starts_with(&material_ledger::site_prefix(site)))
            .map(|(_, grams)| grams)
            .sum();
        assert_eq!(site_holding, 4100, "the works is holding exactly what the world gave up");
        assert_eq!(
            world.held_g("timber_offcuts") + world.held_g("stone_flakes")
                + world.held_g("fibre_dust") + world.held_g("trim_waste"),
            1200,
            "every declared loss is still mass, in a holding, rather than gone"
        );

        // And the campaign's claim, with a rung behind it: the city is made of what it dug.
        let audit = world.mass_audit();
        assert!(audit.conserves(), "the audit is the whole point: {:#?}", audit.findings());
        assert_eq!(audit.held_g, site_holding, "the audit reads the holdings it accounts for");
        assert_eq!(audit.held_g, 4100);

        // The tree has been cut and the stone taken, and the ground says so.
        let ground: i64 = audit
            .holdings()
            .iter()
            .filter(|(account, _)| account.starts_with("ground:"))
            .map(|(_, grams)| *grams)
            .sum();
        assert!(ground <= -4100, "the ground gave up at least the rung's own 4.1 kg: {ground} g");

        // The city has a tool, so it does not post the rung again — the want is answered.
        assert!(!world.post_works_tasks(), "a city with a hatchet has no first rung to work");
    }

    /// A patch nobody can stand beside is a patch nobody gathers: with no road at all, the rung is
    /// refused **with the substance named**, and no work is posted.
    #[test]
    fn a_rung_the_road_cannot_reach_is_refused_with_its_substance_named() {
        let mut world = World::new(256, 256, 7);
        for tile in world.tiles.iter_mut() {
            tile.terrain = Terrain::Ground;
            tile.road = false;
        }
        world.rebuild_derived();

        let site = world.index(10, 10);
        // The world grows things, and none of them is reachable by hand: the roads are what
        // carrying needs (Q84).
        assert!(
            (0..world.tiles.len() as u32).any(|tile| world.surface_at(tile).is_some()),
            "the fixture should have patches, or it proves nothing about reach"
        );
        assert_eq!(world.plan_source(site, "timber"), None, "no road, no patch to stand by");

        let refusal = world
            .post_rung_tasks(site)
            .expect_err("a rung with no reachable leaf is refused");
        // The refusal names the **substance** the reduction asked for first, read from the
        // reduction rather than hardcoded here — the plan decides the order, and a test that
        // repeated it would be a second home for the chain.
        let plan = material_chain::plan(RUNG_GOOD, 2900).expect("the rung plans");
        let first = plan.leaves.first().expect("leaves to gather").substance;
        assert_eq!(refusal.family, first, "the refusal names a substance, not a family");
        assert!(refusal.note.contains("road"), "{}", refusal.note);
        assert!(world.tasks.is_empty(), "and no work is posted for it");

        // The world owns the road it seeds, so the material refusal above is what a **proposed
        // site** gets (Q87's tender is the caller that would propose one). The city's own posting
        // is the other refusal: with nothing to work on, it says so — and says it once.
        assert_eq!(world.works_site(), None, "a roadless world has nowhere to work");
        assert!(!world.post_works_tasks());
        let starved = world.starved();
        assert_eq!(starved.len(), 1, "one case, not one per tick: {starved:?}");
        assert_eq!(starved[0].family, "a works site");
        assert!(starved[0].note.contains("road"), "{}", starved[0].note);
        assert_eq!(world.starved().len(), 1, "and re-posting folds into it");

        // And the material refusal, which is the one a *site* would get.
        world.record_starved(&refusal);
        let starved = world.starved();
        assert_eq!(starved.len(), 2, "a different refusal is a different case: {starved:?}");
        assert_eq!(starved[1].substance, first);
        assert_eq!(
            starved[1].family,
            material_schema::substance(first).expect("a declared substance").family,
            "the family it presents as is still on the case"
        );
        assert!(starved[1].missing_g() > 0);
        assert!(starved[1].objective().contains(first), "{}", starved[1].objective());
    }

    /// A make refuses to run on material that is not there — the rule that makes a process a
    /// process rather than a formula. Q92's *refused until something changes*: the work waits with
    /// a case naming what is missing, instead of finishing into a holding that cannot pay.
    #[test]
    fn a_make_will_not_run_without_its_inputs_at_the_site() {
        let mut world = rung_world();
        let site = world.works_site().expect("a works site");
        world.post_rung_tasks(site).expect("the rung is payable");
        let assemble = world
            .tasks
            .iter()
            .find(|task| task.process == "assemble the hatchet")
            .expect("the last step")
            .clone();

        // Nobody is at the site, and somebody who stood there for the whole of the work would
        // still make nothing: the counter moves, and the mass does not, because the completion is
        // a check rather than a declaration.
        let worker = world.citizens[0].id;
        {
            let task = world.tasks.iter_mut().find(|task| task.id == assemble.id).expect("posted");
            task.claimed_by = Some(worker);
        }
        let index = world.citizens.iter().position(|c| c.id == worker).expect("the worker");
        world.citizens[index].state = CitizenState::AtWork;
        world.citizens[index].task = Some(assemble.id);
        world.citizens[index].path = vec![site];
        world.citizens[index].path_cursor = 0;
        for _ in 0..assemble.work_remaining {
            world.work_tasks();
        }

        assert!(!world.holds(RUNG_GOOD), "a hatchet cannot be made from an empty site");
        assert_eq!(world.held_g(RUNG_GOOD), 0);
        // The work is still real and still open: the case names what was missing.
        let task = world.tasks.iter().find(|task| task.id == assemble.id).expect("still open");
        assert!(task.claimed_by.is_none(), "and it went back to the queue");
        let starved = world.starved();
        assert!(
            starved.iter().any(|case| case.substance == "knapped_edge" || case.substance == "haft_blank" || case.substance == "cord"),
            "a case naming the input: {starved:?}"
        );
    }

    /// A gather is a **two-leg trip**: the worker takes the patch, and the load rides in the
    /// carrier's arms until it is delivered. The mass is the city's the whole way, which is what
    /// Q77's two accounts are for.
    #[test]
    fn a_gather_takes_the_patch_then_carries_it_to_the_site() {
        let mut world = rung_world();
        let site = world.works_site().expect("a works site");
        world.post_rung_tasks(site).expect("the rung is payable");

        let gather = world
            .tasks
            .iter()
            .find(|task| task.verb == sim_task::Verb::Gather && task.substance == "timber")
            .expect("a timber gather")
            .clone();
        let worker = world.citizens[0].id;
        {
            let task = world.tasks.iter_mut().find(|task| task.id == gather.id).expect("posted");
            task.claimed_by = Some(worker);
        }
        let index = world.citizens.iter().position(|c| c.id == worker).expect("the worker");
        world.citizens[index].task = Some(gather.id);
        world.citizens[index].state = CitizenState::AtWork;
        world.citizens[index].path = vec![gather.fetch_from.expect("a patch to take from")];
        world.citizens[index].path_cursor = 0;

        for _ in 0..gather.work_remaining {
            world.work_tasks();
        }

        // Taken, and in the carrier's arms rather than the site's: the first leg is done and the
        // second has not started.
        let carried = world.holding_of(&material_ledger::carried_account(worker, "timber"));
        assert_eq!(carried, 3000, "one armful, exactly as the gather declares");
        assert_eq!(world.holding_of(&material_ledger::site_account(site, "timber")), 0);
        let task = world.tasks.iter().find(|task| task.id == gather.id).expect("still open");
        assert!(task.is_delivering(), "and it is carrying now, not fetching");
        assert_eq!(task.destination(), site, "so the next leg is the site");

        // The patch remembers the take: the ground gave up exactly what was carried.
        let patch = gather.fetch_from.expect("a patch");
        assert!(world.tiles[patch as usize].harvested_g >= 3000);
        let audit = world.mass_audit();
        assert!(audit.conserves(), "a load in somebody's arms is still mass: {:#?}", audit.findings());
        assert_eq!(audit.held_g, 3000, "and the audit reads it as held");

        // Delivered: it stands at the site now, and the task is done.
        world.citizens[index].path = vec![site];
        world.citizens[index].path_cursor = 0;
        world.work_tasks();
        assert_eq!(world.holding_of(&material_ledger::carried_account(worker, "timber")), 0);
        assert_eq!(world.holding_of(&material_ledger::site_account(site, "timber")), 3000);
        assert!(
            !world.tasks.iter().any(|task| task.id == gather.id),
            "the gather retired itself"
        );
        assert!(world.mass_audit().conserves());
    }

    /// Q108 in the world rather than on paper: a patch that was taken from comes back as sim-days
    /// pass, so the ground's depletion is a function of elapsed **time** and the audit stays true.
    #[test]
    fn a_harvested_patch_grows_back_on_the_worlds_own_clock() {
        let mut world = rung_world();
        let patch = (0..world.tiles.len() as u32)
            .find(|index| {
                world
                    .surface_at(*index)
                    .is_some_and(|standing| standing.substance == "timber")
            })
            .expect("a stand");
        let before = world.surface_at(patch).expect("a patch to start from").mass_g;
        let taken = world.harvest(patch, 100_000);
        assert!(taken > 0, "the stand gives up what it holds");
        let after = world.surface_at(patch).expect("still standing").mass_g;
        assert_eq!(after, before - taken, "a take is what leaves the patch, to the gram");

        // A year of sim-days, and it has grown back — brush in weeks, stands in years, and both by
        // the same declared mechanism.
        world.clock.tick += TICKS_PER_DAY * 365;
        let regrown = world.surface_at(patch).expect("still standing").mass_g;
        assert!(regrown > after, "a patch that is not stripped grows back: {after} → {regrown}");
        assert!(regrown <= before, "and never past what the tile holds: {regrown} vs {before}");

        // Regrowth does not un-take the mass: the ground's account is the **take total**, so a
        // city holding what it cut does not read as material nobody dug up.
        let audit = world.mass_audit();
        assert!(
            audit.conserves(),
            "regrowth is not a negative take: {:#?}",
            audit.findings()
        );
        assert_eq!(
            audit.held_g, 0,
            "nothing is held here — the wood was never carried off, so it is not a holding"
        );
    }

    /// The rung through the **real tick**: nobody teleports, the citizens walk the roads, and the
    /// hatchet turns up because the world produced it rather than because a test moved mass.
    ///
    /// The driver test above proves the mass path; this one proves the machinery around it — the
    /// claim pass, the two-leg routing, the delivery and the completion — which is the half a
    /// teleport would hide.
    #[test]
    fn the_city_walks_out_and_makes_a_hatchet_on_its_own() {
        let mut world = rung_world();
        let mut ticks = 0;
        // Measured rather than guessed: the rung lands between 1 000 and 2 000 ticks on this
        // fixture — a day and a half of sim-time, most of it walking — and the bound is set with
        // headroom so a change in the walking rate announces itself as a failure here. The ceiling
        // is not a target and nothing is tuned to it.
        while !world.holds(RUNG_GOOD) && ticks < 2_500 {
            world.tick();
            ticks += 1;
        }
        assert!(
            world.holds(RUNG_GOOD),
            "2 500 ticks and no hatchet: tasks {:?}, starved {:?}, holdings {:?}",
            world.tasks.iter().map(|task| task.describe()).collect::<Vec<_>>(),
            world.starved(),
            world.holdings.holdings()
        );
        assert!(
            world.mass_audit().conserves(),
            "and the audit is true of it: {:#?}",
            world.mass_audit().findings()
        );
    }

    /// A refusal to build is a **case with a number**, not a silence: on a world with no ceramic
    /// at all, no work is posted, and the shortage is recorded in the shape the case engine reads.
    #[test]
    fn a_starved_world_posts_no_work_and_records_the_shortage() {
        let mut world = small_city();
        world.place_building(world.index(1, 3), BuildingKind::PowerPlant).expect("plant");
        world.buildings[0].ready_tick = 0;
        let site = world.index(1, 6);
        world.set_zone(site, Zone::Residential);
        world.recompute_power();
        world.recompute_demand();
        world.post_demand_tasks();

        assert!(
            world.open_task_at(site).is_none(),
            "the world cannot supply a ceramic home, so no work is posted"
        );
        let starved = world.starved();
        assert_eq!(starved.len(), 1, "one reading for the district, not one per tick");
        assert!(starved[0].family.contains("ceramic"), "{}", starved[0].objective());
        assert_eq!(starved[0].found_g, 0, "the world holds none of it");
        assert!(starved[0].missing_g() > 0);
        assert!(starved[0].objective().contains("short"), "{}", starved[0].objective());

        // Filing twice is still one reading: a refusal repeated every tick must not become a
        // thousand cases.
        world.post_demand_tasks();
        assert_eq!(world.starved().len(), 1);
        assert_eq!(world.starved()[0].count, 2, "and the count is real, not decorative");
    }

    #[test]
    fn construction_finishes_before_a_home_is_occupied() {
        let mut world = small_city();
        let tile = world.index(4, 4);
        world.place_building(tile, BuildingKind::Home).expect("home");
        world.buildings[0].ready_tick = 100;
        world.clock.tick = 50;
        world.assign_jobs();
        assert_eq!(world.citizens.len(), 0, "no residents before the building is ready");
        world.clock.tick = 120;
        world.assign_jobs();
        assert!(!world.citizens.is_empty(), "residents move in once it is ready");
    }

    #[test]
    fn income_posts_once_a_month() {
        let mut world = small_city();
        let tile = world.index(4, 4);
        world.place_building(tile, BuildingKind::Home).expect("home");
        world.buildings[0].ready_tick = 0;
        world.recompute_power();
        world.buildings[0].powered = true;
        world.tiles[tile as usize].powered = true;
        let before = world.economy.credits;
        world.clock.tick = TICKS_PER_DAY * DAYS_PER_MONTH;
        world.post_month();
        assert!(
            world.economy.month_income > 0,
            "an occupied, powered home pays tax"
        );
        // One home does not cover thirty-two tiles of road upkeep. This asserted
        // the opposite before the test was corrected: it was checking a feeling
        // about empty cities rather than the arithmetic.
        assert!(
            world.economy.month_expense > world.economy.month_income,
            "road upkeep on a nearly empty city should exceed its tax take"
        );
        assert_eq!(
            world.economy.credits,
            before + world.economy.month_income - world.economy.month_expense
        );
    }

    // -----------------------------------------------------------------
    // The material claim and the versioned save (a152, a155, a175)
    // -----------------------------------------------------------------

    /// A named field removed from a compact RON document, with its separator.
    ///
    /// Written rather than hand-edited so the v1 fixture in the tests below is a real
    /// save with one field taken out, instead of a string somebody typed: a fixture that
    /// is not the thing it claims to be tests nothing.
    fn strip_field(text: &str, field: &str) -> String {
        let needle = format!("{field}:");
        let Some(start) = text.find(&needle) else {
            return text.to_string();
        };
        // A field is removed with the separator that joined it to its neighbour: the comma
        // **before** it for a last field (`...,condition:1.0)`), the comma after its value
        // for any other. Getting this wrong leaves text that does not parse, which is how
        // the first version of this helper announced itself.
        let mut depth = 0usize;
        let mut in_string = false;
        let mut found = None;
        for (offset, ch) in text[start..].char_indices() {
            match ch {
                '"' => in_string = !in_string,
                '(' | '[' if !in_string => depth += 1,
                ')' | ']' if !in_string => {
                    if depth == 0 {
                        found = Some((start + offset, false));
                        break;
                    }
                    depth -= 1;
                }
                ',' if !in_string && depth == 0 => {
                    found = Some((start + offset + 1, true));
                    break;
                }
                _ => {}
            }
        }
        let Some((end, ended_at_comma)) = found else {
            return text.to_string();
        };
        // Exactly one separator goes with the field: the trailing comma when the value had
        // one, the leading comma when it did not (a last field ends at the closing paren).
        let cut_from = if ended_at_comma || !text[..start].ends_with(',') {
            start
        } else {
            start - 1
        };
        format!("{}{}", &text[..cut_from], &text[end..])
    }

    /// A save as v1 wrote it: the two fields this version added, taken back out.
    fn as_v1(world: &World) -> String {
        let mut text = ron::ser::to_string(world).expect("serialise");
        text = strip_field(&text, "format_version");
        let buildings = world.buildings.len();
        for _ in 0..buildings {
            text = strip_field(&text, "material_as_built");
            text = strip_field(&text, "condition");
        }
        assert!(!text.contains("format_version"), "the fixture still carries a version");
        assert!(!text.contains("material_as_built"));
        text
    }

    /// The claim is read off the declared mapping, not written here: if a family is
    /// renamed in `world::PARTS`, this test fails rather than the world drifting.
    #[test]
    fn a_structure_claims_the_body_material_its_kind_declares() {
        for kind in [
            BuildingKind::Home,
            BuildingKind::Shop,
            BuildingKind::Factory,
            BuildingKind::PowerPlant,
        ] {
            let claim = MaterialClaim::of_kind(kind).expect("every kind has a declared body");
            let part = material_world::part(&claim.part)
                .unwrap_or_else(|| panic!("{} names an undeclared part", claim.part));
            assert_eq!(part.family.as_str(), claim.family, "{} family", kind.name());
            assert_eq!(part.hue.as_str(), claim.anchor, "{} anchor", kind.name());
            assert_eq!(part.level.as_str(), claim.level, "{} level", kind.name());
            assert_eq!(claim.level, "body", "the claim is the body part");
            assert!(claim.part.starts_with(kind.part_prefix()));
        }
    }

    #[test]
    fn a_placed_structure_carries_its_claim_and_keeps_it_through_a_save() {
        let mut world = small_city();
        let tile = world.index(4, 4);
        let id = world.place_building(tile, BuildingKind::Home).expect("placed");
        let placed = world.building_on(tile).expect("the structure is on its tile");
        assert_eq!(placed.id, id, "the id is stable, the index is not");
        let claim = placed
            .material_as_built
            .clone()
            .expect("a placed structure claims its material");
        assert_eq!(claim.part, "home.walls");
        assert_eq!(world.building_on(tile).expect("placed").condition, 1.0);

        let dir = std::env::temp_dir().join("ala-cities-test-material-claim");
        let path = dir.join("world.ron");
        world.save(&path).expect("save");
        let loaded = World::load(&path).expect("load");
        assert!(
            loaded.migration.is_none(),
            "a save this build wrote needs no migration"
        );
        assert_eq!(
            loaded.building_on(tile).expect("loaded").material_as_built.as_ref(),
            Some(&claim),
            "the as-built claim is part of the record"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_v1_save_migrates_with_a_report_naming_what_it_derived() {
        let mut world = small_city();
        for (tile, kind) in [
            (world.index(4, 4), BuildingKind::Home),
            (world.index(5, 4), BuildingKind::Home),
            (world.index(6, 4), BuildingKind::Factory),
        ] {
            world.place_building(tile, kind).expect("placed");
        }
        let dir = std::env::temp_dir().join("ala-cities-test-migration");
        let path = dir.join("world-v1.ron");
        std::fs::create_dir_all(&dir).expect("dir");
        std::fs::write(&path, as_v1(&world)).expect("write v1");

        let loaded = World::load(&path).expect("a v1 save loads and migrates");
        assert_eq!(loaded.format_version, FORMAT_VERSION);
        let migration = loaded.migration.as_ref().expect("the derivation is reported");
        assert_eq!((migration.from, migration.to), (0, FORMAT_VERSION));
        assert_eq!(
            migration.derived.len(),
            3,
            "every structure that predated the claim got one derived"
        );
        let described = migration.describe();
        assert!(described.contains("2 × ceramic/natural/body on home.walls"), "{described}");
        assert!(described.contains("metal/natural/body on factory.frame"), "{described}");
        assert!(described.contains("derivation, not a measurement"), "{described}");
        for building in &loaded.buildings {
            assert_eq!(
                building.material_as_built,
                MaterialClaim::of_kind(building.kind),
                "the derived claim is the kind's own declared body material"
            );
            assert_eq!(building.condition, 1.0);
        }
        // Migrated once, then saved as v2, it is no longer a v1 save.
        let saved = dir.join("world-v2.ron");
        loaded.save(&saved).expect("save migrated");
        let again = World::load(&saved).expect("reload");
        assert!(again.migration.is_none(), "a v2 save is not migrated again");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&saved);
    }

    /// Two refusals, both a152's: inventing state for a newer save, and patching over a
    /// current save that is missing the thing the current format is for.
    #[test]
    fn a_save_this_build_cannot_read_is_refused_rather_than_guessed() {
        let mut world = small_city();
        world.place_building(world.index(4, 4), BuildingKind::Home).expect("placed");

        let mut from_the_future = ron::ser::to_string(&world).expect("serialise");
        from_the_future = from_the_future.replacen(
            &format!("format_version:{FORMAT_VERSION}"),
            &format!("format_version:{}", FORMAT_VERSION + 1),
            1,
        );
        let dir = std::env::temp_dir().join("ala-cities-test-migration");
        std::fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("world-newer.ron");
        std::fs::write(&path, from_the_future).expect("write");
        let err = World::load(&path).expect_err("a newer save is refused");
        assert!(
            err.to_string().contains("invented state"),
            "the refusal names the reason: {err}"
        );

        let missing = strip_field(
            &ron::ser::to_string(&world).expect("serialise"),
            "material_as_built",
        );
        std::fs::write(&path, missing).expect("write");
        let err = World::load(&path).expect_err("a v2 save missing a claim is refused");
        assert!(
            err.to_string().contains("no as-built material"),
            "the refusal names the defect: {err}"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// The frame's colours are checked without a window: what is drawn is the table's own
    /// entry for the claim, and a ruin is the same family read one level down. A renderer
    /// rule nobody can run in a test is a rule that rots, so the rule lives here and the
    /// frame only calls it.
    #[test]
    fn a_structure_is_drawn_in_its_claimed_material_and_a_ruin_one_level_down() {
        let mut world = small_city();
        // A plant refuses a tile with no road in reach, so the fixture builds one: a test
        // that has to reach for a workaround to place a structure is testing the
        // workaround.
        world.lay_road(world.index(3, 3), world.index(3, 9));
        for (offset, kind) in [
            BuildingKind::Home,
            BuildingKind::Shop,
            BuildingKind::Factory,
            BuildingKind::PowerPlant,
        ]
        .into_iter()
        .enumerate()
        {
            let tile = world.index(4, 4 + offset as u32);
            world.place_building(tile, kind).expect("placed");
            let building = world.building_on(tile).expect("placed");
            let claim = building.material_as_built.clone().expect("claim");
            let drawn = building.drawn_colour().expect("the table resolves the claim");
            let entry = crate::materials::material_of(&claim.family, &claim.anchor, &claim.level)
                .unwrap_or_else(|| panic!("{} claims an unresolved material", kind.name()));
            assert_eq!(&drawn[..3], &entry.rgb[..], "{}", kind.name());
            assert_eq!(drawn[3], 1.0, "a standing structure is opaque");
            let ruin = building.ruin_colour(0.25).expect("ruin colour");
            assert_eq!(ruin[3], 0.25, "a ruin is drawn faint");
            assert_eq!(
                &ruin[..3],
                &crate::materials::material_of(&claim.family, &claim.anchor, "deep")
                    .expect("every family resolves at deep")
                    .rgb[..],
                "a ruin keeps the family and reads one level down"
            );
            // The point of the change: two kinds must not be the same colour merely
            // because they share a zone-shaped token.
            assert_ne!(drawn, ruin, "a ruin is not the as-built colour");
        }
    }

    #[test]
    fn a_mat_claim_is_read_back_from_the_world_and_a_mismatch_is_named() {
        use crate::gov::{check_expectation, Expectation};
        let mut world = small_city();
        let tile = world.index(4, 4);
        world.place_building(tile, BuildingKind::Home).expect("placed");
        let claim = world
            .building_on(tile)
            .expect("placed")
            .material_as_built
            .clone()
            .expect("claim");
        let claimed = Expectation::MaterialOf {
            tile,
            part: claim.part.clone(),
            family: claim.family.clone(),
            anchor: claim.anchor.clone(),
            level: claim.level.clone(),
        };
        assert!(
            check_expectation(&world, &claimed).is_ok(),
            "the world shows the material its MAT-* ticket claimed"
        );
        assert_eq!(claimed.describe(), format!("the home.walls on tile {tile} is ceramic/natural/body"));
        assert_eq!(claimed.tiles(), vec![tile]);

        // A claim that names another material is a finding that says which.
        let wrong = Expectation::MaterialOf {
            tile,
            part: "home.walls".to_string(),
            family: "metal".to_string(),
            anchor: "natural".to_string(),
            level: "body".to_string(),
        };
        let err = check_expectation(&world, &wrong).expect_err("a mismatch is a finding");
        assert!(err.contains("is ceramic/natural/body"), "{err}");
        assert!(err.contains("metal/natural/body"), "{err}");

        // No structure at all is not a pass: there is nothing to read back.
        let empty = world.index(9, 9);
        let err = check_expectation(
            &world,
            &Expectation::MaterialOf {
                tile: empty,
                part: "home.walls".to_string(),
                family: "ceramic".to_string(),
                anchor: "natural".to_string(),
                level: "body".to_string(),
            },
        )
        .expect_err("a claim about an empty tile cannot be read back");
        assert!(err.contains("no structure stands"), "{err}");
    }

    #[test]
    fn a_save_round_trips_and_rebuilds_the_graph() {
        let mut world = small_city();
        world.lay_road(world.index(3, 3), world.index(9, 3));
        world.place_building(world.index(3, 4), BuildingKind::PowerPlant).expect("plant");
        let dir = std::env::temp_dir().join("ala-cities-test-save");
        let path = dir.join("world.ron");
        world.save(&path).expect("save");
        let loaded = World::load(&path).expect("load");
        assert_eq!(loaded.buildings.len(), world.buildings.len());
        assert_eq!(
            loaded.roads.len(),
            world.roads.len(),
            "the road graph must be rebuilt on load, not serialised"
        );
        assert!(loaded.has_road_access(loaded.index(3, 5)));
        let _ = std::fs::remove_file(&path);
    }

    /// Ignored by default: it writes a few megabytes of RON to a temporary
    /// directory to answer one question — what an autosave actually costs at
    /// the shipped map size. Run it when the save format changes.
    ///
    ///     cargo test --lib measure_world_save_cost -- --ignored --nocapture
    #[test]
    #[ignore = "measurement, not an invariant"]
    fn measure_world_save_cost() {
        use std::time::Instant;

        let mut world = World::new(256, 256, 0xC117_2026);
        for x in 2..120u32 {
            world.lay_road(world.index(x, 2), world.index(x, 3));
        }
        for i in 0..400u32 {
            let tile = world.index(2 + i % 100, 6 + i / 100);
            let _ = world.place_building(tile, BuildingKind::Home);
        }
        let dir = std::env::temp_dir().join("ala-cities-measure-save");
        let path = dir.join("world.ron");

        let started = Instant::now();
        world.save(&path).expect("save");
        let wrote = started.elapsed();
        let bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        let started = Instant::now();
        let loaded = World::load(&path).expect("load");
        let read = started.elapsed();

        println!(
            "save: {} bytes in {:.1} ms · load: {:.1} ms · buildings {}",
            bytes,
            wrote.as_secs_f64() * 1000.0,
            read.as_secs_f64() * 1000.0,
            loaded.buildings.len()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_save_leaves_no_temporary_behind() {
        // The rename is what makes an autosave safe to interrupt. If the
        // temporary file survives, that is the signal the rename did not happen
        // and the destination is stale rather than new.
        let mut world = small_city();
        world.lay_road(world.index(3, 3), world.index(9, 3));
        let dir = std::env::temp_dir().join("ala-cities-test-atomic");
        let path = dir.join("world.ron");
        world.save(&path).expect("save");
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        assert!(
            !std::path::Path::new(&tmp).exists(),
            "a completed save must not leave its temporary file behind"
        );
        let loaded = World::load(&path).expect("load");
        assert_eq!(loaded.roads.len(), world.roads.len());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_tick_is_deterministic_from_a_seed() {
        let mut a = small_city();
        let mut b = small_city();
        for world in [&mut a, &mut b] {
            world.place_building(world.index(1, 3), BuildingKind::PowerPlant);
            world.set_zone(world.index(1, 5), Zone::Residential);
            world.set_zone(world.index(1, 7), Zone::Commercial);
            world.buildings[0].ready_tick = 0;
        }
        for _ in 0..600 {
            a.tick();
            b.tick();
        }
        assert_eq!(a.stats.population, b.stats.population);
        assert_eq!(a.buildings.len(), b.buildings.len());
        assert_eq!(a.economy.credits, b.economy.credits);
        assert_eq!(a.stats.jobs, b.stats.jobs);
    }
}
