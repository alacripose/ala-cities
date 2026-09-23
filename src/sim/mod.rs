//! The city.
//!
//! Everything here is a pure function of `(seed, tick, input events)`. No wall
//! clock, no floating-point randomness, no hidden global state: the headless
//! verifier re-runs this exact code with no window attached, and a save
//! replays to the same city.

pub mod citizen;
pub mod road;
pub mod rng;
pub mod terrain;

use serde::{Deserialize, Serialize};

use crate::gov::RetirementReason;
use crate::materials::world::{self as material_world, Level};
use citizen::{Citizen, CitizenState};
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
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            terrain: Terrain::Ground,
            zone: Zone::None,
            road: false,
            building: None,
            powered: false,
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
}

impl Building {
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
            // Every structure carries its claim, whoever placed it: the world state is
            // what `verify.exe` re-reads, and a structure without a claim would be
            // unverifiable rather than merely unrecorded. The `MAT-*` *ticket* is filed
            // where placement is a recorded act (the player's build), which is the same
            // distinction the record already makes between a claim and growth.
            material_as_built: MaterialClaim::of_kind(kind),
            condition: condition_as_built(),
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
            self.grow();
        }
        if self.clock.tick.is_multiple_of(16) {
            self.assign_jobs();
            self.recount();
        }
        if self.clock.closes_month() {
            self.post_month();
        }

        self.step_citizens();
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

    /// Let zoned, road-connected, powered tiles grow. Growth is simulation, not
    /// a player action, so it files no ticket -- but the buildings it creates
    /// are the population the cases are sampled from.
    fn grow(&mut self) {
        if self.stats.brownout {
            // No power, no growth. Silently building anyway would be a lie the
            // player has no way to see.
            return;
        }
        let mut created = 0;
        for tile in 0..self.tiles.len() as u32 {
            if created >= 4 {
                break;
            }
            let t = self.tiles[tile as usize];
            if t.zone == Zone::None || t.building.is_some() || t.road || !t.powered {
                continue;
            }
            if !self.has_road_access(tile) {
                continue;
            }
            let demand = self.demand.for_zone(t.zone);
            if demand < 0.20 {
                continue;
            }
            let kind = match t.zone {
                Zone::Residential => BuildingKind::Home,
                Zone::Commercial => BuildingKind::Shop,
                Zone::Industrial => BuildingKind::Factory,
                Zone::None => continue,
            };
            if self.place_building(tile, kind).is_some() {
                created += 1;
                // Spending the demand is what stops one busy tick from zoning
                // an entire map at once.
                match t.zone {
                    Zone::Residential => self.demand.residential = (self.demand.residential - 0.12).max(0.0),
                    Zone::Commercial => self.demand.commercial = (self.demand.commercial - 0.12).max(0.0),
                    Zone::Industrial => self.demand.industrial = (self.demand.industrial - 0.12).max(0.0),
                    Zone::None => {}
                }
            }
        }
    }

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
            let (state, work, home, needs_plan, ready) = {
                let c = &self.citizens[index];
                // A parked agent (a one-tile path) is due a new plan; an agent
                // mid-route is not.
                (c.state, c.work, c.home, c.path.len() <= 1, c.ready)
            };
            if !needs_plan || !ready {
                continue;
            }
            let Some(work) = work else { continue };
            let (Some(home), Some(job)) = (home, Some(work)) else {
                continue;
            };
            let from_tile = self.buildings[home as usize].tile;
            let to_tile = self.buildings[job as usize].tile;
            let route = match state {
                CitizenState::AtHome => {
                    self.roads.route_between_tiles(self.width, self.height, from_tile, to_tile, 3)
                }
                CitizenState::AtWork => {
                    self.roads.route_between_tiles(self.width, self.height, to_tile, from_tile, 3)
                }
                _ => None,
            };
            planned += 1;
            match route {
                Some(path) => {
                    let citizen = &mut self.citizens[index];
                    citizen.path = path;
                    citizen.path_cursor = 0;
                    citizen.step_work = 0.0;
                    citizen.state = match state {
                        CitizenState::AtHome => CitizenState::ToWork,
                        CitizenState::AtWork => CitizenState::ToHome,
                        other => other,
                    };
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

    #[test]
    fn zoned_tiles_grow_only_with_power_and_a_road() {
        let mut world = small_city();
        world.place_building(world.index(1, 3), BuildingKind::PowerPlant).expect("plant");
        world.buildings[0].ready_tick = 0;
        // Beside the seed road: growth expected.
        let near = world.index(1, 6);
        world.set_zone(near, Zone::Residential);
        // In the far corner with no road: growth must not happen.
        let far = world.index(31, 31);
        world.set_zone(far, Zone::Residential);
        world.recompute_power();
        for _ in 0..40 {
            world.clock.tick += 1;
            world.recompute_demand();
            world.grow();
        }
        assert!(
            world.tiles[near as usize].building.is_some(),
            "a powered, road-connected zoned tile should grow"
        );
        assert!(
            world.tiles[far as usize].building.is_none(),
            "an unpowered tile with no road must not grow"
        );
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
