#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
    Grass,
    Water,
    Stone,
    Dirt,
    Sand,
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
}
