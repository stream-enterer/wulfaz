use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Sequential building identifier, 1-based index into BuildingRegistry.buildings.
/// 0 is reserved as the "no building" sentinel in tile arrays.
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
    /// Original `Identif` from BATI.shp (cadastral parcel ID, not unique per record).
    pub identif: u32,
    pub quartier: String,
    /// Footprint area in m².
    pub superficie: f32,
    /// BATI classification: 1=built area, 2=non-built open space, 3=minor feature.
    pub bati: u8,
    pub nom_bati: Option<String>,
    pub num_ilot: String,
    /// Perimeter in meters (PERIMETRE field).
    #[serde(default)]
    pub perimetre: f32,
    /// Centroid X, Lambert projection (GEOX field).
    #[serde(default)]
    pub geox: f64,
    /// Centroid Y, Lambert projection (GEOY field).
    #[serde(default)]
    pub geoy: f64,
    /// Survey date (DATE_COYEC field).
    #[serde(default)]
    pub date_coyec: Option<String>,
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
    /// Vasserot's original block numbering (ILOTS_VASS field).
    #[serde(default)]
    pub ilots_vass: String,
    pub buildings: Vec<BuildingId>,
}

/// Vec-backed building registry. BuildingId is a 1-based index into the Vec.
#[derive(Default)]
pub struct BuildingRegistry {
    pub buildings: Vec<BuildingData>,
    /// Reverse lookup: cadastral Identif → all BuildingIds sharing that parcel.
    pub identif_index: HashMap<u32, Vec<BuildingId>>,
}

impl BuildingRegistry {
    pub fn new() -> Self {
        Self {
            buildings: Vec::new(),
            identif_index: HashMap::new(),
        }
    }

    /// Push a new building. Its `data.id` must already be set to the correct
    /// 1-based sequential BuildingId (next_id = buildings.len() + 1).
    pub fn insert(&mut self, data: BuildingData) {
        let id = data.id;
        let identif = data.identif;
        self.buildings.push(data);
        self.identif_index.entry(identif).or_default().push(id);
    }

    /// Allocate the next BuildingId (1-based).
    pub fn next_id(&self) -> BuildingId {
        BuildingId(self.buildings.len() as u32 + 1)
    }

    #[allow(dead_code)]
    pub fn get(&self, id: BuildingId) -> Option<&BuildingData> {
        if id.0 == 0 {
            return None;
        }
        self.buildings.get(id.0 as usize - 1)
    }

    #[allow(dead_code)]
    pub fn get_mut(&mut self, id: BuildingId) -> Option<&mut BuildingData> {
        if id.0 == 0 {
            return None;
        }
        self.buildings.get_mut(id.0 as usize - 1)
    }

    /// Find all buildings sharing a cadastral parcel Identif.
    #[allow(dead_code)]
    pub fn get_by_identif(&self, identif: u32) -> &[BuildingId] {
        self.identif_index
            .get(&identif)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn len(&self) -> usize {
        self.buildings.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.buildings.is_empty()
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
        let id = reg.next_id();
        assert_eq!(id.0, 1);
        reg.insert(BuildingData {
            id,
            identif: 42,
            quartier: "Arcis".into(),
            superficie: 120.0,
            bati: 1,
            nom_bati: None,
            num_ilot: "860IL74".into(),
            perimetre: 44.0,
            geox: 601234.5,
            geoy: 128456.7,
            date_coyec: None,
            floor_count: 3,
            tiles: vec![(10, 20), (11, 20)],
            addresses: Vec::new(),
            occupants: Vec::new(),
        });

        assert!(reg.get(BuildingId(1)).is_some());
        assert_eq!(reg.get(BuildingId(1)).unwrap().quartier, "Arcis");
        assert_eq!(reg.get(BuildingId(1)).unwrap().identif, 42);
        assert!(reg.get(BuildingId(0)).is_none());
        assert!(reg.get(BuildingId(999)).is_none());
    }

    #[test]
    fn test_building_registry_duplicate_identif() {
        let mut reg = BuildingRegistry::new();

        // Two BATI=1 buildings sharing the same cadastral parcel Identif
        // (e.g. main building + rear wing on same parcel)
        let id1 = reg.next_id();
        reg.insert(BuildingData {
            id: id1,
            identif: 100,
            quartier: "Arcis".into(),
            superficie: 80.0,
            bati: 1, // main building
            nom_bati: None,
            num_ilot: "T1".into(),
            perimetre: 36.0,
            geox: 0.0,
            geoy: 0.0,
            date_coyec: None,
            floor_count: 3,
            tiles: vec![(1, 1), (2, 1)],
            addresses: Vec::new(),
            occupants: Vec::new(),
        });

        let id2 = reg.next_id();
        reg.insert(BuildingData {
            id: id2,
            identif: 100,
            quartier: "Arcis".into(),
            superficie: 30.0,
            bati: 1, // rear wing
            nom_bati: None,
            num_ilot: "T1".into(),
            perimetre: 22.0,
            geox: 0.0,
            geoy: 0.0,
            date_coyec: None,
            floor_count: 2,
            tiles: vec![(5, 5), (6, 5)],
            addresses: Vec::new(),
            occupants: Vec::new(),
        });

        // Both buildings are preserved
        assert_eq!(reg.len(), 2);
        assert!(reg.get(id1).is_some());
        assert!(reg.get(id2).is_some());
        assert_eq!(reg.get(id1).unwrap().bati, 1);
        assert_eq!(reg.get(id2).unwrap().bati, 1);

        // Reverse lookup finds both
        let by_identif = reg.get_by_identif(100);
        assert_eq!(by_identif.len(), 2);
        assert!(by_identif.contains(&id1));
        assert!(by_identif.contains(&id2));
    }

    #[test]
    fn test_block_registry_insert_lookup() {
        let mut reg = BlockRegistry::new();
        reg.insert(BlockData {
            id: BlockId(1),
            id_ilots: "860IL74".into(),
            quartier: "Arcis".into(),
            aire: 5000.0,
            ilots_vass: "74".into(),
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
