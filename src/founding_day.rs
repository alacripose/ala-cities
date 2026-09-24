//! Target founding-day systems.
//!
//! This module is intentionally separate from the legacy flat C1 runtime. It is
//! the first executable contract for the target world: signed coordinates,
//! deterministic sparse chunks, an authoritative clock, conserved founding
//! materials, and evidence-producing actions.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const CHUNK_SIZE: i32 = 32;
pub const FIRST_BAND_MIN_Z: i32 = -16;
pub const FIRST_BAND_MAX_Z: i32 = 47;
pub const TICKS_PER_SECOND: u64 = 20;
pub const DAY_TICKS: u64 = 24 * 60 * TICKS_PER_SECOND;
pub const NIGHT_TICKS: u64 = 12 * 60 * TICKS_PER_SECOND;
const FORAGE_TAKE_GRAMS: u64 = 500;
const WOOD_TAKE_GRAMS: u64 = 2_500;
const FIRE_WOOD_GRAMS: u64 = 1_000;
const SHELTER_WOOD_GRAMS: u64 = 1_500;
const LAND_TICKS: u64 = 0;
const FORAGE_TICKS: u64 = 60;
const WOOD_TICKS: u64 = 120;
const FIRE_TICKS: u64 = 180;
const SHELTER_TICKS: u64 = 300;
const FORAGE_PATCH_GRAMS: u64 = 2_000;
const WOOD_PATCH_GRAMS: u64 = 5_000;

/// A signed integer world coordinate. The first implementation deliberately
/// uses integer coordinates; continuous fields are a later typed layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl fmt::Display for Coord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "({},{},{})", self.x, self.y, self.z)
    }
}

impl Coord {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn chunk(self) -> ChunkCoord {
        ChunkCoord {
            x: self.x.div_euclid(CHUNK_SIZE),
            y: self.y.div_euclid(CHUNK_SIZE),
            z: self.z.div_euclid(CHUNK_SIZE),
        }
    }

    pub fn is_in_first_band(self) -> bool {
        (FIRST_BAND_MIN_Z..=FIRST_BAND_MAX_Z).contains(&self.z)
    }

