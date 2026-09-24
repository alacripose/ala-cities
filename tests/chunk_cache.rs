use ala_cities::asset_generator::DynamicAssetGenerator;
use ala_cities::founding_day::{Action, ChunkCoord, FoundingWorld, GeneratorRevision, WorldSeed};

fn world() -> FoundingWorld {
    FoundingWorld::new(WorldSeed(7), GeneratorRevision(1))
}

#[test]
fn a_typed_mass_delta_invalidates_the_chunk_cache_even_when_geometry_is_unchanged() {
    let mut world = world();
    world
        .act(Action::Land(FoundingWorld::starter_camp()))
        .expect("founder lands");
    let mut generator = DynamicAssetGenerator::default();
    let before = generator
        .build_chunk(&world, ChunkCoord::new(0, 0, 0))
        .expect("chunk is resident");

    world
        .act(Action::Forage(FoundingWorld::starter_forage()))
        .expect("partial forage keeps the forage voxel occupied");

    let after = generator
        .build_chunk(&world, ChunkCoord::new(0, 0, 0))
        .expect("chunk remains resident");

    assert_ne!(before.key, after.key);
    assert_eq!(before.mesh, after.mesh);
    assert_eq!(generator.cache_len(), 2);
}

#[test]
fn neighbor_residency_invalidates_boundary_faces_and_unload_restores_the_old_cache_hit() {
    let mut world = world();
    let chunk = ChunkCoord::new(0, 0, 0);
    let neighbor = ChunkCoord::new(1, 0, 0);
    let mut generator = DynamicAssetGenerator::default();
    let before = generator
        .build_chunk(&world, chunk)
        .expect("chunk is resident");

    assert!(world.load_chunk(neighbor));
    let with_neighbor = generator
        .build_chunk(&world, chunk)
        .expect("chunk remains resident");
    assert_ne!(before.key, with_neighbor.key);
    assert_eq!(generator.cache_len(), 2);

    assert!(world.unload_chunk(neighbor));
    let restored = generator
        .build_chunk(&world, chunk)
        .expect("chunk remains resident");
    assert_eq!(restored, before);
    assert_eq!(generator.cache_len(), 2);
    assert_eq!(
        generator.cached_chunks(),
        std::collections::BTreeSet::from([chunk])
    );
    assert_eq!(generator.evict_chunk(chunk), 2);
    assert_eq!(generator.cache_len(), 0);
}
