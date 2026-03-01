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
#[allow(dead_code)] // Populated by GIS loading; read by upcoming street/address systems
#[derive(Clone, Serialize, Deserialize)]
pub struct Address {
    pub street_name: String,
    pub house_number: String,
}

/// Occupant data from SoDUCo directories, populated by A07.
#[allow(dead_code)] // Populated by GIS loading; read by B03 entity spawning + C04 district stats
#[derive(Clone, Serialize, Deserialize)]
pub struct Occupant {
    pub name: String,
    pub activity: String,
    pub naics: String,
}

#[allow(dead_code)] // Fields populated by GIS loading; read by B06 interiors + C04 district stats
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
    /// Occupants by year (SoDUCo snapshot year → occupant list), populated by A07.
    pub occupants_by_year: HashMap<u16, Vec<Occupant>>,
}

impl BuildingData {
    /// Return occupants from the nearest available year within `max_distance` of `target`.
    /// Prefers exact match, then spirals outward (±1, ±2, …). Ties broken toward later year.
    pub fn occupants_nearest(&self, target: u16, max_distance: u16) -> Option<(u16, &[Occupant])> {
        for delta in 0..=max_distance {
            // Try target + delta first (later year), then target - delta (earlier year).
            // For delta == 0 this just checks target once.
            if let Some(occ) = self.occupants_by_year.get(&(target.saturating_add(delta)))
                && !occ.is_empty()
            {
                return Some((target.saturating_add(delta), occ));
            }
            if delta > 0
                && let Some(occ) = self.occupants_by_year.get(&(target.saturating_sub(delta)))
                && !occ.is_empty()
            {
                return Some((target.saturating_sub(delta), occ));
            }
        }
        None
    }
}

#[allow(dead_code)] // Populated by GIS loading; read by C01 district definitions
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

    #[allow(dead_code)] // Used in tests; needed by B06 + C04
    pub fn get(&self, id: BuildingId) -> Option<&BuildingData> {
        if id.0 == 0 {
            return None;
        }
        self.buildings.get(id.0 as usize - 1)
    }

    #[allow(dead_code)] // Needed by B06 interior generation
    pub fn get_mut(&mut self, id: BuildingId) -> Option<&mut BuildingData> {
        if id.0 == 0 {
            return None;
        }
        self.buildings.get_mut(id.0 as usize - 1)
    }

    /// Find all buildings sharing a cadastral parcel Identif.
    #[allow(dead_code)] // Used in tests; needed for multi-building parcel queries
    pub fn get_by_identif(&self, identif: u32) -> &[BuildingId] {
        self.identif_index
            .get(&identif)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn len(&self) -> usize {
        self.buildings.len()
    }

    #[allow(dead_code)] // Standard container method
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

    #[allow(dead_code)] // Used in tests; needed by C01 district definitions
    pub fn get(&self, id: BlockId) -> Option<&BlockData> {
        self.blocks.get(&id)
    }
}

/// Quartier identifier, 1-based index matching tile_map encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuartierId(pub u8);

pub struct QuartierData {
    pub id: QuartierId,
    pub name: String,
    /// Tile-coordinate bounding box from all BATI=1 building tiles.
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub building_count: u32,
    /// Sum of superficie (m²) from BATI=1 buildings.
    pub total_building_area_m2: f32,
    /// Occupant count at active_year via `occupants_nearest(year, 20)`.
    pub occupant_count: u32,
    /// Sub-district block grouping, sorted by `BlockId.0`.
    pub blocks: Vec<BlockId>,
}

#[derive(Default)]
pub struct QuartierRegistry {
    pub quartiers: HashMap<QuartierId, QuartierData>,
    pub name_to_id: HashMap<String, QuartierId>,
}

