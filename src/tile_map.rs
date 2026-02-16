use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
    Grass,
    Water,
    Stone,
    Dirt,
    Sand,
}

impl Terrain {
    pub fn is_walkable(self) -> bool {
        matches!(self, Terrain::Grass | Terrain::Dirt | Terrain::Sand)
    }
}

#[derive(Debug)]
pub struct TileMap {
    width: usize,
    height: usize,
    terrain: Vec<Terrain>,
    temperature: Vec<f32>,
}

impl TileMap {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            terrain: vec![Terrain::Grass; size],
            temperature: vec![20.0; size],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    pub fn get_terrain(&self, x: usize, y: usize) -> Option<Terrain> {
        self.index(x, y).map(|i| self.terrain[i])
    }

    pub fn set_terrain(&mut self, x: usize, y: usize, t: Terrain) {
        if let Some(i) = self.index(x, y) {
            self.terrain[i] = t;
        }
    }

    pub fn get_temperature(&self, x: usize, y: usize) -> Option<f32> {
        self.index(x, y).map(|i| self.temperature[i])
    }

    pub fn set_temperature(&mut self, x: usize, y: usize, temp: f32) {
        if let Some(i) = self.index(x, y) {
            self.temperature[i] = temp;
        }
    }

    /// Check if a tile is walkable (in-bounds and terrain allows passage).
    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        self.get_terrain(x, y).is_some_and(|t| t.is_walkable())
    }

    /// A* pathfinding from start to goal on the tile grid (8-directional, √2 diagonal cost).
    /// Returns the path as positions from start (exclusive) to goal (inclusive).
    /// Returns None if no path exists within the search limit.
    pub fn find_path(&self, start: (i32, i32), goal: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        if start == goal {
            return Some(Vec::new());
        }

        let map_w = self.width as i32;
        let map_h = self.height as i32;

        // Bounds check
        if start.0 < 0 || start.0 >= map_w || start.1 < 0 || start.1 >= map_h {
            return None;
        }
        if goal.0 < 0 || goal.0 >= map_w || goal.1 < 0 || goal.1 >= map_h {
            return None;
        }

        const CARDINAL_COST: u32 = 100;
        const DIAGONAL_COST: u32 = 141; // √2 × 100, truncated

        const MAX_EXPANDED: usize = 8192;
        const DIRS: [(i32, i32); 8] = [
            (0, -1),
            (1, -1),
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
        ];

        // Octile distance heuristic (consistent for 8-dir with √2 diagonal cost)
        let heuristic = |a: (i32, i32), b: (i32, i32)| -> u32 {
            let dx = (a.0 - b.0).unsigned_abs();
            let dy = (a.1 - b.1).unsigned_abs();
            let diag = dx.min(dy);
            let card = dx.max(dy) - diag;
            diag * DIAGONAL_COST + card * CARDINAL_COST
        };

        // Open set: min-heap of (f_score, x, y)
        let mut open: BinaryHeap<Reverse<(u32, i32, i32)>> = BinaryHeap::new();
        let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
        let mut g_score: HashMap<(i32, i32), u32> = HashMap::new();
        let mut closed: HashSet<(i32, i32)> = HashSet::new();

        g_score.insert(start, 0);
        open.push(Reverse((heuristic(start, goal), start.0, start.1)));

        let mut expanded = 0;

        while let Some(Reverse((_, cx, cy))) = open.pop() {
            let current = (cx, cy);

            if current == goal {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = goal;
                while node != start {
                    path.push(node);
                    node = came_from[&node];
                }
                path.reverse();
                return Some(path);
            }

            if !closed.insert(current) {
                continue;
            }

            expanded += 1;
            if expanded > MAX_EXPANDED {
                return None;
            }

            let current_g = g_score[&current];

            for (dx, dy) in DIRS {
                let nx = cx + dx;
                let ny = cy + dy;
                let neighbor = (nx, ny);

                if nx < 0 || nx >= map_w || ny < 0 || ny >= map_h {
                    continue;
                }

                // Goal is always reachable (entity/item is already there)
                if neighbor != goal && !self.is_walkable(nx as usize, ny as usize) {
                    continue;
                }

                if closed.contains(&neighbor) {
                    continue;
                }

                let is_diagonal = dx != 0 && dy != 0;
                let step_cost = if is_diagonal {
                    DIAGONAL_COST
                } else {
                    CARDINAL_COST
                };
                let new_g = current_g + step_cost;

                if new_g < *g_score.get(&neighbor).unwrap_or(&u32::MAX) {
                    g_score.insert(neighbor, new_g);
                    came_from.insert(neighbor, current);
                    let f = new_g + heuristic(neighbor, goal);
                    open.push(Reverse((f, nx, ny)));
                }
            }
        }

        None
    }
}

