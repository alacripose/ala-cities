use ala_cities::asset_generator::{AssetKind, AssetRecipe, DynamicAssetGenerator, ShapeControl};
use ala_cities::founding_day::{ChunkCoord, Coord, FoundingWorld, GeneratorRevision, WorldSeed};
use ala_cities::raycast::Face;
use glam::Vec3;

fn world() -> FoundingWorld {
    FoundingWorld::new(WorldSeed(7), GeneratorRevision(1))
}

#[test]
fn semantic_recipe_derives_a_stable_hash_without_mutating_world_truth() {
    let world = world();
    let before = world.state_digest();
    let mut generator = DynamicAssetGenerator::new(AssetRecipe::default());
    let original = generator
        .build_shape(AssetKind::SettingsGear)
        .expect("settings gear is supported");
    let original_hash = generator.builder_hash();

    assert!(generator.set_control(ShapeControl::Teeth, 8.125));
    let changed = generator
        .build_shape(AssetKind::SettingsGear)
        .expect("settings gear is supported");

    assert_ne!(original_hash, generator.builder_hash());
    assert_ne!(original.key, changed.key);
    assert_eq!(original.recipe.teeth, AssetRecipe::default().teeth);
    assert_eq!(world.state_digest(), before);
}

#[test]
fn a_gear_is_one_mesh_with_a_real_opening_and_accent_geometry() {
    let mut generator = DynamicAssetGenerator::default();
    let asset = generator
        .build_shape(AssetKind::SettingsGear)
        .expect("settings gear is supported");

    assert_eq!(asset.mesh.mesh_count(), 1);
    assert!(asset.mesh.vertex_count() > 100);
    assert!(asset.mesh.triangle_count() > 100);
    assert!(asset
        .mesh
        .vertices
        .iter()
        .any(|vertex| vertex.material == ala_cities::asset_generator::ShapeMaterial::Accent));

    let inner_radius = asset
        .mesh
        .inner_radius()
        .expect("gear records its opening radius");
    let outer_radius = asset
        .mesh
        .outer_radius()
        .expect("gear records its tooth radius");
    assert!(inner_radius > 0.2 && inner_radius < 0.5);
    assert!(outer_radius > inner_radius + 0.5);
}

#[test]
fn teeth_and_opening_are_continuous_semantic_controls() {
    let mut generator = DynamicAssetGenerator::default();
    let a = generator
        .build_shape(AssetKind::SettingsGear)
        .expect("settings gear is supported");
    assert!(generator.set_control(ShapeControl::Teeth, 8.125));
    let b = generator
        .build_shape(AssetKind::SettingsGear)
        .expect("settings gear is supported");
    assert!(generator.set_control(ShapeControl::Teeth, 8.25));
    let c = generator
        .build_shape(AssetKind::SettingsGear)
        .expect("settings gear is supported");

    assert_ne!(a.digest, b.digest);
    assert_ne!(b.digest, c.digest);
    assert_eq!(a.mesh.triangle_count(), c.mesh.triangle_count());
}

#[test]
fn shape_recipe_never_divides_or_rescales_the_world_chunk_mesh() {
    let world = world();
    let chunk = ChunkCoord::new(0, 0, 0);
    let mut generator = DynamicAssetGenerator::default();
    let before = generator
        .build_chunk(&world, chunk)
        .expect("starter chunk is resident");

    assert!(generator.set_control(ShapeControl::Accent, 0.875));
    let after = generator
        .build_chunk(&world, chunk)
        .expect("starter chunk is resident");

    assert_eq!(before.quads, after.quads);
    assert_eq!(before.octree.digest(), after.octree.digest());
    assert_eq!(generator.cache_len(), 1);
}

#[test]
fn chunk_build_remains_sparse_cached_and_octree_ray_queryable() {
    let world = world();
    let chunk = ChunkCoord::new(0, 0, 0);
    let before = world.state_digest();
    let mut generator = DynamicAssetGenerator::default();

    let first = generator
        .build_chunk(&world, chunk)
        .expect("starter chunk is resident");
    let second = generator
        .build_chunk(&world, chunk)
        .expect("starter chunk is resident");

    assert_eq!(first, second);
    assert_eq!(generator.cache_len(), 1);
    assert!(first.octree.occupied_count() < first.octree.capacity());
    assert!(first.octree.node_count() < first.octree.max_nodes());
    assert!(!first.quads.is_empty());
    assert_eq!(world.state_digest(), before);

    let hit = first
        .raycast(Vec3::new(10.5, 10.5, 5.0), Vec3::NEG_Z, 10.0)
        .expect("octree-backed chunk ray hits terrain");
    assert_eq!(hit.coord, Coord::new(10, 10, 0));
    assert_eq!(hit.normal, Some(Face::PosZ));
    assert!((hit.distance - 4.0).abs() < 1e-5);
}