    const fn local_in_chunk(self) -> Self {
        Self {
            x: self.x.rem_euclid(CHUNK_SIZE),
            y: self.y.rem_euclid(CHUNK_SIZE),
            z: self.z.rem_euclid(CHUNK_SIZE),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorldSeed(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GeneratorRevision(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VoxelKind {
    Soil,
    Forage,
    Wood,
}

impl fmt::Display for VoxelKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl VoxelKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Soil => "soil",
            Self::Forage => "forage",
            Self::Wood => "wood",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Voxel {
    pub kind: VoxelKind,
    pub mass_grams: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChunkDelta {
    RemoveVoxelMass { coord: Coord, grams: u64 },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Chunk {
    pub voxels: BTreeMap<Coord, Voxel>,
}

impl Chunk {
    pub fn derive(seed: WorldSeed, revision: GeneratorRevision, chunk: ChunkCoord) -> Self {
        let mut voxels = BTreeMap::new();
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let local = Coord::new(x, y, z);
                    let coord = Coord::new(
                        chunk.x * CHUNK_SIZE + x,
                        chunk.y * CHUNK_SIZE + y,
                        chunk.z * CHUNK_SIZE + z,
                    );
                    let kind = derive_kind(seed, revision, coord, local);
                    let mass = match kind {
                        VoxelKind::Soil => 1_000,
                        VoxelKind::Forage => FORAGE_PATCH_GRAMS,
                        VoxelKind::Wood => WOOD_PATCH_GRAMS,
                    };
                    voxels.insert(
                        local,
                        Voxel {
                            kind,
                            mass_grams: mass,
                        },
                    );
                }
            }
        }
        Self { voxels }
    }

    fn apply_delta(&mut self, chunk_coord: ChunkCoord, delta: ChunkDelta) {
        let ChunkDelta::RemoveVoxelMass { coord, grams } = delta;
        debug_assert_eq!(coord.chunk(), chunk_coord);
        let local = coord.local_in_chunk();
        let Some(voxel) = self.voxels.get_mut(&local) else {
            return;
        };
        voxel.mass_grams = voxel
            .mass_grams
            .checked_sub(grams)
            .expect("typed voxel delta exceeds the derived mass");
        if voxel.mass_grams == 0 {
            self.voxels.remove(&local);
        }
    }
}

fn hash_coord(seed: WorldSeed, revision: GeneratorRevision, coord: Coord) -> u64 {
    let mut value = seed.0
        ^ revision.0.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (coord.x as i64 as u64).rotate_left(17)
        ^ (coord.y as i64 as u64).rotate_left(29)
        ^ (coord.z as i64 as u64).rotate_left(43);
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn derive_kind(
    seed: WorldSeed,
    revision: GeneratorRevision,
    coord: Coord,
    local: Coord,
) -> VoxelKind {
    // The first slice has visible starter resources so the actual material path
    // can be exercised without inventing a second inventory system.
    if coord == Coord::new(4, 4, 0) {
        return VoxelKind::Soil;
    }
    if coord == Coord::new(2, 2, 0) {
        return VoxelKind::Forage;
    }
    if coord == Coord::new(6, 2, 0) || coord == Coord::new(2, 6, 0) {
        return VoxelKind::Wood;
    }
    if local.z != 0 {
        return VoxelKind::Soil;
    }
    match hash_coord(seed, revision, coord) % 23 {
        0 | 1 => VoxelKind::Forage,
        2 | 3 => VoxelKind::Wood,
        _ => VoxelKind::Soil,
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MaterialLedger {
    source: BTreeMap<VoxelKind, u64>,
    world: BTreeMap<VoxelKind, u64>,
    carried: BTreeMap<VoxelKind, u64>,
    spent: BTreeMap<VoxelKind, u64>,
}

impl MaterialLedger {
    fn add_source(&mut self, kind: VoxelKind, grams: u64) {
        *self.source.entry(kind).or_default() += grams;
    }

    fn add_world(&mut self, kind: VoxelKind, grams: u64) {
        *self.world.entry(kind).or_default() += grams;
    }

    fn take_from_world(&mut self, kind: VoxelKind, grams: u64) -> bool {
        let available = self.world.get(&kind).copied().unwrap_or(0);
        if available < grams {
            return false;
        }
        *self.world.entry(kind).or_default() -= grams;
        *self.carried.entry(kind).or_default() += grams;
        true
    }

    fn consume_carried(&mut self, kind: VoxelKind, grams: u64) -> bool {
        let available = self.carried.get(&kind).copied().unwrap_or(0);
        if available < grams {
            return false;
        }
        *self.carried.entry(kind).or_default() -= grams;
        *self.spent.entry(kind).or_default() += grams;
        true
    }

    pub fn carried(&self, kind: VoxelKind) -> u64 {
        self.carried.get(&kind).copied().unwrap_or(0)
    }

    pub fn is_balanced(&self) -> bool {
        [VoxelKind::Soil, VoxelKind::Forage, VoxelKind::Wood]
            .into_iter()
            .all(|kind| {
                self.source.get(&kind).copied().unwrap_or(0)
                    == self.world.get(&kind).copied().unwrap_or(0)
                        + self.carried.get(&kind).copied().unwrap_or(0)
                        + self.spent.get(&kind).copied().unwrap_or(0)
            })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Ready,
    Day,
    Night,
    Dawn,
    Failed,
}

impl fmt::Display for Phase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Ready => "ready",
            Self::Day => "day",
            Self::Night => "night",
            Self::Dawn => "dawn",
            Self::Failed => "failed",
        };
        formatter.write_str(label)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Land(Coord),
    Forage(Coord),
    GatherWood(Coord),
    BuildFire(Coord),
    BuildShelter(Coord),
}

fn action_duration(action: &Action) -> u64 {
    match action {
        Action::Land(_) => LAND_TICKS,
        Action::Forage(_) => FORAGE_TICKS,
        Action::GatherWood(_) => WOOD_TICKS,
        Action::BuildFire(_) => FIRE_TICKS,
        Action::BuildShelter(_) => SHELTER_TICKS,
    }
}

impl fmt::Display for Action {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Land(coord) => write!(formatter, "land({},{},{})", coord.x, coord.y, coord.z),
            Self::Forage(coord) => {
                write!(formatter, "forage({},{},{})", coord.x, coord.y, coord.z)
            }
            Self::GatherWood(coord) => {
                write!(formatter, "wood({},{},{})", coord.x, coord.y, coord.z)
            }
            Self::BuildFire(coord) => {
                write!(formatter, "fire({},{},{})", coord.x, coord.y, coord.z)
            }
            Self::BuildShelter(coord) => {
                write!(formatter, "shelter({},{},{})", coord.x, coord.y, coord.z)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    AlreadyLanded,
    NotLanded,
    InvalidLanding(Coord),
    WrongPhase {
        expected: Phase,
        actual: Phase,
    },
    UnknownVoxel(Coord),
    WrongVoxel {
        coord: Coord,
        expected: VoxelKind,
        actual: VoxelKind,
    },
    NotAtCamp(Coord),
    AlreadyBuilt(&'static str),
    InsufficientMaterial {
        kind: VoxelKind,
        needed: u64,
        available: u64,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyLanded => formatter.write_str("the party has already landed"),
            Self::NotLanded => formatter.write_str("land before attempting work"),
            Self::InvalidLanding(coord) => write!(formatter, "{} is not supported land", coord),
            Self::WrongPhase { expected, actual } => {
                write!(formatter, "expected {expected}, world is {actual}")
            }
            Self::UnknownVoxel(coord) => write!(formatter, "no generated voxel at {coord:?}"),
            Self::WrongVoxel {
                coord,
                expected,
                actual,
            } => write!(formatter, "{coord:?} is {actual}, expected {expected}"),
            Self::NotAtCamp(coord) => {
                write!(formatter, "construction must happen at camp {coord:?}")
            }
            Self::AlreadyBuilt(kind) => write!(formatter, "{kind} is already built"),
            Self::InsufficientMaterial {
                kind,
                needed,
                available,
            } => write!(
                formatter,
                "{} needs {needed}g, only {available}g carried",
                kind.as_str()
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evidence {
    pub tick: u64,
    pub kind: &'static str,
    pub action: String,
    pub result: String,
    pub coord: Option<Coord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoundingWorld {
    pub seed: WorldSeed,
    pub revision: GeneratorRevision,
    pub tick: u64,
    pub phase: Phase,
    pub day_elapsed: u64,
    pub night_elapsed: u64,
    pub clock_running: bool,
    pub founder: Option<Coord>,
    pub camp: Option<Coord>,
    pub fire: Option<Coord>,
    pub shelter: Option<Coord>,
    pub chunks: BTreeMap<ChunkCoord, Chunk>,
    deltas: BTreeMap<ChunkCoord, Vec<ChunkDelta>>,
    accounted_chunks: BTreeSet<ChunkCoord>,
    pub ledger: MaterialLedger,
    pub evidence: Vec<Evidence>,
}

impl FoundingWorld {
    pub fn new(seed: WorldSeed, revision: GeneratorRevision) -> Self {
        let mut world = Self {
            seed,
            revision,
            tick: 0,
            phase: Phase::Ready,
            day_elapsed: 0,
            night_elapsed: 0,
            clock_running: false,
            founder: None,
            camp: None,
            fire: None,
            shelter: None,
            chunks: BTreeMap::new(),
            deltas: BTreeMap::new(),
            accounted_chunks: BTreeSet::new(),
            ledger: MaterialLedger::default(),
            evidence: Vec::new(),
        };
        world.ensure_chunk(ChunkCoord::new(0, 0, 0));
        world
    }

    pub fn starter_camp() -> Coord {
        Coord::new(4, 4, 0)
    }

    pub fn starter_forage() -> Coord {
        Coord::new(2, 2, 0)
    }

    pub fn starter_wood() -> Coord {
        Coord::new(6, 2, 0)
    }

    fn ensure_chunk(&mut self, chunk_coord: ChunkCoord) {
        if self.chunks.contains_key(&chunk_coord) {
            return;
        }
        let first_accounting = self.accounted_chunks.insert(chunk_coord);
        let mut chunk = Chunk::derive(self.seed, self.revision, chunk_coord);
        if first_accounting {
            for voxel in chunk.voxels.values() {
                self.ledger.add_source(voxel.kind, voxel.mass_grams);
            }
        }
        if let Some(deltas) = self.deltas.get(&chunk_coord) {
            for delta in deltas {
                chunk.apply_delta(chunk_coord, *delta);
            }
        }
        if first_accounting {
            for voxel in chunk.voxels.values() {
                self.ledger.add_world(voxel.kind, voxel.mass_grams);
            }
        }
        self.chunks.insert(chunk_coord, chunk);
    }

    pub fn load_chunk(&mut self, chunk_coord: ChunkCoord) -> bool {
        if self.chunks.contains_key(&chunk_coord) {
            return false;
        }
        self.ensure_chunk(chunk_coord);
        true
    }

    pub fn unload_chunk(&mut self, chunk_coord: ChunkCoord) -> bool {
        self.chunks.remove(&chunk_coord).is_some()
    }

    pub fn voxel_at(&self, coord: Coord) -> Option<Voxel> {
        self.chunks
            .get(&coord.chunk())?
            .voxels
            .get(&coord.local_in_chunk())
            .copied()
    }

    pub fn act(&mut self, action: Action) -> Result<(), Refusal> {
        let action_text = action.to_string();
        let duration = action_duration(&action);
        let coord = match action {
            Action::Land(coord)
            | Action::Forage(coord)
            | Action::GatherWood(coord)
            | Action::BuildFire(coord)
            | Action::BuildShelter(coord) => Some(coord),
        };
        let result = self.apply_action(action);
        let result_text = match &result {
            Ok(()) => "accepted".to_string(),
            Err(refusal) => format!("refused: {refusal}"),
        };
        self.evidence.push(Evidence {
            tick: self.tick,
            kind: "action",
            action: action_text,
            result: result_text,
            coord,
        });
        if result.is_ok() {
            self.advance_ticks(duration);
        }
        result
    }

    fn apply_action(&mut self, action: Action) -> Result<(), Refusal> {
        match action {
            Action::Land(coord) => {
                if self.founder.is_some() {
                    return Err(Refusal::AlreadyLanded);
                }
                if !coord.is_in_first_band()
                    || self
                        .voxel_at(coord)
                        .is_none_or(|voxel| voxel.kind != VoxelKind::Soil)
                {
                    return Err(Refusal::InvalidLanding(coord));
                }
                self.founder = Some(coord);
                self.camp = Some(coord);
                self.phase = Phase::Day;
                self.clock_running = true;
                Ok(())
            }
            Action::Forage(coord) => {
                self.require_day()?;
                let Some(voxel) = self.voxel_at(coord) else {
                    return Err(Refusal::UnknownVoxel(coord));
                };
                if voxel.kind != VoxelKind::Forage {
                    return Err(Refusal::WrongVoxel {
                        coord,
                        expected: VoxelKind::Forage,
                        actual: voxel.kind,
                    });
                }
                if !self
                    .ledger
                    .take_from_world(VoxelKind::Forage, FORAGE_TAKE_GRAMS)
                {
                    return Err(Refusal::InsufficientMaterial {
                        kind: VoxelKind::Forage,
                        needed: FORAGE_TAKE_GRAMS,
                        available: voxel.mass_grams,
                    });
                }
                self.remove_voxel_mass(coord, FORAGE_TAKE_GRAMS);
                Ok(())
            }
            Action::GatherWood(coord) => {
                self.require_day()?;
                let Some(voxel) = self.voxel_at(coord) else {
                    return Err(Refusal::UnknownVoxel(coord));
                };
                if voxel.kind != VoxelKind::Wood {
                    return Err(Refusal::WrongVoxel {
                        coord,
                        expected: VoxelKind::Wood,
                        actual: voxel.kind,
                    });
                }
                if !self
                    .ledger
                    .take_from_world(VoxelKind::Wood, WOOD_TAKE_GRAMS)
                {
                    return Err(Refusal::InsufficientMaterial {
                        kind: VoxelKind::Wood,
                        needed: WOOD_TAKE_GRAMS,
                        available: voxel.mass_grams,
                    });
                }
                self.remove_voxel_mass(coord, WOOD_TAKE_GRAMS);
                Ok(())
            }
            Action::BuildFire(coord) => {
                self.require_day()?;
                if self.fire.is_some() {
                    return Err(Refusal::AlreadyBuilt("fire"));
                }
                self.require_camp(coord)?;
                let available = self.ledger.carried(VoxelKind::Wood);
                if !self
                    .ledger
                    .consume_carried(VoxelKind::Wood, FIRE_WOOD_GRAMS)
                {
                    return Err(Refusal::InsufficientMaterial {
                        kind: VoxelKind::Wood,
                        needed: FIRE_WOOD_GRAMS,
                        available,
                    });
                }
                self.fire = Some(coord);
                Ok(())
            }
            Action::BuildShelter(coord) => {
                self.require_day()?;
                if self.shelter.is_some() {
                    return Err(Refusal::AlreadyBuilt("shelter"));
                }
                self.require_camp(coord)?;
                let available = self.ledger.carried(VoxelKind::Wood);
                if !self
                    .ledger
                    .consume_carried(VoxelKind::Wood, SHELTER_WOOD_GRAMS)
                {
                    return Err(Refusal::InsufficientMaterial {
                        kind: VoxelKind::Wood,
                        needed: SHELTER_WOOD_GRAMS,
                        available,
                    });
                }
                self.shelter = Some(coord);
                Ok(())
            }
        }
    }

    fn require_day(&self) -> Result<(), Refusal> {
        if self.phase != Phase::Day {
            return Err(Refusal::WrongPhase {
                expected: Phase::Day,
                actual: self.phase,
            });
        }
        if self.founder.is_none() {
            return Err(Refusal::NotLanded);
        }
        Ok(())
    }

    fn require_camp(&self, coord: Coord) -> Result<(), Refusal> {
        if self.camp != Some(coord) {
            return Err(Refusal::NotAtCamp(coord));
        }
        Ok(())
    }

    fn remove_voxel_mass(&mut self, coord: Coord, grams: u64) {
        let chunk_coord = coord.chunk();
        {
            let chunk = self
                .chunks
                .get_mut(&chunk_coord)
                .expect("cannot extract from an unloaded chunk");
            let local = coord.local_in_chunk();
            let voxel = chunk
                .voxels
                .get_mut(&local)
                .expect("extraction requires a resident voxel");
            voxel.mass_grams = voxel
                .mass_grams
                .checked_sub(grams)
                .expect("extraction exceeds the resident voxel mass");
            if voxel.mass_grams == 0 {
                chunk.voxels.remove(&local);
            }
        }
        self.deltas
            .entry(chunk_coord)
            .or_default()
            .push(ChunkDelta::RemoveVoxelMass { coord, grams });
    }

    pub fn advance_ticks(&mut self, ticks: u64) {
        if !self.clock_running || matches!(self.phase, Phase::Ready | Phase::Dawn | Phase::Failed) {
            return;
        }
        for _ in 0..ticks {
            self.tick = self.tick.saturating_add(1);
            match self.phase {
                Phase::Day => {
                    self.day_elapsed = self.day_elapsed.saturating_add(1);
                    if self.day_elapsed >= DAY_TICKS {
                        self.phase = Phase::Night;
                        self.night_elapsed = 0;
                        self.evidence.push(Evidence {
                            tick: self.tick,
                            kind: "world",
                            action: "night".into(),
                            result: "world clock entered night".into(),
                            coord: self.camp,
                        });
                    }
                }
                Phase::Night => {
                    self.night_elapsed = self.night_elapsed.saturating_add(1);
                    if self.night_elapsed >= NIGHT_TICKS {
                        self.finish_night();
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn finish_night(&mut self) {
        let can_survive = self.fire.is_some()
            && self.shelter.is_some()
            && self.ledger.carried(VoxelKind::Forage) >= FORAGE_TAKE_GRAMS;
        if can_survive {
            self.ledger
                .consume_carried(VoxelKind::Forage, FORAGE_TAKE_GRAMS);
            self.phase = Phase::Dawn;
            self.clock_running = false;
            self.evidence.push(Evidence {
                tick: self.tick,
                kind: "world",
                action: "dawn".into(),
                result: "first night passed with food, fire, and shelter".into(),
                coord: self.camp,
            });
        } else {
            self.phase = Phase::Failed;
            self.clock_running = false;
            self.evidence.push(Evidence {
                tick: self.tick,
                kind: "world",
                action: "dawn".into(),
                result: "first night failed: food, fire, and shelter were not all established"
                    .into(),
                coord: self.camp,
            });
        }
    }

    pub fn state_digest(&self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        fn feed(hash: &mut u64, bytes: &[u8]) {
            for byte in bytes {
                *hash ^= u64::from(*byte);
                *hash = hash.wrapping_mul(0x1000_0000_01b3);
            }
        }
        fn feed_u64(hash: &mut u64, value: u64) {
            feed(hash, &value.to_le_bytes());
        }
        fn feed_coord(hash: &mut u64, coord: Option<Coord>) {
            match coord {
                Some(coord) => {
                    feed(hash, &[1]);
                    feed_u64(hash, coord.x as i64 as u64);
                    feed_u64(hash, coord.y as i64 as u64);
                    feed_u64(hash, coord.z as i64 as u64);
                }
                None => feed(hash, &[0]),
            }
        }
        feed_u64(&mut hash, self.seed.0);
        feed_u64(&mut hash, self.revision.0);
        feed_u64(&mut hash, self.tick);
        feed_u64(&mut hash, self.day_elapsed);
        feed_u64(&mut hash, self.night_elapsed);
        feed(&mut hash, &[u8::from(self.clock_running)]);
        feed(&mut hash, self.phase.to_string().as_bytes());
        feed_coord(&mut hash, self.founder);
        feed_coord(&mut hash, self.camp);
        feed_coord(&mut hash, self.fire);
        feed_coord(&mut hash, self.shelter);
        for (chunk_coord, deltas) in &self.deltas {
            feed_u64(&mut hash, chunk_coord.x as i64 as u64);
            feed_u64(&mut hash, chunk_coord.y as i64 as u64);
            feed_u64(&mut hash, chunk_coord.z as i64 as u64);
            for delta in deltas {
                let ChunkDelta::RemoveVoxelMass { coord, grams } = delta;
                feed_coord(&mut hash, Some(*coord));
                feed_u64(&mut hash, *grams);
            }
        }
        for kind in [VoxelKind::Soil, VoxelKind::Forage, VoxelKind::Wood] {
            feed_u64(
                &mut hash,
                self.ledger.source.get(&kind).copied().unwrap_or(0),
            );
            feed_u64(
                &mut hash,
                self.ledger.world.get(&kind).copied().unwrap_or(0),
            );
            feed_u64(&mut hash, self.ledger.carried(kind));
            feed_u64(
                &mut hash,
                self.ledger.spent.get(&kind).copied().unwrap_or(0),
            );
        }
        hash
    }

    pub fn snapshot(&self) -> FoundingSnapshot {
        FoundingSnapshot {
            tick: self.tick,
            phase: self.phase,
            day_elapsed: self.day_elapsed,
            night_elapsed: self.night_elapsed,
            camp: self.camp,
            fire: self.fire,
            shelter: self.shelter,
            carried_forage_grams: self.ledger.carried(VoxelKind::Forage),
            carried_wood_grams: self.ledger.carried(VoxelKind::Wood),
            material_balanced: self.ledger.is_balanced(),
            digest: self.state_digest(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoundingSnapshot {
    pub tick: u64,
    pub phase: Phase,
    pub day_elapsed: u64,
    pub night_elapsed: u64,
    pub camp: Option<Coord>,
    pub fire: Option<Coord>,
    pub shelter: Option<Coord>,
    pub carried_forage_grams: u64,
    pub carried_wood_grams: u64,
    pub material_balanced: bool,
    pub digest: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> FoundingWorld {
        FoundingWorld::new(WorldSeed(7), GeneratorRevision(1))
    }

    #[test]
    fn same_seed_revision_and_coordinate_have_the_same_voxel() {
        let first = world();
        let second = world();
        assert_eq!(
            first.voxel_at(FoundingWorld::starter_forage()),
            second.voxel_at(FoundingWorld::starter_forage())
        );
        assert_eq!(first.state_digest(), second.state_digest());
    }

    #[test]
    fn signed_chunk_coordinates_load_and_rederive() {
        let mut world = world();
        let chunk = ChunkCoord::new(-1, 2, -1);
        let coord = Coord::new(-1, 65, -1);

        assert!(world.load_chunk(chunk));
        let voxel = world
            .voxel_at(coord)
            .expect("signed coordinate belongs to loaded chunk");
        assert!(world.unload_chunk(chunk));
        assert!(!world.chunks.contains_key(&chunk));
        assert!(world.load_chunk(chunk));

        assert_eq!(world.voxel_at(coord), Some(voxel));
        assert!(world.ledger.is_balanced());
    }

    #[test]
    fn unload_rederive_preserves_typed_deltas_and_digest() {
        let mut world = world();
        let camp = FoundingWorld::starter_camp();
        let forage = FoundingWorld::starter_forage();
        let chunk = camp.chunk();
        world.act(Action::Land(camp)).unwrap();
        world.act(Action::Forage(forage)).unwrap();
        let harvested = world
            .voxel_at(forage)
            .expect("forage remains after one harvest");
        let digest = world.state_digest();

        assert!(world.unload_chunk(chunk));
        assert_eq!(world.state_digest(), digest);
        assert!(world.load_chunk(chunk));

        assert_eq!(world.voxel_at(forage), Some(harvested));
        assert_eq!(world.state_digest(), digest);
        assert!(world.ledger.is_balanced());
    }

    #[test]
    fn the_clock_transitions_without_player_actions() {
        let mut world = world();
        world
            .act(Action::Land(FoundingWorld::starter_camp()))
            .unwrap();
        assert_eq!(world.phase, Phase::Day);
        world.advance_ticks(DAY_TICKS);
        assert_eq!(world.phase, Phase::Night);
        world.advance_ticks(NIGHT_TICKS);
        assert_eq!(world.phase, Phase::Failed);
    }

    #[test]
    fn founding_materials_are_conserved_and_dawn_is_automatic() {
        let mut world = world();
        let camp = FoundingWorld::starter_camp();
        world.act(Action::Land(camp)).unwrap();
        world
            .act(Action::Forage(FoundingWorld::starter_forage()))
            .unwrap();
        world
            .act(Action::Forage(FoundingWorld::starter_forage()))
            .unwrap();
        world
            .act(Action::Forage(FoundingWorld::starter_forage()))
            .unwrap();
        world
            .act(Action::Forage(FoundingWorld::starter_forage()))
            .unwrap();
        let wood = FoundingWorld::starter_wood();
        world.act(Action::GatherWood(wood)).unwrap();
        world.act(Action::GatherWood(wood)).unwrap();
        world.act(Action::BuildFire(camp)).unwrap();
        world.act(Action::BuildShelter(camp)).unwrap();
        assert!(world.ledger.is_balanced());
        world.advance_ticks(DAY_TICKS + NIGHT_TICKS);
        assert_eq!(world.phase, Phase::Dawn);
        assert!(world.ledger.is_balanced());
    }
}
