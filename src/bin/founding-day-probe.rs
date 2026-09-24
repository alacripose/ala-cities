use ala_cities::founding_day::{
    Action, FoundingWorld, GeneratorRevision, Phase, WorldSeed, DAY_TICKS, NIGHT_TICKS,
};

fn main() {
    let mut world = FoundingWorld::new(WorldSeed(7), GeneratorRevision(1));
    let camp = FoundingWorld::starter_camp();
    let forage = FoundingWorld::starter_forage();
    let wood = FoundingWorld::starter_wood();

    println!(
        "initial: phase={} tick={} digest={:016x}",
        world.phase,
        world.tick,
        world.state_digest()
    );

    let actions = [
        Action::Land(camp),
        Action::Forage(forage),
        Action::Forage(forage),
        Action::Forage(forage),
        Action::Forage(forage),
        Action::GatherWood(wood),
        Action::GatherWood(wood),
        Action::BuildFire(camp),
        Action::BuildShelter(camp),
    ];
    for (index, action) in actions.into_iter().enumerate() {
        match world.act(action) {
            Ok(()) => println!("accepted: {action}"),
            Err(refusal) => {
                eprintln!("refused: {action}: {refusal}");
                std::process::exit(1);
            }
        }

        if index == 1 {
            let starter_chunk = camp.chunk();
            let harvested_forage = world
                .voxel_at(forage)
                .expect("partially harvested forage remains resident");
            let before_unload = world.state_digest();
            assert!(world.unload_chunk(starter_chunk));
            assert!(!world.chunks.contains_key(&starter_chunk));
            assert_eq!(world.state_digest(), before_unload);
            println!(
                "unloaded: chunk={},{},{} digest={:016x}",
                starter_chunk.x, starter_chunk.y, starter_chunk.z, before_unload
            );
            assert!(world.load_chunk(starter_chunk));
            assert_eq!(world.voxel_at(forage), Some(harvested_forage));
            assert_eq!(world.state_digest(), before_unload);
            println!(
                "rederived: forage_mass={}g digest={:016x}",
                harvested_forage.mass_grams, before_unload
            );
        }
    }

    // No action below advances time. The world reaches night and dawn itself.
    world.advance_ticks(DAY_TICKS + NIGHT_TICKS);
    let snapshot = world.snapshot();
    println!(
        "final: phase={} tick={} day={} night={} food={}g wood={}g balanced={} digest={:016x}",
        snapshot.phase,
        snapshot.tick,
        snapshot.day_elapsed,
        snapshot.night_elapsed,
        snapshot.carried_forage_grams,
        snapshot.carried_wood_grams,
        snapshot.material_balanced,
        snapshot.digest,
    );
    assert_eq!(snapshot.phase, Phase::Dawn);
    assert!(snapshot.material_balanced);
}
