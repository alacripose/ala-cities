use std::collections::BTreeSet;

use ala_cities::asset_generator::DynamicAssetGenerator;
use ala_cities::founding_day::{ChunkCoord, FoundingWorld, GeneratorRevision, WorldSeed};
use ala_cities::render::{Camera, Screen};
use glam::Vec3;

#[test]
fn the_next_preview_builds_a_whole_cubic_chunk() {
    let chunk_coord = ChunkCoord::new(0, 0, 0);
    let world = FoundingWorld::cubic_preview(WorldSeed(7), GeneratorRevision(1), chunk_coord);
    let resident = world.chunk(chunk_coord).expect("cubic preview is resident");

    assert_eq!(resident.voxels.len(), 32 * 32 * 32);

    let asset = DynamicAssetGenerator::default()
        .build_chunk(&world, chunk_coord)
        .expect("cubic chunk builds");
    assert_eq!(asset.mesh.triangle_count(), 12);

    let normals = asset
        .mesh
        .triangles()
        .map(|triangle| {
            let normal = triangle.vertices[0].normal;
            [normal.x as i32, normal.y as i32, normal.z as i32]
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        normals.len(),
        6,
        "a cube exposes one outward direction per face"
    );

    let mut camera = Camera::new(
        Screen {
            w: 1280.0,
            h: 800.0,
        },
        32,
        32,
    );
    camera.focus = Vec3::new(16.0, 16.0, 16.0);
    camera.zoom = 24.0;
    let visible = asset.visible_triangles(camera.triangle_culler()).count();
    assert!(visible > 0);
    assert!(visible < asset.mesh.triangle_count());

    let (near, far) = camera.ray(640.0, 400.0);
    let hit = asset
        .raycast(near, far - near, (far - near).length() + 1.0)
        .expect("camera ray hits the cubic preview");
    assert!((0..32).contains(&hit.coord.x));
    assert!((0..32).contains(&hit.coord.y));
    assert!((0..32).contains(&hit.coord.z));
}
