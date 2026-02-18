use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt;

use crate::components::Tick;

/// Side length of each chunk in tiles. 1 chunk = 64×64 = 4096 tiles.
pub const CHUNK_SIZE: usize = 64;
const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
    Road,      // streets, alleys, open ground — walkable
    Wall,      // building perimeter — blocked
    Floor,     // building interior — walkable
    Door,      // building entrance — walkable
    Courtyard, // enclosed open space within a city block — walkable
    Garden,    // parks, green space — walkable
    Water,     // river — blocked
    Bridge,    // river crossing — walkable
}

impl Terrain {
    pub fn is_walkable(self) -> bool {
        matches!(
            self,
            Terrain::Road
                | Terrain::Floor
                | Terrain::Door
                | Terrain::Courtyard
                | Terrain::Garden
                | Terrain::Bridge
        )
    }
}

/// Chunk coordinate — identifies a 64×64 chunk in the world grid.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    pub cx: i32,
    pub cy: i32,
}

impl fmt::Debug for ChunkCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChunkCoord({}, {})", self.cx, self.cy)
    }
}

/// A 64×64 tile chunk with terrain and temperature layers.
pub struct Chunk {
    terrain: [Terrain; CHUNK_AREA],
    temperature: [f32; CHUNK_AREA],
    /// Set to true when any tile in this chunk is modified.
    pub dirty: bool,
    /// Last simulation tick that touched this chunk.
    pub last_tick: Tick,
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Chunk")
            .field("dirty", &self.dirty)
            .field("last_tick", &self.last_tick)
            .finish_non_exhaustive()
    }
}

impl Chunk {
    /// Create a new chunk with default terrain (Road) and temperature (16°C).
    fn new() -> Self {
        Self {
            terrain: [Terrain::Road; CHUNK_AREA],
            temperature: [16.0; CHUNK_AREA],
            dirty: false,
            last_tick: Tick(0),
        }
    }

    fn local_index(lx: usize, ly: usize) -> usize {
        debug_assert!(lx < CHUNK_SIZE && ly < CHUNK_SIZE);
        ly * CHUNK_SIZE + lx
    }

    pub fn get_terrain(&self, lx: usize, ly: usize) -> Terrain {
        self.terrain[Self::local_index(lx, ly)]
    }

    pub fn set_terrain(&mut self, lx: usize, ly: usize, t: Terrain) {
        self.terrain[Self::local_index(lx, ly)] = t;
        self.dirty = true;
    }

    pub fn get_temperature(&self, lx: usize, ly: usize) -> f32 {
        self.temperature[Self::local_index(lx, ly)]
    }

    pub fn set_temperature(&mut self, lx: usize, ly: usize, temp: f32) {
        self.temperature[Self::local_index(lx, ly)] = temp;
        self.dirty = true;
    }
}

pub struct TileMap {
    chunks: HashMap<ChunkCoord, Chunk>,
    width: usize,  // total tiles
    height: usize, // total tiles
}

impl fmt::Debug for TileMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TileMap")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("chunks", &self.chunks.len())
            .finish()
    }
}

impl TileMap {
    pub fn new(width: usize, height: usize) -> Self {
        let chunks_x = width.div_ceil(CHUNK_SIZE);
        let chunks_y = height.div_ceil(CHUNK_SIZE);
        let mut chunks = HashMap::with_capacity(chunks_x * chunks_y);
        for cy in 0..chunks_y {
            for cx in 0..chunks_x {
                chunks.insert(
                    ChunkCoord {
                        cx: cx as i32,
                        cy: cy as i32,
                    },
                    Chunk::new(),
                );
            }
        }
        Self {
            chunks,
            width,
            height,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Convert tile coordinates to chunk coord + local offset.
    /// Returns None if out of bounds.
    fn chunk_and_local(&self, x: usize, y: usize) -> Option<(ChunkCoord, usize, usize)> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let coord = ChunkCoord {
            cx: (x / CHUNK_SIZE) as i32,
            cy: (y / CHUNK_SIZE) as i32,
        };
        let lx = x % CHUNK_SIZE;
        let ly = y % CHUNK_SIZE;
        Some((coord, lx, ly))
    }

    /// Convert tile coordinates to a ChunkCoord (public, for A04/A05).
    #[allow(dead_code)]
    pub fn tile_to_chunk(x: usize, y: usize) -> ChunkCoord {
        ChunkCoord {
            cx: (x / CHUNK_SIZE) as i32,
            cy: (y / CHUNK_SIZE) as i32,
        }
    }

    pub fn get_terrain(&self, x: usize, y: usize) -> Option<Terrain> {
        let (coord, lx, ly) = self.chunk_and_local(x, y)?;
        self.chunks.get(&coord).map(|c| c.get_terrain(lx, ly))
    }

    pub fn set_terrain(&mut self, x: usize, y: usize, t: Terrain) {
        if x >= self.width || y >= self.height {
            return;
        }
        let coord = ChunkCoord {
            cx: (x / CHUNK_SIZE) as i32,
            cy: (y / CHUNK_SIZE) as i32,
        };
        let lx = x % CHUNK_SIZE;
        let ly = y % CHUNK_SIZE;
        self.chunks
            .entry(coord)
            .or_insert_with(Chunk::new)
            .set_terrain(lx, ly, t);
    }

    pub fn get_temperature(&self, x: usize, y: usize) -> Option<f32> {
        let (coord, lx, ly) = self.chunk_and_local(x, y)?;
        self.chunks.get(&coord).map(|c| c.get_temperature(lx, ly))
    }

    pub fn set_temperature(&mut self, x: usize, y: usize, temp: f32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let coord = ChunkCoord {
            cx: (x / CHUNK_SIZE) as i32,
            cy: (y / CHUNK_SIZE) as i32,
        };
        let lx = x % CHUNK_SIZE;
        let ly = y % CHUNK_SIZE;
        self.chunks
            .entry(coord)
            .or_insert_with(Chunk::new)
            .set_temperature(lx, ly, temp);
    }

    /// Check if a tile is walkable (in-bounds and terrain allows passage).
    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        self.get_terrain(x, y).is_some_and(|t| t.is_walkable())
    }

