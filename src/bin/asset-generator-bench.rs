//! Reproducible first-pass measurements for the Dynamic Asset Generator.
//!
//! This records raw release-build readings. It deliberately does not declare
//! performance budgets: #28 requires budgets to be derived from measurements.

use std::hint::black_box;
use std::time::{Duration, Instant};

use ala_cities::asset_generator::DynamicAssetGenerator;
use ala_cities::asset_streaming::{ChunkAssetStreamer, ChunkStreamPolicy};
use ala_cities::buildinfo;
use ala_cities::founding_day::{ChunkCoord, FoundingWorld, GeneratorRevision, WorldSeed};
use ala_cities::render::{Camera, Screen};
use serde::Serialize;

const DEFAULT_SAMPLES: usize = 20;
const RAYCASTS_PER_SAMPLE: usize = 1_000;

#[derive(Serialize)]
struct Report {
    schema: &'static str,
    build: BuildIdentity,
    samples: usize,
    raycasts_per_sample: usize,
    scenarios: Vec<Scenario>,
    resources: Vec<ResourceReading>,
    streaming: StreamingReading,
}

#[derive(Serialize)]
struct BuildIdentity {
    commit: Option<String>,
    branch: Option<String>,
    dirty: Option<bool>,
    profile: &'static str,
}

#[derive(Serialize)]
struct Scenario {
    name: &'static str,
    unit: &'static str,
    operations_per_sample: usize,
    samples: usize,
    min_us: f64,
    p50_us: f64,
    p95_us: f64,
    max_us: f64,
}

#[derive(Serialize)]
struct StreamingReading {
    requested_chunks: usize,
    completed_ticks: usize,
    built_chunks: usize,
    evicted_chunks: usize,
    retained_chunks: usize,
    cache_entries: usize,
    world_digest_unchanged: bool,
}

#[derive(Serialize)]
struct ResourceReading {
    scenario: &'static str,
    world_voxel_entries: usize,
    octree_occupied_voxels: usize,
    mesh_vertices: usize,
    mesh_indices: usize,
    logical_mesh_bytes: usize,
    octree_nodes: usize,
    octree_max_nodes: usize,
    voxel_capacity: usize,
    visible_triangles: usize,
}