/// Returns true if the step from `a` to `b` is diagonal (both axes change).
pub fn is_diagonal_step(a: (i32, i32), b: (i32, i32)) -> bool {
    a.0 != b.0 && a.1 != b.1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let map = TileMap::new(4, 3);
        assert_eq!(map.width(), 4);
        assert_eq!(map.height(), 3);
        assert_eq!(map.get_terrain(0, 0), Some(Terrain::Grass));
        assert_eq!(map.get_temperature(0, 0), Some(20.0));
    }

    #[test]
    fn test_get_set_terrain() {
        let mut map = TileMap::new(10, 10);
        assert_eq!(map.get_terrain(3, 5), Some(Terrain::Grass));

        map.set_terrain(3, 5, Terrain::Water);
        assert_eq!(map.get_terrain(3, 5), Some(Terrain::Water));

        map.set_terrain(3, 5, Terrain::Stone);
        assert_eq!(map.get_terrain(3, 5), Some(Terrain::Stone));

        map.set_terrain(0, 0, Terrain::Dirt);
        assert_eq!(map.get_terrain(0, 0), Some(Terrain::Dirt));

        map.set_terrain(9, 9, Terrain::Sand);
        assert_eq!(map.get_terrain(9, 9), Some(Terrain::Sand));
    }

    #[test]
    fn test_get_set_temperature() {
        let mut map = TileMap::new(8, 6);
        assert_eq!(map.get_temperature(2, 3), Some(20.0));

        map.set_temperature(2, 3, 35.5);
        assert_eq!(map.get_temperature(2, 3), Some(35.5));

        map.set_temperature(0, 0, -10.0);
        assert_eq!(map.get_temperature(0, 0), Some(-10.0));
    }

    #[test]
    fn test_out_of_bounds_returns_none() {
        let map = TileMap::new(5, 5);

        assert_eq!(map.get_terrain(5, 0), None);
        assert_eq!(map.get_terrain(0, 5), None);
        assert_eq!(map.get_terrain(5, 5), None);
        assert_eq!(map.get_terrain(100, 100), None);

        assert_eq!(map.get_temperature(5, 0), None);
        assert_eq!(map.get_temperature(0, 5), None);
        assert_eq!(map.get_temperature(100, 100), None);
    }

    #[test]
    fn test_out_of_bounds_set_is_silent() {
        let mut map = TileMap::new(5, 5);
        // These should not panic.
        map.set_terrain(5, 0, Terrain::Water);
        map.set_terrain(0, 5, Terrain::Water);
        map.set_temperature(100, 100, 99.0);
    }

    #[test]
    fn test_index_matches_y_times_width_plus_x() {
        let map = TileMap::new(10, 8);

        assert_eq!(map.index(0, 0), Some(0));
        assert_eq!(map.index(1, 0), Some(1));
        assert_eq!(map.index(0, 1), Some(10));
        assert_eq!(map.index(3, 2), Some(2 * 10 + 3));
        assert_eq!(map.index(9, 7), Some(7 * 10 + 9));
    }

    #[test]
    fn test_adjacent_tiles_independent() {
        let mut map = TileMap::new(10, 10);
        map.set_terrain(3, 3, Terrain::Water);

        assert_eq!(map.get_terrain(2, 3), Some(Terrain::Grass));
        assert_eq!(map.get_terrain(4, 3), Some(Terrain::Grass));
        assert_eq!(map.get_terrain(3, 2), Some(Terrain::Grass));
        assert_eq!(map.get_terrain(3, 4), Some(Terrain::Grass));
        assert_eq!(map.get_terrain(3, 3), Some(Terrain::Water));
    }

    #[test]
    fn test_zero_size_map() {
        let map = TileMap::new(0, 0);
        assert_eq!(map.width(), 0);
        assert_eq!(map.height(), 0);
        assert_eq!(map.get_terrain(0, 0), None);
        assert_eq!(map.get_temperature(0, 0), None);
    }

    // --- Walkability ---

    #[test]
    fn test_terrain_walkability() {
        assert!(Terrain::Grass.is_walkable());
        assert!(Terrain::Dirt.is_walkable());
        assert!(Terrain::Sand.is_walkable());
        assert!(!Terrain::Water.is_walkable());
        assert!(!Terrain::Stone.is_walkable());
    }

    #[test]
    fn test_tilemap_is_walkable() {
        let mut map = TileMap::new(5, 5);
        assert!(map.is_walkable(0, 0)); // default Grass
        map.set_terrain(2, 2, Terrain::Water);
        assert!(!map.is_walkable(2, 2));
        assert!(!map.is_walkable(5, 0)); // out of bounds
    }

    // --- A* pathfinding ---

    #[test]
    fn test_find_path_same_tile() {
        let map = TileMap::new(10, 10);
        let path = map.find_path((3, 3), (3, 3));
        assert_eq!(path, Some(vec![]));
    }

    #[test]
    fn test_find_path_adjacent() {
        let map = TileMap::new(10, 10);
        let path = map.find_path((3, 3), (4, 3)).unwrap();
        assert_eq!(path, vec![(4, 3)]);
    }

    #[test]
    fn test_find_path_diagonal() {
        let map = TileMap::new(10, 10);
        let path = map.find_path((0, 0), (2, 2)).unwrap();
        // Optimal 8-dir path: 2 diagonal steps
        assert_eq!(path.len(), 2);
        assert_eq!(*path.last().unwrap(), (2, 2));
    }

    #[test]
    fn test_find_path_around_wall() {
        // Create a wall blocking direct path
        // . . . . .
        // . S # . .
        // . . # . .
        // . . # G .
        // . . . . .
        let mut map = TileMap::new(5, 5);
        map.set_terrain(2, 1, Terrain::Stone);
        map.set_terrain(2, 2, Terrain::Stone);
        map.set_terrain(2, 3, Terrain::Stone);

        let path = map.find_path((1, 1), (3, 3)).unwrap();
        // Path must go around the wall
        assert!(*path.last().unwrap() == (3, 3));
        // No step should be on a wall tile
        for &(px, py) in &path {
            assert!(
                map.is_walkable(px as usize, py as usize) || (px, py) == (3, 3),
                "path step ({px},{py}) is not walkable"
            );
        }
    }

    #[test]
    fn test_find_path_blocked() {
        // Completely surround the goal with water
        let mut map = TileMap::new(5, 5);
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                map.set_terrain((2 + dx) as usize, (2 + dy) as usize, Terrain::Water);
            }
        }
        // Goal at (2,2) is reachable (goal always treated as walkable)
        // but all neighbors are water, so start at (0,0) can't get adjacent
        let path = map.find_path((0, 0), (2, 2));
        // A* treats goal as walkable, but the 8 neighbors of goal are all water,
        // so no path can reach it from (0,0) unless going through water.
        // Since (2,2) itself is Grass but surrounded by water, the path needs
        // to step on water to reach (2,2). Since water is not walkable (and not goal),
        // this should return None.
        assert!(path.is_none());
    }

    #[test]
    fn test_find_path_out_of_bounds() {
        let map = TileMap::new(5, 5);
        assert!(map.find_path((-1, 0), (3, 3)).is_none());
        assert!(map.find_path((0, 0), (5, 5)).is_none());
    }

    #[test]
    fn test_find_path_optimal_length() {
        // Open map, path should be Chebyshev distance (step count)
        let map = TileMap::new(20, 20);
        let path = map.find_path((0, 0), (10, 7)).unwrap();
        // Chebyshev distance = max(10, 7) = 10 steps
        assert_eq!(path.len(), 10);
    }

    #[test]
    fn test_find_path_prefers_cardinal_when_shorter() {
        // Purely cardinal path: (0,0) to (3,0) should be 3 cardinal steps
        let map = TileMap::new(10, 10);
        let path = map.find_path((0, 0), (3, 0)).unwrap();
        assert_eq!(path.len(), 3);
        // Each step should only change x, not y (all cardinal)
        let mut prev = (0, 0);
        for &step in &path {
            assert_eq!(step.1, 0, "y should not change for a horizontal path");
            assert_eq!(
                (step.0 - prev.0).abs(),
                1,
                "should advance one tile at a time"
            );
            prev = step;
        }
    }

    #[test]
    fn test_diagonal_step_detection() {
        use super::is_diagonal_step;
        // Cardinal steps
        assert!(!is_diagonal_step((0, 0), (1, 0)));
        assert!(!is_diagonal_step((0, 0), (0, 1)));
        assert!(!is_diagonal_step((5, 3), (5, 4)));
        assert!(!is_diagonal_step((5, 3), (4, 3)));
        // Diagonal steps
        assert!(is_diagonal_step((0, 0), (1, 1)));
        assert!(is_diagonal_step((3, 3), (2, 4)));
        assert!(is_diagonal_step((5, 5), (4, 4)));
        assert!(is_diagonal_step((0, 0), (1, -1)));
    }
}