    /// Get an immutable reference to a chunk by coordinate (for A04/A05).
    #[allow(dead_code)]
    pub fn get_chunk(&self, coord: ChunkCoord) -> Option<&Chunk> {
        self.chunks.get(&coord)
    }

    /// Get a mutable reference to a chunk by coordinate (for A04/A05).
    #[allow(dead_code)]
    pub fn get_chunk_mut(&mut self, coord: ChunkCoord) -> Option<&mut Chunk> {
        self.chunks.get_mut(&coord)
    }

    /// Iterate over all chunks (for A04/A05).
    #[allow(dead_code)]
    pub fn chunks(&self) -> impl Iterator<Item = (&ChunkCoord, &Chunk)> {
        self.chunks.iter()
    }

    /// Iterate over all chunks mutably (for A04/A05).
    #[allow(dead_code)]
    pub fn chunks_mut(&mut self) -> impl Iterator<Item = (&ChunkCoord, &mut Chunk)> {
        self.chunks.iter_mut()
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
        assert_eq!(map.get_terrain(0, 0), Some(Terrain::Road));
        assert_eq!(map.get_temperature(0, 0), Some(16.0));
    }

    #[test]
    fn test_get_set_terrain() {
        let mut map = TileMap::new(10, 10);
        assert_eq!(map.get_terrain(3, 5), Some(Terrain::Road));

        map.set_terrain(3, 5, Terrain::Water);
        assert_eq!(map.get_terrain(3, 5), Some(Terrain::Water));

        map.set_terrain(3, 5, Terrain::Wall);
        assert_eq!(map.get_terrain(3, 5), Some(Terrain::Wall));

        map.set_terrain(0, 0, Terrain::Floor);
        assert_eq!(map.get_terrain(0, 0), Some(Terrain::Floor));

        map.set_terrain(9, 9, Terrain::Bridge);
        assert_eq!(map.get_terrain(9, 9), Some(Terrain::Bridge));
    }

