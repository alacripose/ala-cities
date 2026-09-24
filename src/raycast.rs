//! Exact voxel ray traversal for arbitrary 3D directions.
//!
//! This is the Amanatides–Woo grid traversal algorithm: advance to the next
//! voxel boundary on each axis, take the nearest boundary, and report the face
//! crossed when an occupied voxel is entered. Signed coordinates and diagonal
//! rays are first-class; no screen-space tile approximation is used.

use crate::founding_day::Coord;
use glam::Vec3;

/// The face crossed when a ray enters a voxel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

impl Face {
    pub const fn normal(self) -> Vec3 {
        match self {
            Self::PosX => Vec3::X,
            Self::NegX => Vec3::NEG_X,
            Self::PosY => Vec3::Y,
            Self::NegY => Vec3::NEG_Y,
            Self::PosZ => Vec3::Z,
            Self::NegZ => Vec3::NEG_Z,
        }
    }
}

/// The first occupied voxel reached by a ray.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoxelHit {
    pub coord: Coord,
    pub previous: Option<Coord>,
    pub normal: Option<Face>,
    pub distance: f32,
}

fn cell_coord(value: f32) -> Option<i32> {
    let floor = value.floor();
    if !floor.is_finite() || floor < i32::MIN as f32 || floor > i32::MAX as f32 {
        return None;
    }
    Some(floor as i32)
}

fn stepped(coord: Coord, axis: usize, step: i32) -> Option<Coord> {
    let next = match axis {
        0 => coord.x.checked_add(step)?,
        1 => coord.y.checked_add(step)?,
        2 => coord.z.checked_add(step)?,
        _ => unreachable!("ray has three axes"),
    };
    Some(match axis {
        0 => Coord::new(next, coord.y, coord.z),
        1 => Coord::new(coord.x, next, coord.z),
        2 => Coord::new(coord.x, coord.y, next),
        _ => unreachable!("ray has three axes"),
    })
}

fn entered_face(axis: usize, step: i32) -> Face {
    match (axis, step) {
        (0, 1) => Face::NegX,
        (0, -1) => Face::PosX,
        (1, 1) => Face::NegY,
        (1, -1) => Face::PosY,
        (2, 1) => Face::NegZ,
        (2, -1) => Face::PosZ,
        _ => unreachable!("a traversed axis always has a non-zero step"),
    }
}

/// Cast through signed integer voxels and return the first occupied cell.
///
/// `occupied` is queried once per visited cell. A ray that starts inside a
/// voxel returns that voxel at distance zero with no entry face; every later
/// hit carries the face crossed to enter it.
pub fn raycast_voxels<F>(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    mut occupied: F,
) -> Option<VoxelHit>
where
    F: FnMut(Coord) -> bool,
{
    if !origin.is_finite()
        || !direction.is_finite()
        || !max_distance.is_finite()
        || max_distance < 0.0
    {
        return None;
    }
    let length_squared = direction.length_squared();
    if length_squared <= f32::EPSILON {
        return None;
    }
    let direction = direction / length_squared.sqrt();

    let mut current = Coord::new(
        cell_coord(origin.x)?,
        cell_coord(origin.y)?,
        cell_coord(origin.z)?,
    );
    if occupied(current) {
        return Some(VoxelHit {
            coord: current,
            previous: None,
            normal: None,
            distance: 0.0,
        });
    }

    let origin_cell = [origin.x, origin.y, origin.z];
    let direction_axis = [direction.x, direction.y, direction.z];
    let current_axis = [current.x, current.y, current.z];
    let mut steps = [0_i32; 3];
    let mut next_distance = [f32::INFINITY; 3];
    let mut step_distance = [f32::INFINITY; 3];

    for axis in 0..3 {
        if direction_axis[axis] == 0.0 {
            continue;
        }
        let step = if direction_axis[axis] > 0.0 { 1 } else { -1 };
        steps[axis] = step;
        let boundary = if step > 0 {
            current_axis[axis] as f32 + 1.0
        } else {
            current_axis[axis] as f32
        };
        next_distance[axis] = ((boundary - origin_cell[axis]) / direction_axis[axis]).max(0.0);
        step_distance[axis] = (1.0 / direction_axis[axis]).abs();
    }

    loop {
        let axis = next_distance
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(axis, _)| axis)?;
        let distance = next_distance[axis];
        if distance > max_distance {
            return None;
        }
        next_distance[axis] += step_distance[axis];

        let previous = current;
        current = stepped(current, axis, steps[axis])?;
        if occupied(current) {
            return Some(VoxelHit {
                coord: current,
                previous: Some(previous),
                normal: Some(entered_face(axis, steps[axis])),
                distance,
            });
        }
    }
}