impl QuartierRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build quartier aggregates from existing registries.
    /// `quartier_names` is the 0-indexed Vec where index+1 = QuartierId.
    pub fn build_from_registries(
        quartier_names: &[String],
        buildings: &BuildingRegistry,
        blocks: &BlockRegistry,
        active_year: u16,
    ) -> Self {
        let mut registry = Self::new();

        // Initialize one QuartierData per name with sentinel bounds
        for (i, name) in quartier_names.iter().enumerate() {
            let id = QuartierId((i + 1) as u8);
            registry.name_to_id.insert(name.clone(), id);
            registry.quartiers.insert(
                id,
                QuartierData {
                    id,
                    name: name.clone(),
                    min_x: i32::MAX,
                    min_y: i32::MAX,
                    max_x: i32::MIN,
                    max_y: i32::MIN,
                    building_count: 0,
                    total_building_area_m2: 0.0,
                    occupant_count: 0,
                    blocks: Vec::new(),
                },
            );
        }

        // Aggregate building data
        for bdata in &buildings.buildings {
            if bdata.bati != 1 {
                continue;
            }
            let Some(&qid) = registry.name_to_id.get(&bdata.quartier) else {
                continue;
            };
            let Some(qdata) = registry.quartiers.get_mut(&qid) else {
                continue;
            };

            qdata.building_count += 1;
            qdata.total_building_area_m2 += bdata.superficie;

            // Count occupants at active_year
            if let Some((_year, occupants)) = bdata.occupants_nearest(active_year, 20) {
                qdata.occupant_count += occupants.len() as u32;
            }

            // Expand bounds from building tiles
            for &(tx, ty) in &bdata.tiles {
                qdata.min_x = qdata.min_x.min(tx);
                qdata.min_y = qdata.min_y.min(ty);
                qdata.max_x = qdata.max_x.max(tx);
                qdata.max_y = qdata.max_y.max(ty);
            }
        }

        // Assign blocks to quartiers
        for block in blocks.blocks.values() {
            if let Some(&qid) = registry.name_to_id.get(&block.quartier)
                && let Some(qdata) = registry.quartiers.get_mut(&qid)
            {
                qdata.blocks.push(block.id);
            }
        }

        // Sort block vecs by BlockId.0 for determinism
        for qdata in registry.quartiers.values_mut() {
            qdata.blocks.sort_by_key(|b| b.0);
        }

        registry
    }

    #[allow(dead_code)] // Used in tests; needed by C02 LOD zone framework
    pub fn get(&self, id: QuartierId) -> Option<&QuartierData> {
        self.quartiers.get(&id)
    }

    #[allow(dead_code)] // Used in tests; needed by C02 LOD zone framework
    pub fn get_by_name(&self, name: &str) -> Option<&QuartierData> {
        self.name_to_id
            .get(name)
            .and_then(|id| self.quartiers.get(id))
    }
}

/// Sequential street identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StreetId(pub u16);

#[allow(dead_code)] // Populated by GIS loading; read by upcoming street-based systems
#[derive(Clone, Serialize, Deserialize)]
pub struct StreetData {
    pub name: String,
    pub buildings: Vec<BuildingId>,
}

/// Street registry, reconstructed from building address data at load time.
#[allow(dead_code)] // Populated by GIS loading; fields read by upcoming street-based systems
#[derive(Default)]
pub struct StreetRegistry {
    pub streets: HashMap<StreetId, StreetData>,
    pub name_to_id: HashMap<String, StreetId>,
}

impl StreetRegistry {
    pub fn new() -> Self {
        Self {
            streets: HashMap::new(),
            name_to_id: HashMap::new(),
        }
    }