    #[test]
    fn test_get_set_temperature() {
        let mut map = TileMap::new(8, 6);
        assert_eq!(map.get_temperature(2, 3), Some(16.0));

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
    fn test_chunk_and_local_mapping() {
        let map = TileMap::new(200, 150);

        // Tile (0,0) → chunk (0,0), local (0,0)
        let (coord, lx, ly) = map.chunk_and_local(0, 0).unwrap();
        assert_eq!(coord, ChunkCoord { cx: 0, cy: 0 });
        assert_eq!((lx, ly), (0, 0));

        // Tile (63,63) → chunk (0,0), local (63,63)
        let (coord, lx, ly) = map.chunk_and_local(63, 63).unwrap();
        assert_eq!(coord, ChunkCoord { cx: 0, cy: 0 });
        assert_eq!((lx, ly), (63, 63));

        // Tile (64,0) → chunk (1,0), local (0,0)
        let (coord, lx, ly) = map.chunk_and_local(64, 0).unwrap();
        assert_eq!(coord, ChunkCoord { cx: 1, cy: 0 });
        assert_eq!((lx, ly), (0, 0));

        // Tile (65,64) → chunk (1,1), local (1,0)
        let (coord, lx, ly) = map.chunk_and_local(65, 64).unwrap();
        assert_eq!(coord, ChunkCoord { cx: 1, cy: 1 });
        assert_eq!((lx, ly), (1, 0));

        // Tile (199,149) → chunk (3,2), local (7,21)
        let (coord, lx, ly) = map.chunk_and_local(199, 149).unwrap();
        assert_eq!(coord, ChunkCoord { cx: 3, cy: 2 });
        assert_eq!((lx, ly), (199 % 64, 149 % 64));

        // Out of bounds
        assert!(map.chunk_and_local(200, 0).is_none());
        assert!(map.chunk_and_local(0, 150).is_none());
    }

    #[test]
    fn test_tile_to_chunk() {
        assert_eq!(TileMap::tile_to_chunk(0, 0), ChunkCoord { cx: 0, cy: 0 });
        assert_eq!(TileMap::tile_to_chunk(63, 63), ChunkCoord { cx: 0, cy: 0 });
        assert_eq!(TileMap::tile_to_chunk(64, 0), ChunkCoord { cx: 1, cy: 0 });
        assert_eq!(
            TileMap::tile_to_chunk(128, 128),
            ChunkCoord { cx: 2, cy: 2 }
        );
    }

    #[test]
    fn test_multi_chunk_get_set() {
        // 200×150 map spans 4×3 = 12 chunks
        let mut map = TileMap::new(200, 150);

        // Set terrain in different chunks
        map.set_terrain(0, 0, Terrain::Water); // chunk (0,0)
        map.set_terrain(100, 0, Terrain::Wall); // chunk (1,0)
        map.set_terrain(0, 100, Terrain::Garden); // chunk (0,1)
        map.set_terrain(199, 149, Terrain::Bridge); // chunk (3,2)

        assert_eq!(map.get_terrain(0, 0), Some(Terrain::Water));
        assert_eq!(map.get_terrain(100, 0), Some(Terrain::Wall));
        assert_eq!(map.get_terrain(0, 100), Some(Terrain::Garden));
        assert_eq!(map.get_terrain(199, 149), Some(Terrain::Bridge));

        // Unmodified tiles are still Road
        assert_eq!(map.get_terrain(50, 50), Some(Terrain::Road));
        assert_eq!(map.get_terrain(130, 80), Some(Terrain::Road));
    }

    #[test]
    fn test_chunk_dirty_tracking() {
        let mut map = TileMap::new(200, 150);

        // All chunks start clean
        for (_, chunk) in map.chunks() {
            assert!(!chunk.dirty);
        }

        // Modify a tile in chunk (1,1)
        map.set_terrain(70, 70, Terrain::Water);

        let coord = ChunkCoord { cx: 1, cy: 1 };
        assert!(map.get_chunk(coord).unwrap().dirty);

        // Chunk (0,0) should still be clean
        let coord0 = ChunkCoord { cx: 0, cy: 0 };
        assert!(!map.get_chunk(coord0).unwrap().dirty);
    }

    #[test]
    fn test_chunk_count() {
        // 64×64 → exactly 1 chunk
        let map = TileMap::new(64, 64);
        assert_eq!(map.chunks().count(), 1);

        // 65×65 → 2×2 = 4 chunks
        let map = TileMap::new(65, 65);
        assert_eq!(map.chunks().count(), 4);

        // 200×150 → 4×3 = 12 chunks (ceil(200/64) × ceil(150/64))
        let map = TileMap::new(200, 150);
        assert_eq!(map.chunks().count(), 12);

        // 0×0 → 0 chunks
        let map = TileMap::new(0, 0);
        assert_eq!(map.chunks().count(), 0);
    }

    #[test]
    fn test_adjacent_tiles_independent() {
        let mut map = TileMap::new(10, 10);
        map.set_terrain(3, 3, Terrain::Water);

        assert_eq!(map.get_terrain(2, 3), Some(Terrain::Road));
        assert_eq!(map.get_terrain(4, 3), Some(Terrain::Road));
        assert_eq!(map.get_terrain(3, 2), Some(Terrain::Road));
        assert_eq!(map.get_terrain(3, 4), Some(Terrain::Road));
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
        assert!(Terrain::Road.is_walkable());
        assert!(Terrain::Floor.is_walkable());
        assert!(Terrain::Door.is_walkable());
        assert!(Terrain::Courtyard.is_walkable());
        assert!(Terrain::Garden.is_walkable());
        assert!(Terrain::Bridge.is_walkable());
        assert!(!Terrain::Wall.is_walkable());
        assert!(!Terrain::Water.is_walkable());
    }

    #[test]
    fn test_tilemap_is_walkable() {
        let mut map = TileMap::new(5, 5);
        assert!(map.is_walkable(0, 0)); // default Road
        map.set_terrain(2, 2, Terrain::Water);
        assert!(!map.is_walkable(2, 2));
        map.set_terrain(3, 3, Terrain::Wall);
        assert!(!map.is_walkable(3, 3));
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
        map.set_terrain(2, 1, Terrain::Wall);
        map.set_terrain(2, 2, Terrain::Wall);
        map.set_terrain(2, 3, Terrain::Wall);

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
        // Since (2,2) itself is Road but surrounded by water, the path needs
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
