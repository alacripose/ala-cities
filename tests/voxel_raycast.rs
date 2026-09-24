use ala_cities::founding_day::{ChunkCoord, Coord, FoundingWorld, GeneratorRevision, WorldSeed};
use ala_cities::raycast::{raycast_voxels, Face};
use ala_cities::render::{Camera, Screen};
use glam::Vec3;

fn world() -> FoundingWorld {
    FoundingWorld::new(WorldSeed(7), GeneratorRevision(1))
}

fn cast(
    world: &FoundingWorld,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
) -> Option<ala_cities::raycast::VoxelHit> {
    raycast_voxels(origin, direction, max_distance, |coord| {
        world.voxel_at(coord).is_some()
    })
}

#[test]
fn axis_aligned_ray_reports_the_exact_entry_face() {
    let world = world();
    let hit = cast(&world, Vec3::new(10.5, 10.5, 5.0), Vec3::NEG_Z, 10.0)
        .expect("ray hits the generated surface");

    assert_eq!(hit.coord, Coord::new(10, 10, 0));
    assert_eq!(hit.normal, Some(Face::PosZ));
    assert!((hit.distance - 4.0).abs() < 1e-5);
}

#[test]
fn raycasting_works_across_negative_chunk_coordinates() {
    let mut world = world();
    let chunk = ChunkCoord::new(-1, 0, 0);
    assert!(world.load_chunk(chunk));
    let hit = cast(&world, Vec3::new(-30.5, 10.5, 4.0), Vec3::NEG_Z, 10.0)
        .expect("ray hits the negative-coordinate surface");

    assert_eq!(hit.coord, Coord::new(-31, 10, 0));
    assert_eq!(hit.normal, Some(Face::PosZ));
}

#[test]
fn the_native_target_camera_ray_matches_authoritative_world_raycasting() {
    let world = world();
    let mut generator = ala_cities::asset_generator::DynamicAssetGenerator::default();
    let asset = generator
        .build_chunk(&world, ChunkCoord::new(0, 0, 0))
        .expect("origin chunk builds");
    let mut camera = Camera::new(
        Screen {
            w: 1280.0,
            h: 800.0,
        },
        32,
        32,
    );
    camera.focus = Vec3::new(16.0, 16.0, 1.5);
    camera.zoom = 24.0;
    let (near, far) = camera.ray(640.0, 400.0);
    let direction = far - near;
    let expected = cast(&world, near, direction, direction.length() + 1.0)
        .expect("authoritative world ray hits terrain");
    let actual = asset
        .raycast(near, direction, direction.length() + 1.0)
        .expect("runtime chunk ray hits terrain");

    assert_eq!(actual, expected);
    assert!((0..32).contains(&actual.coord.x));
    assert!((0..32).contains(&actual.coord.y));
    assert!((0..32).contains(&actual.coord.z));
}

#[test]
fn an_exact_camera_ray_round_trips_to_the_visible_voxel() {
    let world = world();
    let mut camera = Camera::new(Screen { w: 800.0, h: 600.0 }, 64, 64);
    camera.focus = Vec3::new(16.0, 16.0, 0.0);
    camera.zoom = 24.0;

    let visible_point = Vec3::new(10.5, 12.5, 1.0);
    let screen = camera.world_to_screen(visible_point);
    let (near, far) = camera.ray(screen.0, screen.1);
    let direction = far - near;
    let hit = cast(&world, near, direction, direction.length() + 1.0)
        .expect("camera ray hits generated terrain");

    assert_eq!(hit.coord, Coord::new(10, 12, 0));
    assert_eq!(hit.normal, Some(Face::PosZ));
}
