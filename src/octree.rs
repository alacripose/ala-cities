//! A sparse 32³ voxel octree used by the target presentation builder.
//!
//! Empty air is implicit. Each occupied coordinate allocates only the branch
//! path to its leaf, so a surface chunk remains sparse instead of paying for
//! 32³ empty cells.

use crate::founding_day::{Coord, VoxelKind};
use crate::raycast::{raycast_voxels, VoxelHit};
use glam::Vec3;

/// Five halvings from one 32-wide root to one-voxel leaves.
pub const OCTREE_DEPTH: u8 = 5;
pub const OCTREE_CAPACITY: usize = 32 * 32 * 32;
pub const OCTREE_MAX_NODES: usize = 37_449;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Node {
    Empty,
    Leaf(Option<VoxelKind>),
    Branch([Option<Box<Node>>; 8]),
}

impl Node {
    fn empty_for_depth(depth: u8) -> Self {
        if depth == OCTREE_DEPTH {
            Self::Leaf(None)
        } else {
            Self::Branch(std::array::from_fn(|_| None))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SparseVoxelOctree {
    root: Node,
    nodes: usize,
    occupied: usize,
}

impl Default for SparseVoxelOctree {
    fn default() -> Self {
        Self {
            root: Node::Empty,
            nodes: 1,
            occupied: 0,
        }
    }
}

impl SparseVoxelOctree {
    fn branch_index(coord: Coord, depth: u8) -> usize {
        let bit = OCTREE_DEPTH - 1 - depth;
        let x = ((coord.x as u32 >> bit) & 1) as usize;
        let y = ((coord.y as u32 >> bit) & 1) as usize;
        let z = ((coord.z as u32 >> bit) & 1) as usize;
        x | (y << 1) | (z << 2)
    }

    fn insert_at(
        node: &mut Node,
        coord: Coord,
        depth: u8,
        kind: VoxelKind,
    ) -> (Option<VoxelKind>, usize) {
        if depth == OCTREE_DEPTH {
            let was_empty = matches!(node, Node::Empty);
            if was_empty {
                *node = Node::Leaf(None);
            }
            let Node::Leaf(slot) = node else {
                unreachable!("depth-five nodes are leaves");
            };
            return (slot.replace(kind), usize::from(was_empty));
        }

        if matches!(node, Node::Empty) {
            *node = Node::Branch(std::array::from_fn(|_| None));
        }
        let Node::Branch(children) = node else {
            unreachable!("nodes above depth five are branches");
        };
        let index = Self::branch_index(coord, depth);
        let created_child = children[index].is_none();
        if created_child {
            children[index] = Some(Box::new(Node::empty_for_depth(depth + 1)));
        }
        let child = children[index]
            .as_mut()
            .expect("the selected child exists after allocation");
        let (previous, created_below) = Self::insert_at(child, coord, depth + 1, kind);
        (previous, created_below + usize::from(created_child))
    }

    fn get_at(node: &Node, coord: Coord, depth: u8) -> Option<VoxelKind> {
        if depth == OCTREE_DEPTH {
            let Node::Leaf(kind) = node else {
                return None;
            };
            return *kind;
        }
        let Node::Branch(children) = node else {
            return None;
        };
        children[Self::branch_index(coord, depth)]
            .as_deref()
            .and_then(|child| Self::get_at(child, coord, depth + 1))
    }

    pub fn insert(&mut self, coord: Coord, kind: VoxelKind) -> Option<VoxelKind> {
        if !(0..32).contains(&coord.x) || !(0..32).contains(&coord.y) || !(0..32).contains(&coord.z)
        {
            return None;
        }
        let (previous, created) = Self::insert_at(&mut self.root, coord, 0, kind);
        self.nodes += created;
        if previous.is_none() {
            self.occupied += 1;
        }
        previous
    }

    pub fn get(&self, coord: Coord) -> Option<VoxelKind> {
        if !(0..32).contains(&coord.x) || !(0..32).contains(&coord.y) || !(0..32).contains(&coord.z)
        {
            return None;
        }
        Self::get_at(&self.root, coord, 0)
    }

    pub const fn node_count(&self) -> usize {
        self.nodes
    }

    pub const fn occupied_count(&self) -> usize {
        self.occupied
    }

    pub const fn capacity(&self) -> usize {
        OCTREE_CAPACITY
    }

    pub const fn max_nodes(&self) -> usize {
        OCTREE_MAX_NODES
    }

    pub fn is_sparse(&self) -> bool {
        self.occupied < self.capacity() && self.nodes < self.max_nodes()
    }

    pub fn raycast_local(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<VoxelHit> {
        raycast_voxels(origin, direction, max_distance, |coord| {
            self.get(coord).is_some()
        })
    }

    pub fn digest(&self) -> u64 {
        fn feed(hash: &mut u64, value: u8) {
            *hash ^= u64::from(value);
            *hash = hash.wrapping_mul(0x1000_0000_01b3);
        }
        fn kind_byte(kind: VoxelKind) -> u8 {
            match kind {
                VoxelKind::Soil => 1,
                VoxelKind::Forage => 2,
                VoxelKind::Wood => 3,
            }
        }
        fn visit(hash: &mut u64, node: &Node) {
            match node {
                Node::Empty => feed(hash, 0),
                Node::Leaf(None) => feed(hash, 1),
                Node::Leaf(Some(kind)) => {
                    feed(hash, 2);
                    feed(hash, kind_byte(*kind));
                }
                Node::Branch(children) => {
                    feed(hash, 3);
                    for child in children {
                        match child {
                            Some(child) => visit(hash, child),
                            None => feed(hash, 0),
                        }
                    }
                }
            }
        }
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        visit(&mut hash, &self.root);
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::{SparseVoxelOctree, OCTREE_DEPTH, OCTREE_MAX_NODES};
    use crate::founding_day::{Coord, VoxelKind};

    #[test]
    fn node_count_counts_each_allocated_node_once() {
        let mut octree = SparseVoxelOctree::default();
        assert_eq!(octree.node_count(), 1, "the root exists before insertion");
        assert_eq!(octree.occupied_count(), 0);

        assert!(octree
            .insert(Coord::new(0, 0, 0), VoxelKind::Soil)
            .is_none());
        assert_eq!(octree.node_count(), 1 + OCTREE_DEPTH as usize);
        assert_eq!(octree.occupied_count(), 1);

        let before_replacement = octree.node_count();
        assert_eq!(
            octree.insert(Coord::new(0, 0, 0), VoxelKind::Wood),
            Some(VoxelKind::Soil)
        );
        assert_eq!(octree.node_count(), before_replacement);
        assert_eq!(octree.occupied_count(), 1);

        assert!(octree
            .insert(Coord::new(16, 16, 16), VoxelKind::Soil)
            .is_none());
        assert_eq!(
            octree.node_count(),
            before_replacement + OCTREE_DEPTH as usize,
            "a distinct top-level octant adds one five-node path"
        );
        assert_eq!(octree.occupied_count(), 2);
    }

    #[test]
    fn a_full_chunk_has_exactly_the_maximum_node_count() {
        let mut octree = SparseVoxelOctree::default();
        for z in 0..32 {
            for y in 0..32 {
                for x in 0..32 {
                    octree.insert(Coord::new(x, y, z), VoxelKind::Soil);
                }
            }
        }
        assert_eq!(octree.occupied_count(), octree.capacity());
        assert_eq!(octree.node_count(), OCTREE_MAX_NODES);
        assert!(!octree.is_sparse());
    }
}