    /// Reconstruct street registry from building address data.
    /// Scans all buildings for unique street names and maps buildings to streets.
    pub fn build_from_buildings(buildings: &BuildingRegistry) -> Self {
        let mut registry = Self::new();
        let mut next_id: u16 = 1;

        for bdata in &buildings.buildings {
            for addr in &bdata.addresses {
                if addr.street_name.is_empty() {
                    continue;
                }
                let street_id = if let Some(&sid) = registry.name_to_id.get(&addr.street_name) {
                    sid
                } else {
                    let sid = StreetId(next_id);
                    next_id += 1;
                    registry.name_to_id.insert(addr.street_name.clone(), sid);
                    registry.streets.insert(
                        sid,
                        StreetData {
                            name: addr.street_name.clone(),
                            buildings: Vec::new(),
                        },
                    );
                    sid
                };
                if let Some(sd) = registry.streets.get_mut(&street_id)
                    && !sd.buildings.contains(&bdata.id)
                {
                    sd.buildings.push(bdata.id);
                }
            }
        }

        registry
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
            occupants_by_year: HashMap::new(),
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
            occupants_by_year: HashMap::new(),
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
            occupants_by_year: HashMap::new(),
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
    fn test_street_registry_build_from_buildings() {
        let mut reg = BuildingRegistry::new();

        let id1 = reg.next_id();
        reg.insert(BuildingData {
            id: id1,
            identif: 1,
            quartier: "Arcis".into(),
            superficie: 100.0,
            bati: 1,
            nom_bati: None,
            num_ilot: "T1".into(),
            perimetre: 0.0,
            geox: 0.0,
            geoy: 0.0,
            date_coyec: None,
            floor_count: 3,
            tiles: vec![(1, 1)],
            addresses: vec![
                Address {
                    street_name: "Rue du Temple".into(),
                    house_number: "12".into(),
                },
                Address {
                    street_name: "Rue de Rivoli".into(),
                    house_number: "1".into(),
                },
            ],
            occupants_by_year: HashMap::new(),
        });

        let id2 = reg.next_id();
        reg.insert(BuildingData {
            id: id2,
            identif: 2,
            quartier: "Arcis".into(),
            superficie: 80.0,
            bati: 1,
            nom_bati: None,
            num_ilot: "T1".into(),
            perimetre: 0.0,
            geox: 0.0,
            geoy: 0.0,
            date_coyec: None,
            floor_count: 2,
            tiles: vec![(2, 2)],
            addresses: vec![Address {
                street_name: "Rue du Temple".into(),
                house_number: "14".into(),
            }],
            occupants_by_year: HashMap::new(),
        });

        let streets = StreetRegistry::build_from_buildings(&reg);

        // Two unique streets
        assert_eq!(streets.streets.len(), 2);
        assert_eq!(streets.name_to_id.len(), 2);

        // "Rue du Temple" has 2 buildings
        let temple_id = streets.name_to_id["Rue du Temple"];
        let temple = &streets.streets[&temple_id];
        assert_eq!(temple.buildings.len(), 2);
        assert!(temple.buildings.contains(&id1));
        assert!(temple.buildings.contains(&id2));

        // "Rue de Rivoli" has 1 building
        let rivoli_id = streets.name_to_id["Rue de Rivoli"];
        let rivoli = &streets.streets[&rivoli_id];
        assert_eq!(rivoli.buildings.len(), 1);
        assert!(rivoli.buildings.contains(&id1));
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

    fn make_building_with_years(years: &[u16]) -> BuildingData {
        let mut occ = HashMap::new();
        for &y in years {
            occ.insert(
                y,
                vec![Occupant {
                    name: format!("person_{y}"),
                    activity: "test".into(),
                    naics: "".into(),
                }],
            );
        }
        BuildingData {
            id: BuildingId(1),
            identif: 1,
            quartier: String::new(),
            superficie: 100.0,
            bati: 1,
            nom_bati: None,
            num_ilot: String::new(),
            perimetre: 0.0,
            geox: 0.0,
            geoy: 0.0,
            date_coyec: None,
            floor_count: 3,
            tiles: Vec::new(),
            addresses: Vec::new(),
            occupants_by_year: occ,
        }
    }

    #[test]
    fn test_occupants_nearest_exact_match() {
        let b = make_building_with_years(&[1839, 1845]);
        let (year, occ) = b.occupants_nearest(1839, 20).unwrap();
        assert_eq!(year, 1839);
        assert_eq!(occ[0].name, "person_1839");
    }

    #[test]
    fn test_occupants_nearest_fallback_later() {
        // Target 1839, no exact match — nearest is 1842 (delta +3)
        let b = make_building_with_years(&[1833, 1842]);
        let (year, _) = b.occupants_nearest(1839, 20).unwrap();
        // 1842 is +3, 1833 is -6 → 1842 wins
        assert_eq!(year, 1842);
    }

    #[test]
    fn test_occupants_nearest_fallback_earlier() {
        // Target 1839, nearest is 1833 (delta -6), 1850 is +11
        let b = make_building_with_years(&[1833, 1850]);
        let (year, _) = b.occupants_nearest(1839, 20).unwrap();
        assert_eq!(year, 1833);
    }

    #[test]
    fn test_occupants_nearest_beyond_max_distance() {
        let b = make_building_with_years(&[1900]);
        assert!(b.occupants_nearest(1839, 20).is_none());
    }

    #[test]
    fn test_occupants_nearest_empty_building() {
        let b = make_building_with_years(&[]);
        assert!(b.occupants_nearest(1839, 20).is_none());
    }

    fn make_test_building(
        id: BuildingId,
        quartier: &str,
        bati: u8,
        superficie: f32,
        tiles: Vec<(i32, i32)>,
        occupants_by_year: HashMap<u16, Vec<Occupant>>,
    ) -> BuildingData {
        BuildingData {
            id,
            identif: id.0,
            quartier: quartier.into(),
            superficie,
            bati,
            nom_bati: None,
            num_ilot: String::new(),
            perimetre: 0.0,
            geox: 0.0,
            geoy: 0.0,
            date_coyec: None,
            floor_count: estimate_floor_count(superficie),
            tiles,
            addresses: Vec::new(),
            occupants_by_year,
        }
    }

    #[test]
    fn test_quartier_registry_positive() {
        let quartier_names = vec!["Arcis".to_string(), "Marais".to_string()];

        let mut buildings = BuildingRegistry::new();
        // Arcis: 2 buildings, no occupants
        let id1 = buildings.next_id();
        buildings.insert(make_test_building(
            id1,
            "Arcis",
            1,
            100.0,
            vec![(10, 20), (11, 20), (12, 21)],
            HashMap::new(),
        ));
        let id2 = buildings.next_id();
        buildings.insert(make_test_building(
            id2,
            "Arcis",
            1,
            200.0,
            vec![(50, 60)],
            HashMap::new(),
        ));
        // Marais: 1 building with 2 occupants at year 1845
        let id3 = buildings.next_id();
        let mut occ_map = HashMap::new();
        occ_map.insert(
            1845,
            vec![
                Occupant {
                    name: "Jean".into(),
                    activity: "boulanger".into(),
                    naics: "311".into(),
                },
                Occupant {
                    name: "Marie".into(),
                    activity: "couturière".into(),
                    naics: "315".into(),
                },
            ],
        );
        buildings.insert(make_test_building(
            id3,
            "Marais",
            1,
            150.0,
            vec![(30, 40), (31, 40)],
            occ_map,
        ));

        let mut blocks = BlockRegistry::new();
        blocks.insert(BlockData {
            id: BlockId(1),
            id_ilots: "IL01".into(),
            quartier: "Arcis".into(),
            aire: 5000.0,
            ilots_vass: "1".into(),
            buildings: vec![id1],
        });
        blocks.insert(BlockData {
            id: BlockId(2),
            id_ilots: "IL02".into(),
            quartier: "Marais".into(),
            aire: 3000.0,
            ilots_vass: "2".into(),
            buildings: vec![id3],
        });

        let reg =
            QuartierRegistry::build_from_registries(&quartier_names, &buildings, &blocks, 1845);

        // Arcis
        let arcis = reg.get_by_name("Arcis").unwrap();
        assert_eq!(arcis.building_count, 2);
        assert!((arcis.total_building_area_m2 - 300.0).abs() < 0.01);
        assert_eq!(arcis.occupant_count, 0);
        assert_eq!(arcis.min_x, 10);
        assert_eq!(arcis.min_y, 20);
        assert_eq!(arcis.max_x, 50);
        assert_eq!(arcis.max_y, 60);
        assert_eq!(arcis.blocks, vec![BlockId(1)]);

        // Marais
        let marais = reg.get_by_name("Marais").unwrap();
        assert_eq!(marais.building_count, 1);
        assert!((marais.total_building_area_m2 - 150.0).abs() < 0.01);
        assert_eq!(marais.occupant_count, 2);
        assert_eq!(marais.min_x, 30);
        assert_eq!(marais.max_x, 31);
        assert_eq!(marais.blocks, vec![BlockId(2)]);
    }

    #[test]
    fn test_quartier_registry_negative() {
        let quartier_names = vec!["Arcis".to_string()];

        let mut buildings = BuildingRegistry::new();
        // BATI=2 building — should be skipped
        let id1 = buildings.next_id();
        buildings.insert(make_test_building(
            id1,
            "Arcis",
            2,
            500.0,
            vec![(1, 1)],
            HashMap::new(),
        ));
        // Unknown quartier — should be skipped
        let id2 = buildings.next_id();
        buildings.insert(make_test_building(
            id2,
            "UnknownQuartier",
            1,
            80.0,
            vec![(5, 5)],
            HashMap::new(),
        ));

        let blocks = BlockRegistry::new();
        let reg =
            QuartierRegistry::build_from_registries(&quartier_names, &buildings, &blocks, 1845);

        let arcis = reg.get_by_name("Arcis").unwrap();
        assert_eq!(arcis.building_count, 0);
        assert_eq!(arcis.total_building_area_m2, 0.0);
        assert_eq!(arcis.occupant_count, 0);
        // Sentinel bounds preserved (no valid buildings expanded them)
        assert_eq!(arcis.min_x, i32::MAX);
        assert_eq!(arcis.max_x, i32::MIN);
    }

    #[test]
    fn test_quartier_registry_lookup() {
        let quartier_names = vec!["Arcis".to_string()];
        let buildings = BuildingRegistry::new();
        let blocks = BlockRegistry::new();
        let reg =
            QuartierRegistry::build_from_registries(&quartier_names, &buildings, &blocks, 1845);

        // Valid lookup
        assert!(reg.get(QuartierId(1)).is_some());
        assert_eq!(reg.get(QuartierId(1)).unwrap().name, "Arcis");

        // Invalid id
        assert!(reg.get(QuartierId(0)).is_none());
        assert!(reg.get(QuartierId(255)).is_none());

        // Name lookup
        assert!(reg.get_by_name("Arcis").is_some());
        assert!(reg.get_by_name("Nonexistent").is_none());
    }
}
