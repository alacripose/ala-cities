//! The road network.
//!
//! The graph is rebuilt from the tile grid rather than stored beside it, so
//! there is exactly one source of truth for where a road is and no way for the
//! graph to disagree with the map. A* is ours: the graph is a grid, the
//! heuristic is Manhattan, and the whole thing is under a hundred lines.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use crate::sim::Tile;

/// The largest number of tiles a single road leg may span. A leg beyond this is
/// refused by the governor rather than truncated.
pub const MAX_LEG_TILES: u32 = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoadNode {
    pub tile: u32,
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, Default)]
pub struct RoadGraph {
    pub nodes: Vec<RoadNode>,
    by_tile: HashMap<u32, u32>,
    adj: Vec<Vec<u32>>,
}

impl RoadGraph {
    /// Rebuild from the tiles. Called after any road change; roads are laid
    /// rarely and the map is small, so the cost is not worth optimising away.
    pub fn rebuild(width: u32, height: u32, tiles: &[Tile]) -> Self {
        let mut graph = Self::default();
        for y in 0..height {
            for x in 0..width {
                let tile = y * width + x;
                if tiles[tile as usize].road {
                    let id = graph.nodes.len() as u32;
                    graph.nodes.push(RoadNode { tile, x, y });
                    graph.by_tile.insert(tile, id);
                    graph.adj.push(Vec::new());
                }
            }
        }

        // Connect orthogonal neighbours. Diagonal connections are not roads, so
        // a car cannot cut a corner through two touching tiles.
        for id in 0..graph.nodes.len() as u32 {
            let node = graph.nodes[id as usize];
            let mut neighbours = Vec::with_capacity(4);
            for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let nx = node.x as i32 + dx;
                let ny = node.y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                    continue;
                }
                let neighbour_tile = ny as u32 * width + nx as u32;
                if let Some(&neighbour) = graph.by_tile.get(&neighbour_tile) {
                    neighbours.push(neighbour);
                }
            }
            graph.adj[id as usize] = neighbours;
        }

        graph
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn node_of_tile(&self, tile: u32) -> Option<u32> {
        self.by_tile.get(&tile).copied()
    }

    pub fn neighbours(&self, node: u32) -> &[u32] {
        self.adj.get(node as usize).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// The nearest road node to a tile, searched outwards in rings.
    ///
    /// A building with no road within `radius` returns `None` — which is a
    /// refusal the caller must handle, not a silent fallback to nowhere.
    pub fn nearest_node_within(
        &self,
        width: u32,
        height: u32,
        tile: u32,
        radius: u32,
    ) -> Option<u32> {
        let (tx, ty) = (tile % width, tile / width);
        for r in 0..=radius {
            let mut best: Option<(u32, u32)> = None;
            let ri = r as i32;
            for dy in -ri..=ri {
                for dx in -ri..=ri {
                    // Only the ring at exactly this radius is new work.
                    if dx.abs() != ri && dy.abs() != ri {
                        continue;
                    }
                    let x = tx as i32 + dx;
                    let y = ty as i32 + dy;
                    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
                        continue;
                    }
                    let candidate = y as u32 * width + x as u32;
                    if let Some(&node) = self.by_tile.get(&candidate) {
                        let distance = (dx * dx + dy * dy) as u32;
                        if best.map(|(d, _)| distance < d).unwrap_or(true) {
                            best = Some((distance, node));
                        }
                    }
                }
            }
            if let Some((_, node)) = best {
                return Some(node);
            }
        }
        None
    }

    /// A* over the road graph. Returns the node path, including both ends.
    pub fn astar(&self, from: u32, to: u32) -> Option<Vec<u32>> {
        if from as usize >= self.nodes.len() || to as usize >= self.nodes.len() {
            return None;
        }
        if from == to {
            return Some(vec![from]);
        }

        let heuristic = |node: u32| -> u32 {
            let a = self.nodes[node as usize];
            let b = self.nodes[to as usize];
            (a.x.abs_diff(b.x)) + (a.y.abs_diff(b.y))
        };

        let mut best_cost: HashMap<u32, u32> = HashMap::new();
        let mut came_from: HashMap<u32, u32> = HashMap::new();
        let mut open: BinaryHeap<Reverse<(u32, u32)>> = BinaryHeap::new();

        best_cost.insert(from, 0);
        open.push(Reverse((heuristic(from), from)));

        while let Some(Reverse((_, current))) = open.pop() {
            if current == to {
                let mut path = vec![current];
                let mut cursor = current;
                while let Some(&prev) = came_from.get(&cursor) {
                    path.push(prev);
                    cursor = prev;
                }
                path.reverse();
                return Some(path);
            }

            let current_cost = best_cost.get(&current).copied().unwrap_or(u32::MAX);
            for &next in self.neighbours(current) {
                let step = 1 + self.edge_cost(current, next);
                let tentative = current_cost.saturating_add(step);
                if tentative < best_cost.get(&next).copied().unwrap_or(u32::MAX) {
                    best_cost.insert(next, tentative);
                    came_from.insert(next, current);
                    open.push(Reverse((tentative.saturating_add(heuristic(next)), next)));
                }
            }
        }
        None
    }

    /// Congestion surcharge on an edge, from the live traffic count.
    fn edge_cost(&self, _from: u32, _to: u32) -> u32 {
        // Traffic-aware costing lands with the traffic model in C2. Returning a
        // flat zero keeps every path deterministic in the meantime, which is
        // what a replay needs.
        0
    }

    /// The tile path between two tiles, or `None` when either cannot reach a
    /// road. Callers must treat `None` as a refused commute, never as success.
    pub fn route_between_tiles(
        &self,
        width: u32,
        height: u32,
        from_tile: u32,
        to_tile: u32,
        radius: u32,
    ) -> Option<Vec<u32>> {
        let from = self.nearest_node_within(width, height, from_tile, radius)?;
        let to = self.nearest_node_within(width, height, to_tile, radius)?;
        let nodes = self.astar(from, to)?;
        Some(nodes.iter().map(|&n| self.nodes[n as usize].tile).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::tests_support::blank_tiles;

    fn with_road(width: u32, height: u32, road: &[(u32, u32)]) -> RoadGraph {
        let mut tiles = blank_tiles(width, height);
        for &(x, y) in road {
            tiles[(y * width + x) as usize].road = true;
        }
        RoadGraph::rebuild(width, height, &tiles)
    }

    #[test]
    fn a_straight_road_is_a_connected_chain() {
        let graph = with_road(10, 1, &[(0, 0), (1, 0), (2, 0)]);
        assert_eq!(graph.len(), 3);
        let path = graph.astar(0, 2).expect("path along a straight road");
        assert_eq!(path, vec![0, 1, 2]);
    }

    #[test]
    fn diagonal_touch_is_not_a_connection() {
        let graph = with_road(3, 3, &[(0, 0), (1, 1)]);
        assert_eq!(graph.len(), 2);
        assert!(
            graph.astar(0, 1).is_none(),
            "diagonally touching tiles must not be routable"
        );
    }

    #[test]
    fn disconnected_roads_have_no_path() {
        let graph = with_road(6, 3, &[(0, 0), (5, 2)]);
        assert!(graph.astar(0, 1).is_none());
    }

    #[test]
    fn nearest_node_searches_outwards() {
        let graph = with_road(10, 10, &[(5, 5)]);
        let node = graph.nearest_node_within(10, 10, 5 * 10 + 5, 3);
        assert_eq!(node, Some(0));
        let node = graph.nearest_node_within(10, 10, 5 * 10 + 7, 3);
        assert_eq!(node, Some(0));
        // Seven tiles away is outside a radius of three.
        assert_eq!(graph.nearest_node_within(10, 10, 5 * 10 + 9, 3), None);
    }

    #[test]
    fn a_bend_is_routed_around() {
        // An L: across the top row then down the right column.
        let graph = with_road(5, 5, &[(0, 0), (1, 0), (2, 0), (2, 1), (2, 2)]);
        let path = graph.astar(0, 4).expect("path around the corner");
        assert_eq!(path, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn routing_between_buildings_uses_the_grid() {
        let graph = with_road(6, 1, &[(0, 0), (1, 0), (2, 0), (3, 0)]);
        let path = graph
            .route_between_tiles(6, 1, 0, 3, 2)
            .expect("a route between two road tiles");
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], 0);
        assert_eq!(path[3], 3);
    }

    #[test]
    fn a_building_with_no_road_nearby_refuses_a_route() {
        let graph = with_road(20, 20, &[(0, 0)]);
        assert!(graph.route_between_tiles(20, 20, 19 * 20 + 19, 0, 2).is_none());
    }
}