fn main() -> Result<(), String> {
    let samples = match std::env::args().nth(1) {
        Some(value) => value
            .parse::<usize>()
            .map_err(|error| format!("sample count must be a positive integer: {error}"))?,
        None => DEFAULT_SAMPLES,
    };
    if samples == 0 {
        return Err("sample count must be greater than zero".into());
    }

    let mut cold_positive = Vec::with_capacity(samples);
    let mut warm_positive = Vec::with_capacity(samples);
    let mut cold_negative = Vec::with_capacity(samples);
    let mut cubic_build = Vec::with_capacity(samples);
    let mut culling = Vec::with_capacity(samples);
    let mut raycast = Vec::with_capacity(samples);
    let mut streaming_schedule = Vec::with_capacity(samples);
    let mut streaming_reading = None;
    let mut resources = Vec::with_capacity(4);

    for sample in 0..samples {
        let positive = starter_world(ChunkCoord::new(0, 0, 0));
        let mut generator = DynamicAssetGenerator::default();
        let (cold, warm, asset) = timed_build(&mut generator, &positive, ChunkCoord::new(0, 0, 0));
        cold_positive.push(cold);
        warm_positive.push(warm);
        if sample == 0 {
            resources.push(resource_reading(
                "positive_starter",
                &positive,
                ChunkCoord::new(0, 0, 0),
                &asset,
                0,
            ));
        }

        let negative = starter_world(ChunkCoord::new(-1, 0, 0));
        let mut generator = DynamicAssetGenerator::default();
        let (cold, _, asset) = timed_build(&mut generator, &negative, ChunkCoord::new(-1, 0, 0));
        cold_negative.push(cold);
        if sample == 0 {
            resources.push(resource_reading(
                "negative_starter",
                &negative,
                ChunkCoord::new(-1, 0, 0),
                &asset,
                0,
            ));
        }

        let chunk = ChunkCoord::new(0, 0, 0);
        let cubic = FoundingWorld::cubic_preview(WorldSeed(7), GeneratorRevision(1), chunk);
        let mut generator = DynamicAssetGenerator::default();
        let started = Instant::now();
        let asset = black_box(
            generator
                .build_chunk(&cubic, chunk)
                .map_err(|error| error.to_string())?,
        );
        cubic_build.push(started.elapsed());
        if sample == 0 {
            resources.push(resource_reading("cubic_preview", &cubic, chunk, &asset, 0));
        }

        let camera = camera();
        let culler = camera.triangle_culler();
        let started = Instant::now();
        let visible = black_box(asset.visible_triangles(culler).count());
        culling.push(started.elapsed());
        if sample == 0 {
            let last = resources
                .last_mut()
                .expect("cubic resource reading was just inserted");
            last.visible_triangles = visible;
        }

        let ray = camera.ray(640.0, 400.0);
        let direction = ray.1 - ray.0;
        let started = Instant::now();
        for _ in 0..RAYCASTS_PER_SAMPLE {
            black_box(
                asset
                    .raycast(ray.0, direction, direction.length() + 1.0)
                    .expect("cubic preview camera ray must hit"),
            );
        }
        raycast.push(started.elapsed());

        let (duration, reading) = measure_streaming();
        streaming_schedule.push(duration);
        if sample == 0 {
            streaming_reading = Some(reading);
        }
    }

    let build = buildinfo::query();
    let report = Report {
        schema: "ala-cities/asset-generator-benchmark/v2",
        build: BuildIdentity {
            commit: build.commit,
            branch: build.branch,
            dirty: build.dirty,
            profile: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
        },
        samples,
        raycasts_per_sample: RAYCASTS_PER_SAMPLE,
        scenarios: vec![
            scenario("cold_positive_chunk", 1, cold_positive),
            scenario("warm_cached_chunk", 1, warm_positive),
            scenario("cold_negative_chunk", 1, cold_negative),
            scenario("cold_cubic_preview", 1, cubic_build),
            scenario("cubic_visible_triangle_filter", 1, culling),
            scenario("exact_raycast_batch", RAYCASTS_PER_SAMPLE, raycast),
            scenario("nine_chunk_stream_budget_one", 9, streaming_schedule),
        ],
        resources,
        streaming: streaming_reading.expect("at least one benchmark sample is required"),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn starter_world(chunk: ChunkCoord) -> FoundingWorld {
    let mut world = FoundingWorld::new(WorldSeed(7), GeneratorRevision(1));
    world.load_chunk(chunk);
    world
}

fn timed_build(
    generator: &mut DynamicAssetGenerator,
    world: &FoundingWorld,
    chunk: ChunkCoord,
) -> (Duration, Duration, ala_cities::asset_generator::ChunkAsset) {
    let started = Instant::now();
    let asset = black_box(
        generator
            .build_chunk(world, chunk)
            .expect("benchmark chunk must build"),
    );
    let cold = started.elapsed();
    let started = Instant::now();
    black_box(
        generator
            .build_chunk(world, chunk)
            .expect("warm benchmark chunk must build"),
    );
    (cold, started.elapsed(), asset)
}

fn camera() -> Camera {
    let mut camera = Camera::new(
        Screen {
            w: 1280.0,
            h: 800.0,
        },
        32,
        32,
    );
    camera.focus = glam::Vec3::new(16.0, 16.0, 16.0);
    camera.zoom = 14.0;
    camera
}

fn measure_streaming() -> (Duration, StreamingReading) {
    let chunks = [
        ChunkCoord::new(-1, -1, -1),
        ChunkCoord::new(0, -1, -1),
        ChunkCoord::new(1, -1, -1),
        ChunkCoord::new(-1, 0, -1),
        ChunkCoord::new(0, 0, -1),
        ChunkCoord::new(1, 0, -1),
        ChunkCoord::new(-1, 1, -1),
        ChunkCoord::new(0, 1, -1),
        ChunkCoord::new(1, 1, -1),
    ];
    let mut world = FoundingWorld::new(WorldSeed(7), GeneratorRevision(1));
    for chunk in &chunks {
        world.load_chunk(*chunk);
    }
    let before = world.state_digest();
    let mut streamer = ChunkAssetStreamer::new(ChunkStreamPolicy::new(1, 3));
    for chunk in chunks {
        assert!(streamer.request(chunk), "benchmark requests must be unique");
    }

    let started = Instant::now();
    let mut completed_ticks = 0;
    let mut built_chunks = 0;
    let mut evicted_chunks = 0;
    while streamer.pending_len() > 0 {
        let tick = streamer.step(&world);
        completed_ticks += 1;
        built_chunks += tick.built.len();
        evicted_chunks += tick.evicted.len();
        assert!(
            tick.failures.is_empty(),
            "resident stream requests must build"
        );
        assert!(tick.built.len() <= 1, "stream budget must be exact");
    }
    let duration = started.elapsed();
    let reading = StreamingReading {
        requested_chunks: chunks.len(),
        completed_ticks,
        built_chunks,
        evicted_chunks,
        retained_chunks: streamer.cached_chunks().len(),
        cache_entries: streamer.cache_entry_count(),
        world_digest_unchanged: world.state_digest() == before,
    };
    assert_eq!(built_chunks, chunks.len());
    assert_eq!(reading.retained_chunks, 3);
    assert_eq!(reading.cache_entries, 3);
    assert_eq!(evicted_chunks, chunks.len() - 3);
    assert!(reading.world_digest_unchanged);
    (duration, reading)
}

fn resource_reading(
    scenario: &'static str,
    world: &FoundingWorld,
    chunk: ChunkCoord,
    asset: &ala_cities::asset_generator::ChunkAsset,
    visible_triangles: usize,
) -> ResourceReading {
    let resident = world.chunk(chunk).expect("benchmark chunk is resident");
    let logical_mesh_bytes = asset.mesh.vertices.len()
        * std::mem::size_of::<ala_cities::asset_generator::MeshVertex>()
        + asset.mesh.indices.len() * std::mem::size_of::<u32>();
    ResourceReading {
        scenario,
        world_voxel_entries: resident.voxels.len(),
        octree_occupied_voxels: asset.octree.occupied_count(),
        mesh_vertices: asset.mesh.vertices.len(),
        mesh_indices: asset.mesh.indices.len(),
        logical_mesh_bytes,
        octree_nodes: asset.octree.node_count(),
        octree_max_nodes: asset.octree.max_nodes(),
        voxel_capacity: asset.octree.capacity(),
        visible_triangles,
    }
}

fn scenario(
    name: &'static str,
    operations_per_sample: usize,
    mut durations: Vec<Duration>,
) -> Scenario {
    durations.sort_unstable();
    Scenario {
        name,
        unit: "microseconds_per_sample_total",
        operations_per_sample,
        samples: durations.len(),
        min_us: micros(durations[0]),
        p50_us: micros(percentile(&durations, 0.50)),
        p95_us: micros(percentile(&durations, 0.95)),
        max_us: micros(durations[durations.len() - 1]),
    }
}

fn percentile(sorted: &[Duration], percentile: f64) -> Duration {
    let rank = (percentile * sorted.len() as f64).ceil() as usize;
    sorted[rank.saturating_sub(1).min(sorted.len() - 1)]
}

fn micros(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::{percentile, Scenario};
    use std::time::Duration;

    #[test]
    fn percentile_uses_nearest_rank() {
        let values = [
            Duration::from_micros(1),
            Duration::from_micros(2),
            Duration::from_micros(3),
            Duration::from_micros(4),
        ];
        assert_eq!(percentile(&values, 0.0), Duration::from_micros(1));
        assert_eq!(percentile(&values, 0.5), Duration::from_micros(2));
        assert_eq!(percentile(&values, 0.95), Duration::from_micros(4));
    }

    #[test]
    fn scenario_summary_keeps_raw_sample_count() {
        let summary = super::scenario(
            "test",
            1,
            vec![
                Duration::from_micros(4),
                Duration::from_micros(1),
                Duration::from_micros(3),
                Duration::from_micros(2),
            ],
        );
        assert_eq!(summary.operations_per_sample, 1);
        assert_eq!(summary.samples, 4);
        assert_eq!(summary.min_us, 1.0);
        assert_eq!(summary.p50_us, 2.0);
        assert_eq!(summary.p95_us, 4.0);
        assert_eq!(summary.max_us, 4.0);
        let _: &Scenario = &summary;
    }
}
