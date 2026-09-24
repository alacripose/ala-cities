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
