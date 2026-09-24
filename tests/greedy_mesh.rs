use std::collections::BTreeSet;

use ala_cities::asset_generator::{DynamicAssetGenerator, MeshTriangle};
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

fn expected_faces(world: &FoundingWorld, chunk: ChunkCoord) -> BTreeSet<(Coord, Coord, VoxelKind)> {
    let mut expected = BTreeSet::new();
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
                expected.insert((coord, direction, voxel.kind));
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

fn axis_value(value: Vec3, axis: usize) -> f32 {
    match axis {
        0 => value.x,
        1 => value.y,
        _ => value.z,
    }
}

fn triangle_plane(triangle: &MeshTriangle) -> (usize, i32, Coord, [usize; 2]) {
    let normal = triangle.vertices[0].normal;
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
    let plane = axis_value(triangle.vertices[0].position, axis).floor() as i32;
    let normal_coord = if sign > 0 { plane - 1 } else { plane };
    let tangents = match axis {
        0 => [1, 2],
        1 => [0, 2],
        _ => [0, 1],
    };
    (axis, sign, normal_coord_value(axis, normal_coord), tangents)
}

fn normal_coord_value(axis: usize, value: i32) -> Coord {
    match axis {
        0 => Coord::new(value, 0, 0),
        1 => Coord::new(0, value, 0),
        _ => Coord::new(0, 0, value),
    }
}

type ProjectedTriangle = ([f32; 2], [f32; 2], [f32; 2], [i32; 2], [i32; 2]);

fn project_triangle(triangle: &MeshTriangle) -> ProjectedTriangle {
    let (_, _, _, tangents) = triangle_plane(triangle);
    let projected: Vec<[f32; 2]> = triangle
        .vertices
        .iter()
        .map(|vertex| {
            [
                axis_value(vertex.position, tangents[0]),
                axis_value(vertex.position, tangents[1]),
            ]
        })
        .collect();
    let minimum = [
        projected
            .iter()
            .map(|point| point[0])
            .fold(f32::INFINITY, f32::min)
            .floor() as i32,
        projected
            .iter()
            .map(|point| point[1])
            .fold(f32::INFINITY, f32::min)
            .floor() as i32,
    ];
    let maximum = [
        projected
            .iter()
            .map(|point| point[0])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil() as i32,
        projected
            .iter()
            .map(|point| point[1])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil() as i32,
    ];
    (
        [projected[0][0], projected[0][1]],
        [projected[1][0], projected[1][1]],
        [projected[2][0], projected[2][1]],
        minimum,
        maximum,
    )
}

fn barycentric_contains(triangle: [[f32; 2]; 3], point: [f32; 2]) -> bool {
    let [a, b, c] = triangle;
    let denominator = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
    if denominator.abs() <= f32::EPSILON {
        return false;
    }
    let first =
        ((b[1] - c[1]) * (point[0] - c[0]) + (c[0] - b[0]) * (point[1] - c[1])) / denominator;
    let second =
        ((c[1] - a[1]) * (point[0] - c[0]) + (a[0] - c[0]) * (point[1] - c[1])) / denominator;
    let third = 1.0 - first - second;
    first >= -1e-5 && second >= -1e-5 && third >= -1e-5
}

fn triangle_faces(triangle: &MeshTriangle) -> BTreeSet<(Coord, Coord, VoxelKind)> {
    let (axis, sign, normal_coord, _) = triangle_plane(triangle);
    let (a, b, c, minimum, maximum) = project_triangle(triangle);
    let direction = match axis {
        0 => Coord::new(sign, 0, 0),
        1 => Coord::new(0, sign, 0),
        _ => Coord::new(0, 0, sign),
    };
    let mut faces = BTreeSet::new();
    for first in minimum[0]..maximum[0] {
        for second in minimum[1]..maximum[1] {
            let center = [first as f32 + 0.5, second as f32 + 0.5];
            if barycentric_contains([a, b, c], center) {
                let coord = match axis {
                    0 => Coord::new(normal_coord.x, first, second),
                    1 => Coord::new(first, normal_coord.y, second),
                    _ => Coord::new(first, second, normal_coord.z),
                };
                faces.insert((coord, direction, triangle.material));
            }
        }
    }
    faces
}

fn assert_outward_winding(triangle: &MeshTriangle) {
    let edge_a = triangle.vertices[1].position - triangle.vertices[0].position;
    let edge_b = triangle.vertices[2].position - triangle.vertices[0].position;
    let winding = edge_a.cross(edge_b).normalize();
    assert!(
        winding.dot(triangle.vertices[0].normal) > 0.999,
        "triangle winding {:?} disagrees with declared normal {:?}",
        winding,
        triangle.vertices[0].normal
    );
}

#[test]
fn greedy_indexed_triangles_cover_exactly_the_visible_faces_and_materials() {
    for chunk in [ChunkCoord::new(0, 0, 0), ChunkCoord::new(-1, 0, 0)] {
        let mut world = world();
        world.load_chunk(chunk);
        let asset = DynamicAssetGenerator::default()
            .build_chunk(&world, chunk)
            .expect("chunk builds");
        let expected = expected_faces(&world, chunk);
        let mut actual = BTreeSet::new();

        assert_eq!(asset.mesh.indices.len() % 3, 0);
        assert!(asset
            .mesh
            .indices
            .iter()
            .all(|index| (*index as usize) < asset.mesh.vertices.len()));
        for triangle in asset.mesh.triangles() {
            assert_outward_winding(&triangle);
            actual.extend(triangle_faces(&triangle));
        }

        assert_eq!(actual, expected);
    }
}

#[test]
fn greedy_triangles_reduce_the_naive_triangle_count() {
    let world = world();
    let chunk = ChunkCoord::new(0, 0, 0);
    let expected_count = expected_faces(&world, chunk).len();
    let asset = DynamicAssetGenerator::default()
        .build_chunk(&world, chunk)
        .expect("chunk builds");

    assert!(
        asset.mesh.triangle_count() * 4 < expected_count * 3,
        "expected triangle reduction, got {} triangles for {expected_count} visible faces",
        asset.mesh.triangle_count()
    );
}
