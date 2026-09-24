//! Deterministic, tick-budgeted runtime chunk asset streaming.
//!
//! This schedules already-resident world truth. It does not generate, mutate,
//! unload, or otherwise own canonical chunks.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::asset_generator::{AssetBuildError, ChunkAsset, DynamicAssetGenerator};
use crate::founding_day::{ChunkCoord, FoundingWorld};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkStreamPolicy {
    pub max_builds_per_tick: usize,
    pub max_cached_chunks: usize,
}

impl ChunkStreamPolicy {
    pub fn new(max_builds_per_tick: usize, max_cached_chunks: usize) -> Self {
        assert!(
            max_builds_per_tick > 0,
            "a stream tick needs build capacity"
        );
        assert!(max_cached_chunks > 0, "the stream cache needs one chunk");
        Self {
            max_builds_per_tick,
            max_cached_chunks,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct StreamTick {
    pub tick: u64,
    pub built: Vec<(ChunkCoord, ChunkAsset)>,
    pub failures: Vec<(ChunkCoord, AssetBuildError)>,
    pub evicted: Vec<ChunkCoord>,
    pub remaining_requests: usize,
}

pub struct ChunkAssetStreamer {
    generator: DynamicAssetGenerator,
    policy: ChunkStreamPolicy,
    pending: VecDeque<ChunkCoord>,
    queued: BTreeSet<ChunkCoord>,
    last_used: BTreeMap<ChunkCoord, u64>,
    access_clock: u64,
    tick: u64,
}

impl ChunkAssetStreamer {
    pub fn new(policy: ChunkStreamPolicy) -> Self {
        Self {
            generator: DynamicAssetGenerator::default(),
            policy,
            pending: VecDeque::new(),
            queued: BTreeSet::new(),
            last_used: BTreeMap::new(),
            access_clock: 0,
            tick: 0,
        }
    }

    /// Queue one refresh/build. Duplicate pending requests are ignored.
    pub fn request(&mut self, chunk: ChunkCoord) -> bool {
        if !self.queued.insert(chunk) {
            return false;
        }
        self.pending.push_back(chunk);
        true
    }

    /// Queue a signed-coordinate neighborhood in deterministic near-to-far order.
    /// Returns the number of newly queued coordinates.
    pub fn request_neighborhood(&mut self, center: ChunkCoord, radius: i32) -> usize {
        const MAX_REQUEST_CELLS: i64 = 1_000_000;
        if radius < 0 {
            return 0;
        }
        let diameter = i64::from(radius) * 2 + 1;
        let Some(cell_count) = diameter
            .checked_mul(diameter)
            .and_then(|value| value.checked_mul(diameter))
        else {
            return 0;
        };
        if cell_count > MAX_REQUEST_CELLS {
            return 0;
        }
        let mut candidates = Vec::with_capacity(cell_count as usize);
        for z in -radius..=radius {
            for y in -radius..=radius {
                for x in -radius..=radius {
                    let (Some(chunk_x), Some(chunk_y), Some(chunk_z)) = (
                        center.x.checked_add(x),
                        center.y.checked_add(y),
                        center.z.checked_add(z),
                    ) else {
                        continue;
                    };
                    candidates.push((
                        i64::from(x).abs() + i64::from(y).abs() + i64::from(z).abs(),
                        ChunkCoord::new(chunk_x, chunk_y, chunk_z),
                    ));
                }
            }
        }
        candidates.sort_by_key(|(distance, chunk)| (*distance, *chunk));
        candidates
            .into_iter()
            .filter(|(_, chunk)| self.request(*chunk))
            .count()
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    pub fn cached_chunks(&self) -> BTreeSet<ChunkCoord> {
        self.generator.cached_chunks()
    }

    pub fn cache_entry_count(&self) -> usize {
        self.generator.cache_len()
    }

    /// Build at most the configured number of queued chunks and apply cache pressure.
    pub fn step(&mut self, world: &FoundingWorld) -> StreamTick {
        self.tick += 1;
        let mut result = StreamTick {
            tick: self.tick,
            remaining_requests: self.pending.len(),
            ..StreamTick::default()
        };
        let mut remaining_budget = self.policy.max_builds_per_tick;
        while remaining_budget > 0 {
            let Some(chunk) = self.pending.pop_front() else {
                break;
            };
            self.queued.remove(&chunk);
            match self.generator.build_chunk(world, chunk) {
                Ok(asset) => {
                    self.generator.evict_other_chunk_versions(chunk, asset.key);
                    self.access_clock += 1;
                    self.last_used.insert(chunk, self.access_clock);
                    result.built.push((chunk, asset));
                }
                Err(error) => result.failures.push((chunk, error)),
            }
            remaining_budget -= 1;
        }
        result.evicted = self.enforce_cache_capacity();
        result.remaining_requests = self.pending.len();
        result
    }

    fn enforce_cache_capacity(&mut self) -> Vec<ChunkCoord> {
        let mut evicted = Vec::new();
        while self.generator.cached_chunks().len() > self.policy.max_cached_chunks {
            let Some(chunk) = self
                .generator
                .cached_chunks()
                .into_iter()
                .min_by_key(|chunk| (self.last_used.get(chunk).copied().unwrap_or(0), *chunk))
            else {
                break;
            };
            self.generator.evict_chunk(chunk);
            self.last_used.remove(&chunk);
            evicted.push(chunk);
        }
        evicted
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{ChunkAssetStreamer, ChunkStreamPolicy};
    use crate::founding_day::{Action, ChunkCoord, FoundingWorld, GeneratorRevision, WorldSeed};

    fn world(chunks: &[ChunkCoord]) -> FoundingWorld {
        let mut world = FoundingWorld::new(WorldSeed(7), GeneratorRevision(1));
        for chunk in chunks {
            world.load_chunk(*chunk);
        }
        world
    }

    #[test]
    fn neighborhood_requests_are_signed_centered_and_near_to_far() {
        let mut streamer = ChunkAssetStreamer::new(ChunkStreamPolicy::new(2, 4));
        assert_eq!(
            streamer.request_neighborhood(ChunkCoord::new(0, 0, 0), 1),
            27
        );
        let queued = streamer.pending.iter().copied().collect::<Vec<_>>();
        assert_eq!(queued[0], ChunkCoord::new(0, 0, 0));
        assert_eq!(queued[1], ChunkCoord::new(-1, 0, 0));
        assert_eq!(queued[2], ChunkCoord::new(0, -1, 0));
        assert_eq!(queued[3], ChunkCoord::new(0, 0, -1));
        assert_eq!(queued[4], ChunkCoord::new(0, 0, 1));
        assert_eq!(queued[5], ChunkCoord::new(0, 1, 0));
        assert_eq!(queued[6], ChunkCoord::new(1, 0, 0));
        assert_eq!(
            streamer.request_neighborhood(ChunkCoord::new(0, 0, 0), -1),
            0
        );
    }

    #[test]
    fn a_tick_builds_only_its_budget_without_mutating_world_truth() {
        let chunks = [
            ChunkCoord::new(-1, 0, 0),
            ChunkCoord::new(0, 0, 0),
            ChunkCoord::new(1, 0, 0),
        ];
        let world = world(&chunks);
        let before = world.state_digest();
        let mut streamer = ChunkAssetStreamer::new(ChunkStreamPolicy::new(2, 3));
        for chunk in chunks {
            assert!(streamer.request(chunk));
        }
        assert!(
            !streamer.request(chunks[0]),
            "pending duplicates are rejected"
        );

        let tick = streamer.step(&world);
        assert_eq!(tick.tick, 1);
        assert_eq!(
            tick.built
                .iter()
                .map(|(chunk, _)| *chunk)
                .collect::<Vec<_>>(),
            chunks[..2]
        );
        assert!(tick.failures.is_empty());
        assert_eq!(tick.remaining_requests, 1);
        assert_eq!(world.state_digest(), before);
    }

    #[test]
    fn cache_pressure_evicts_the_least_recently_built_chunk() {
        let chunks = [
            ChunkCoord::new(-1, 0, 0),
            ChunkCoord::new(0, 0, 0),
            ChunkCoord::new(1, 0, 0),
        ];
        let world = world(&chunks);
        let mut streamer = ChunkAssetStreamer::new(ChunkStreamPolicy::new(1, 1));
        for chunk in chunks {
            streamer.request(chunk);
        }

        assert_eq!(streamer.step(&world).built[0].0, chunks[0]);
        let second = streamer.step(&world);
        assert_eq!(second.built[0].0, chunks[1]);
        assert_eq!(second.evicted, vec![chunks[0]]);
        assert_eq!(streamer.cached_chunks(), BTreeSet::from([chunks[1]]));

        let third = streamer.step(&world);
        assert_eq!(third.built[0].0, chunks[2]);
        assert_eq!(third.evicted, vec![chunks[1]]);
    }

    #[test]
    fn successful_refresh_removes_stale_versions_of_the_same_chunk() {
        let chunk = ChunkCoord::new(0, 0, 0);
        let mut world = world(&[chunk]);
        let mut streamer = ChunkAssetStreamer::new(ChunkStreamPolicy::new(1, 3));
        streamer.request(chunk);
        assert_eq!(streamer.step(&world).built.len(), 1);
        assert_eq!(streamer.cache_entry_count(), 1);

        world
            .act(Action::Land(FoundingWorld::starter_camp()))
            .expect("founder lands");
        streamer.request(chunk);
        assert_eq!(streamer.step(&world).built.len(), 1);
        assert_eq!(
            streamer.cache_entry_count(),
            1,
            "a successful refresh must not retain stale input-digest versions"
        );
    }

    #[test]
    fn a_failed_request_consumes_budget_without_retrying_forever() {
        let world = world(&[]);
        let mut streamer = ChunkAssetStreamer::new(ChunkStreamPolicy::new(1, 3));
        streamer.request(ChunkCoord::new(4, 4, 4));
        let tick = streamer.step(&world);
        assert!(tick.built.is_empty());
        assert_eq!(tick.failures.len(), 1);
        assert_eq!(tick.remaining_requests, 0);
        assert_eq!(streamer.pending_len(), 0);
    }

    #[test]
    fn identical_requests_replay_identical_asset_digests() {
        let chunks = [
            ChunkCoord::new(-1, 0, 0),
            ChunkCoord::new(0, 0, 0),
            ChunkCoord::new(1, 0, 0),
        ];
        let world = world(&chunks);
        let replay = || {
            let mut streamer = ChunkAssetStreamer::new(ChunkStreamPolicy::new(1, 2));
            for chunk in chunks {
                streamer.request(chunk);
            }
            let mut digests = Vec::new();
            while streamer.pending_len() > 0 {
                digests.extend(
                    streamer
                        .step(&world)
                        .built
                        .into_iter()
                        .map(|(_, asset)| asset.digest),
                );
            }
            digests
        };
        assert_eq!(replay(), replay());
    }
}
