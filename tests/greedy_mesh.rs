use std::collections::BTreeMap;

use ala_cities::asset_generator::{DynamicAssetGenerator, MeshQuad};
use ala_cities::founding_day::{
    ChunkCoord, Coord, FoundingWorld, GeneratorRevision, VoxelKind, WorldSeed, CHUNK_SIZE,
};
use glam::Vec3;

const DIRECTIONS: [Coord; 6] = [
    Coord::new(1, 0, 0),
    Coord::new(-1, 0, 0),
    Coord::new(0, 1, 0),
    Coord::new(0, -1, 0),
    Coord::new(0, 0, 1),
    Coord::new(0, 0, -1),
];

fn world() -> FoundingWorld {
    FoundingWorld::new(WorldSeed(7), GeneratorRevision(1))
}

fn expected_faces(world: &FoundingWorld, chunk: ChunkCoord) -> BTreeMap<(Coord, Coord), VoxelKind> {
    let mut expected = BTreeMap::new();
    let resident = world.chunk(chunk).expect("test chunk is resident");
    for (local, voxel) in &resident.voxels {
        let coord = world_coord(chunk, *local);
        for direction in DIRECTIONS {
            let neighbour = Coord::new(
                coord.x + direction.x,
                coord.y + direction.y,
                coord.z + direction.z,
            );
            if world.voxel_at(neighbour).is_none() {
                expected.insert((coord, direction), voxel.kind);
            }
        }
    }
    expected
}

fn world_coord(chunk: ChunkCoord, local: Coord) -> Coord {
    Coord::new(
        chunk.x * CHUNK_SIZE + local.x,
        chunk.y * CHUNK_SIZE + local.y,
        chunk.z * CHUNK_SIZE + local.z,
    )
}

fn quad_faces(quad: &MeshQuad) -> Vec<(Coord, Coord)> {
    let normal = quad.vertices[0].normal;
    let axis = if normal.x.abs() > normal.y.abs() && normal.x.abs() > normal.z.abs() {
        0
    } else if normal.y.abs() > normal.z.abs() {
        1
    } else {
        2
    };
    let sign = if axis_value(normal, axis) > 0.0 {
        1
    } else {
        -1
    };
    let plane = axis_value(quad.vertices[0].position, axis).floor() as i32;
    let normal_coord = if sign > 0 { plane - 1 } else { plane };
    let tangents = match axis {
        0 => [1, 2],
        1 => [0, 2],
        _ => [0, 1],
    };
    let mut minimum = [i32::MAX; 2];
    let mut maximum = [i32::MIN; 2];
    for vertex in &quad.vertices {
        for (slot, axis) in tangents.into_iter().enumerate() {
            let value = axis_value(vertex.position, axis) as i32;
            minimum[slot] = minimum[slot].min(value);
            maximum[slot] = maximum[slot].max(value);
        }
    }

    let mut faces = Vec::new();
    for first in minimum[0]..maximum[0] {
        for second in minimum[1]..maximum[1] {
            let coord = match axis {
                0 => Coord::new(normal_coord, first, second),
                1 => Coord::new(first, normal_coord, second),
                _ => Coord::new(first, second, normal_coord),
            };
            let direction = match axis {
                0 => Coord::new(sign, 0, 0),
                1 => Coord::new(0, sign, 0),
                _ => Coord::new(0, 0, sign),
            };
            faces.push((coord, direction));
        }
    }
    faces
}

fn assert_outward_winding(quad: &MeshQuad) {
    let edge_across = quad.vertices[1].position - quad.vertices[0].position;
    let edge_up = quad.vertices[3].position - quad.vertices[0].position;
    let winding = edge_across.cross(edge_up).normalize();
    assert!(
        winding.dot(quad.vertices[0].normal) > 0.999,
        "triangle winding {:?} disagrees with declared normal {:?}",
        winding,
        quad.vertices[0].normal
    );
}

fn axis_value(value: Vec3, axis: usize) -> f32 {
    match axis {
        0 => value.x,
        1 => value.y,
        _ => value.z,
    }
}

#[test]
fn greedy_quads_cover_exactly_the_same_visible_faces_and_materials() {
    for chunk in [ChunkCoord::new(0, 0, 0), ChunkCoord::new(-1, 0, 0)] {
        let mut world = world();
        world.load_chunk(chunk);
        let asset = DynamicAssetGenerator::default()
            .build_chunk(&world, chunk)
            .expect("chunk builds");
        let expected = expected_faces(&world, chunk);
        let mut actual = BTreeMap::new();

        for quad in &asset.quads {
            assert_outward_winding(quad);
            for face in quad_faces(quad) {
                assert!(
                    actual.insert(face, quad.material).is_none(),
                    "greedy quads overlap at {face:?}"
                );
            }
        }

        assert_eq!(actual, expected);
    }
}

#[test]
fn coplanar_faces_merge_without_changing_the_surface() {
    let world = world();
    let chunk = ChunkCoord::new(0, 0, 0);
    let expected_count = expected_faces(&world, chunk).len();
    let asset = DynamicAssetGenerator::default()
        .build_chunk(&world, chunk)
        .expect("chunk builds");

    assert!(
        asset.quads.len() * 2 < expected_count,
        "expected greedy reduction, got {} quads for {expected_count} visible faces",
        asset.quads.len()
    );
}
