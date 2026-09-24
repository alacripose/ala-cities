use ala_cities::asset_generator::DynamicAssetGenerator;
use ala_cities::founding_day::{ChunkCoord, FoundingWorld, GeneratorRevision, WorldSeed};
use ala_cities::render::{Camera, Screen};
use glam::Vec3;

fn target_camera() -> Camera {
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
    camera
}

fn triangle(center: Vec3, right: Vec3, up: Vec3) -> [Vec3; 3] {
    [
        center - right - up,
        center + right - up,
        center + right + up,
    ]
}

#[test]
fn view_culling_keeps_front_triangles_and_rejects_backfaces_and_offscreen_triangles() {
    let camera = target_camera();
    let culler = camera.triangle_culler();
    let extent_right = camera.right() * 0.1;
    let extent_up = camera.up() * 0.1;
    let normal = camera.toward_camera();

    assert!(culler.accepts_triangle(triangle(camera.focus, extent_right, extent_up), normal));
    assert!(!culler.accepts_triangle(triangle(camera.focus, extent_right, extent_up), -normal));

    let half_width = camera.screen.w / 2.0 / camera.zoom;
    let offscreen = camera.focus - camera.right() * (half_width + 20.0);
    assert!(!culler.accepts_triangle(triangle(offscreen, extent_right, extent_up), normal));

    let crossing = camera.focus - camera.right() * (half_width + 5.0);
    assert!(culler.accepts_triangle(
        triangle(crossing, camera.right() * (half_width + 10.0), extent_up),
        normal
    ));
}

#[test]
fn a_rotated_camera_culls_a_negative_chunk_without_mutating_world_truth() {
    let mut world = FoundingWorld::new(WorldSeed(7), GeneratorRevision(1));
    let chunk = ChunkCoord::new(-1, 0, 0);
    world.load_chunk(chunk);
    let before = world.state_digest();
    let asset = DynamicAssetGenerator::default()
        .build_chunk(&world, chunk)
        .expect("negative chunk builds");

    let mut camera = target_camera();
    camera.focus = Vec3::new(-16.0, -16.0, 1.5);
    camera.yaw = 0.73;
    camera.pitch = 1.07;
    camera.zoom = 48.0;
    let visible = asset.visible_triangles(camera.triangle_culler()).count();

    assert!(visible > 0, "camera culled the entire chunk");
    assert!(
        visible < asset.mesh.triangle_count(),
        "camera culled nothing: {} of {} triangles remain",
        visible,
        asset.mesh.triangle_count()
    );
    assert_eq!(world.state_digest(), before);
}
