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
use crate::materials::effects as material_effects;
use crate::materials::geology as material_geology;
use crate::materials::ledger as material_ledger;
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

/// A refusal to build, because the world could not supply the mass.
///
/// Round 11's Q74 answered that a refusal **files a case** rather than passing quietly: the
/// shortfall is a thing that happened to the world, and the world needs to be able to say what
/// it wanted, what it found, and from what kind of material. This type is the claim; the case is
/// filed by whoever asked — and if nobody asks, nothing was built either way.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialShortfall {
    pub site: u32,
    pub family: String,
    pub wanted_g: i64,
    pub found_g: i64,
}

/// Where the city is short of material, aggregated by district and family — the reading a case
/// is sampled from (round 11's Q74).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Starved {
    pub district: u32,
    pub family: String,
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
        format!("materials:{}:{}", self.district, self.family)
    }

    /// What a person reads. Round 11's Q74 asked for a refusal that says what it wanted and what
    /// it found; round 5's Q36 said the government owns what nobody owns, so the objective names
    /// the shortage as work rather than as a complaint.
    pub fn objective(&self) -> String {
        format!(
            "{} build site(s) in district {} want {} g of {} and the world holds {} g: {} g short",
            self.count,
            self.district,
            self.wanted_g,
            self.family,
            self.found_g,
            self.missing_g()
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
        // that produced it, so the substance cannot be remembered wrongly.
        for (index, tile) in self.tiles.iter().enumerate() {
            if tile.extracted_g == 0 {
                continue;
            }
            let (x, y) = (index as u32 % self.width, index as u32 / self.width);
            let Some(deposit) = material_geology::deposit(self.seed, x, y) else {
                // Taken from a tile whose deposit no longer derives: the take is still real, so
                // it is recorded against the ground itself rather than dropped.
                ledger.record(&material_ledger::ground_account("unattributed"), -tile.extracted_g);
                extracted_g += tile.extracted_g;
                continue;
            };
            let substance = material_geology::DEPOSIT_KINDS[deposit.kind].substance;
            ledger.record(&material_ledger::ground_account(substance), -tile.extracted_g);
            extracted_g += tile.extracted_g;
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

        material_ledger::MassAudit { ledger, extracted_g, standing_g, defects }
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
        let shortfall = |found_g: i64| MaterialShortfall {
            site,
            family: family.to_string(),
            wanted_g: need_g,
            found_g,
        };
        if need_g <= 0 {
            return Ok(Vec::new());
        }

        let (sx, sy) = self.coords(site);
        let mut candidates: Vec<(i64, u32, &'static str)> = Vec::new();
        for index in 0..self.tiles.len() as u32 {
            let Some(deposit) = self.deposit_at(index) else {
                continue;
            };
            let kind = &material_geology::DEPOSIT_KINDS[deposit.kind];
            if kind.family != family {
                continue;
            }
            let (x, y) = (index % self.width, index / self.width);
            let (dx, dy) = (
                x as i64 - sx as i64,
                y as i64 - sy as i64,
            );
            candidates.push((dx * dx + dy * dy, index, kind.substance));
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
        let built = self.place_building(tile, kind).ok_or_else(|| MaterialShortfall {
            site: tile,
            family: family.clone(),
            wanted_g: mass,
            found_g: 0,
        })?;
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
            claimed_by: None,
            opened_tick: self.clock.tick,
        });
        Ok(id)
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
        let district = crate::gov::district_of(self.width, shortfall.site);
        if let Some(existing) = self
            .starved
            .iter_mut()
            .find(|s| s.district == district && s.family == shortfall.family)
        {
            existing.count += 1;
            existing.wanted_g += shortfall.wanted_g;
            existing.found_g += shortfall.found_g;
            return;
        }
        self.starved.push(Starved {
            district,
            family: shortfall.family.clone(),
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
                let (tx, ty) = (task.site % self.width, task.site / self.width);
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
        let width = self.width;
        let mut finished: Vec<u32> = Vec::new();
        let mut shortfalls: Vec<MaterialShortfall> = Vec::new();

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
            let (sx, sy) = (task.site % width, task.site / width);
            let (cx, cy) = (standing % width, standing / width);
            if (sx as i64 - cx as i64).abs() > 1 || (sy as i64 - cy as i64).abs() > 1 {
                // Standing somewhere else is not working on it.
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
                    // Either the ground no longer covers the plan or the site stopped being
                    // buildable. Both are refusals with a number, and the case engine reads them.
                    shortfalls.push(MaterialShortfall {
                        site: task.site,
                        family: task.family.clone(),
                        wanted_g: task.requires_g,
                        found_g: drawn_g,
                    });
                }
            }
            self.tasks.remove(position);
            for citizen in self.citizens.iter_mut() {
                if citizen.task == Some(id) {
                    citizen.task = None;
                    citizen.state = CitizenState::ToHome;
                    // Re-planned from where they stand, so a finished worker walks home.
                }
            }
        }

        for shortfall in shortfalls {
            self.record_starved(&shortfall);
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
                    (from_tile, task.site, CitizenState::ToWork)
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
            let route = self
                .roads
                .route_between_tiles(self.width, self.height, origin, to_tile, 3);
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
                if material_geology::DEPOSIT_KINDS[deposit.kind].family == family {
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
            let kind = &material_geology::DEPOSIT_KINDS[deposit.kind];
            assert_eq!(
                kind.family, claim.family,
                "tile {} is {} ({}), not {}",
                line.tile, kind.substance, kind.family, claim.family
            );
            assert_eq!(line.substance, kind.substance, "and the line names what is there");
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
