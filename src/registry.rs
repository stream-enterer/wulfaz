use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Unique building identifier — matches `Identif` field from BATI.shp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BuildingId(pub u32);

/// Sequential block identifier, assigned during GIS loading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockId(pub u16);

/// Placeholder for address data, populated by A07.
#[allow(dead_code)]
#[derive(Clone, Serialize, Deserialize)]
pub struct Address {
    pub street_name: String,
    pub house_number: String,
}

/// Placeholder for occupant data, populated by A07.
#[allow(dead_code)]
#[derive(Clone, Serialize, Deserialize)]
pub struct Occupant {
    pub name: String,
    pub activity: String,
}

#[allow(dead_code)]
#[derive(Clone, Serialize, Deserialize)]
pub struct BuildingData {
    pub id: BuildingId,
    pub quartier: String,
    /// Footprint area in m².
    pub superficie: f32,
    /// Building type: 1=main, 2=annex, 3=market stall.
    pub bati: u8,
    pub nom_bati: Option<String>,
    pub num_ilot: String,
    /// Estimated floor count, derived from superficie.
    pub floor_count: u8,
    /// All tile coordinates belonging to this building.
    pub tiles: Vec<(i32, i32)>,
    /// Populated later by A07.
    pub addresses: Vec<Address>,
    /// Populated later by A07.
    pub occupants: Vec<Occupant>,
}

#[allow(dead_code)]
#[derive(Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub id: BlockId,
    /// Original ID from shapefile, e.g. "860IL74".
    pub id_ilots: String,
    pub quartier: String,
    /// Block area in m².
    pub aire: f32,
    pub buildings: Vec<BuildingId>,
}

#[derive(Default)]
pub struct BuildingRegistry {
    pub buildings: HashMap<BuildingId, BuildingData>,
}

impl BuildingRegistry {
    pub fn new() -> Self {
        Self {
            buildings: HashMap::new(),
        }
    }

    pub fn insert(&mut self, data: BuildingData) {
        self.buildings.insert(data.id, data);
    }

    #[allow(dead_code)]
    pub fn get(&self, id: BuildingId) -> Option<&BuildingData> {
        self.buildings.get(&id)
    }
}

#[derive(Default)]
pub struct BlockRegistry {
    pub blocks: HashMap<BlockId, BlockData>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }

    pub fn insert(&mut self, data: BlockData) {
        self.blocks.insert(data.id, data);
    }

    #[allow(dead_code)]
    pub fn get(&self, id: BlockId) -> Option<&BlockData> {
        self.blocks.get(&id)
    }
}

/// Estimate floor count from building footprint area (m²).
/// <50m² → 2, 50-150m² → 3, 150-400m² → 4, >400m² → 5.
pub fn estimate_floor_count(superficie: f32) -> u8 {
    if superficie < 50.0 {
        2
    } else if superficie < 150.0 {
        3
    } else if superficie < 400.0 {
        4
    } else {
        5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_registry_insert_lookup() {
        let mut reg = BuildingRegistry::new();
        reg.insert(BuildingData {
            id: BuildingId(42),
            quartier: "Arcis".into(),
            superficie: 120.0,
            bati: 1,
            nom_bati: None,
            num_ilot: "860IL74".into(),
            floor_count: 3,
            tiles: vec![(10, 20), (11, 20)],
            addresses: Vec::new(),
            occupants: Vec::new(),
        });

        assert!(reg.get(BuildingId(42)).is_some());
        assert_eq!(reg.get(BuildingId(42)).unwrap().quartier, "Arcis");
        assert!(reg.get(BuildingId(999)).is_none());
    }

    #[test]
    fn test_block_registry_insert_lookup() {
        let mut reg = BlockRegistry::new();
        reg.insert(BlockData {
            id: BlockId(1),
            id_ilots: "860IL74".into(),
            quartier: "Arcis".into(),
            aire: 5000.0,
            buildings: vec![BuildingId(10), BuildingId(20)],
        });

        assert!(reg.get(BlockId(1)).is_some());
        assert_eq!(reg.get(BlockId(1)).unwrap().buildings.len(), 2);
        assert!(reg.get(BlockId(999)).is_none());
    }

    #[test]
    fn test_floor_count_estimation() {
        assert_eq!(estimate_floor_count(30.0), 2);
        assert_eq!(estimate_floor_count(49.9), 2);
        assert_eq!(estimate_floor_count(50.0), 3);
        assert_eq!(estimate_floor_count(100.0), 3);
        assert_eq!(estimate_floor_count(149.9), 3);
        assert_eq!(estimate_floor_count(150.0), 4);
        assert_eq!(estimate_floor_count(300.0), 4);
        assert_eq!(estimate_floor_count(399.9), 4);
        assert_eq!(estimate_floor_count(400.0), 5);
        assert_eq!(estimate_floor_count(1000.0), 5);
    }
}
