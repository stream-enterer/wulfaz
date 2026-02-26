use std::collections::{HashMap, HashSet};

use rand::rngs::StdRng;

use crate::components::*;

/// Spatial index cell size in tiles (meters). Power of 2 for bit-shift.
const SPATIAL_CELL_SHIFT: i32 = 4; // 2^4 = 16m cells

/// Coarse spatial grid: cell coords → list of (entity, x, y) in that cell.
pub type SpatialGrid = HashMap<(i32, i32), Vec<(Entity, i32, i32)>>;

use crate::events::EventLog;
use crate::registry::{BlockRegistry, BuildingRegistry, StreetRegistry};
use crate::rng::create_rng;
use crate::systems::decisions::UtilityConfig;
use crate::tile_map::TileMap;

pub struct BodyTables {
    pub positions: HashMap<Entity, Position>,
    pub healths: HashMap<Entity, Health>,
    pub fatigues: HashMap<Entity, Fatigue>,
    pub combat_stats: HashMap<Entity, CombatStats>,
    pub gait_profiles: HashMap<Entity, GaitProfile>,
    pub current_gaits: HashMap<Entity, Gait>,
    pub move_cooldowns: HashMap<Entity, MoveCooldown>,
    pub icons: HashMap<Entity, Icon>,
    pub names: HashMap<Entity, Name>,
}

impl BodyTables {
    fn new() -> Self {
        Self {
            positions: HashMap::new(),
            healths: HashMap::new(),
            fatigues: HashMap::new(),
            combat_stats: HashMap::new(),
            gait_profiles: HashMap::new(),
            current_gaits: HashMap::new(),
            move_cooldowns: HashMap::new(),
            icons: HashMap::new(),
            names: HashMap::new(),
        }
    }

    fn remove(&mut self, entity: &Entity) {
        self.positions.remove(entity);
        self.healths.remove(entity);
        self.fatigues.remove(entity);
        self.combat_stats.remove(entity);
        self.gait_profiles.remove(entity);
        self.current_gaits.remove(entity);
        self.move_cooldowns.remove(entity);
        self.icons.remove(entity);
        self.names.remove(entity);
    }
}

pub struct MindTables {
    pub hungers: HashMap<Entity, Hunger>,
    pub nutritions: HashMap<Entity, Nutrition>,
    pub intentions: HashMap<Entity, Intention>,
    pub action_states: HashMap<Entity, ActionState>,
    pub wander_targets: HashMap<Entity, WanderTarget>,
    pub occupations: HashMap<Entity, Occupation>,
    pub utility_config: UtilityConfig,
}

impl MindTables {
    fn new() -> Self {
        Self {
            hungers: HashMap::new(),
            nutritions: HashMap::new(),
            intentions: HashMap::new(),
            action_states: HashMap::new(),
            wander_targets: HashMap::new(),
            occupations: HashMap::new(),
            utility_config: UtilityConfig::default(),
        }
    }

    fn remove(&mut self, entity: &Entity) {
        self.hungers.remove(entity);
        self.nutritions.remove(entity);
        self.intentions.remove(entity);
        self.action_states.remove(entity);
        self.wander_targets.remove(entity);
        self.occupations.remove(entity);
    }
}

pub struct GisTables {
    pub buildings: BuildingRegistry,
    pub blocks: BlockRegistry,
    /// Maps quartier_id (1-based index) to quartier name string.
    pub quartier_names: Vec<String>,
    /// Street registry, reconstructed from building address data.
    pub streets: StreetRegistry,
    /// Active SoDUCo snapshot year for occupant display.
    pub active_year: u16,
    // Per-entity GIS links
    pub home_buildings: HashMap<Entity, HomeBuilding>,
    pub workplaces: HashMap<Entity, Workplace>,
}

impl GisTables {
    fn new() -> Self {
        Self {
            buildings: BuildingRegistry::new(),
            blocks: BlockRegistry::new(),
            quartier_names: Vec::new(),
            streets: StreetRegistry::new(),
            active_year: 1845,
            home_buildings: HashMap::new(),
            workplaces: HashMap::new(),
        }
    }

    fn remove(&mut self, entity: &Entity) {
        self.home_buildings.remove(entity);
        self.workplaces.remove(entity);
    }
}

pub struct World {
    // Entity tracking
    pub alive: HashSet<Entity>,
    pub pending_deaths: HashSet<Entity>,
    next_entity_id: u64,

    // Sub-struct property tables
    pub body: BodyTables,
    pub mind: MindTables,
    pub gis: GisTables,

    // Spatial acceleration
    /// Coarse spatial index, rebuilt from positions each tick.
    pub spatial_index: SpatialGrid,

    // Infrastructure
    pub tiles: TileMap,
    pub events: EventLog,
    pub rng: StdRng,
    pub tick: Tick,
    /// Player-controlled entity. None = realtime mode, Some = roguelike mode.
    pub player: Option<Entity>,
}

impl World {
    /// Create a new World with all fields initialized and a deterministic RNG seed.
    pub fn new_with_seed(seed: u64) -> Self {
        Self {
            alive: HashSet::new(),
            pending_deaths: HashSet::new(),
            next_entity_id: 1, // 0 is reserved/unused

            body: BodyTables::new(),
            mind: MindTables::new(),
            gis: GisTables::new(),

            spatial_index: HashMap::new(),

            tiles: TileMap::new(64, 64), // 64m × 64m
            events: EventLog::default_capacity(),
            rng: create_rng(seed),
            tick: Tick(0),
            player: None,
        }
    }

    /// Spawn a new entity. Returns the Entity with a unique ID.
    /// The entity is added to the alive set but has no components yet.
    pub fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next_entity_id);
        self.next_entity_id += 1;
        self.alive.insert(entity);
        entity
    }

    /// Rebuild the spatial index from current positions.
    /// Call at the start of each tick, after run_death has cleared pending_deaths.
    pub fn rebuild_spatial_index(&mut self) {
        self.spatial_index.clear();
        for (&entity, pos) in &self.body.positions {
            if self.alive.contains(&entity) {
                let key = (pos.x >> SPATIAL_CELL_SHIFT, pos.y >> SPATIAL_CELL_SHIFT);
                self.spatial_index
                    .entry(key)
                    .or_default()
                    .push((entity, pos.x, pos.y));
            }
        }
    }

    /// Return all entities at a given tile coordinate.
    pub fn entities_at(&self, x: i32, y: i32) -> impl Iterator<Item = Entity> + '_ {
        let key = (x >> SPATIAL_CELL_SHIFT, y >> SPATIAL_CELL_SHIFT);
        self.spatial_index
            .get(&key)
            .into_iter()
            .flat_map(|v| v.iter())
            .filter(move |(_, ex, ey)| *ex == x && *ey == y)
            .map(|(e, _, _)| *e)
    }

    /// Return all entities within Chebyshev distance `range` of (cx, cy).
    /// Scans coarse grid cells, filters by exact Chebyshev distance.
    pub fn entities_in_range(
        &self,
        cx: i32,
        cy: i32,
        range: i32,
    ) -> impl Iterator<Item = Entity> + '_ {
        let min_x = (cx - range) >> SPATIAL_CELL_SHIFT;
        let max_x = (cx + range) >> SPATIAL_CELL_SHIFT;
        let min_y = (cy - range) >> SPATIAL_CELL_SHIFT;
        let max_y = (cy + range) >> SPATIAL_CELL_SHIFT;
        (min_y..=max_y).flat_map(move |cell_y| {
            (min_x..=max_x).flat_map(move |cell_x| {
                self.spatial_index
                    .get(&(cell_x, cell_y))
                    .into_iter()
                    .flat_map(|v| v.iter())
                    .filter(move |(_, ex, ey)| (*ex - cx).abs().max((*ey - cy).abs()) <= range)
                    .map(|(e, _, _)| *e)
            })
        })
    }

    /// Remove an entity from ALL tables. Called ONLY by run_death.
    /// CRITICAL: Every HashMap property table MUST have a .remove() call here.
    /// If you add a new property table to World, add a corresponding remove here.
    pub fn despawn(&mut self, entity: Entity) {
        self.alive.remove(&entity);
        if self.player == Some(entity) {
            self.player = None;
        }
        self.body.remove(&entity);
        self.mind.remove(&entity);
        self.gis.remove(&entity);
    }
}

/// Validate world invariants. Run every tick in debug builds.
/// Checks that no entity exists in any property table without being in alive.
#[cfg(debug_assertions)]
pub fn validate_world(world: &World) {
    // Body tables
    for entity in world.body.positions.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in positions but not in alive",
            entity
        );
    }

    for entity in world.body.healths.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in healths but not in alive",
            entity
        );
    }

    for entity in world.body.fatigues.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in fatigues but not in alive",
            entity
        );
    }

    for entity in world.body.combat_stats.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in combat_stats but not in alive",
            entity
        );
    }

    for entity in world.body.gait_profiles.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in gait_profiles but not in alive",
            entity
        );
    }

    for entity in world.body.current_gaits.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in current_gaits but not in alive",
            entity
        );
    }

    for entity in world.body.move_cooldowns.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in move_cooldowns but not in alive",
            entity
        );
    }

    for entity in world.body.icons.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in icons but not in alive",
            entity
        );
    }

    for entity in world.body.names.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in names but not in alive",
            entity
        );
    }

    // Mind tables
    for entity in world.mind.hungers.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in hungers but not in alive",
            entity
        );
    }

    for entity in world.mind.nutritions.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in nutritions but not in alive",
            entity
        );
    }

    for entity in world.mind.intentions.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in intentions but not in alive",
            entity
        );
    }

    for entity in world.mind.action_states.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in action_states but not in alive",
            entity
        );
    }

    for entity in world.mind.wander_targets.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in wander_targets but not in alive",
            entity
        );
    }

    for entity in world.mind.occupations.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in occupations but not in alive",
            entity
        );
    }

    // GIS per-entity tables
    for entity in world.gis.home_buildings.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in home_buildings but not in alive",
            entity
        );
    }

    for entity in world.gis.workplaces.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in workplaces but not in alive",
            entity
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_creates_unique_entities() {
        let mut world = World::new_with_seed(42);
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();
        assert_ne!(e1, e2);
        assert_ne!(e2, e3);
        assert!(world.alive.contains(&e1));
        assert!(world.alive.contains(&e2));
        assert!(world.alive.contains(&e3));
    }

    #[test]
    fn despawn_removes_from_all_tables() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 5, y: 10 });
        world.mind.hungers.insert(
            e,
            Hunger {
                current: 50.0,
                max: 100.0,
            },
        );
        world.body.healths.insert(
            e,
            Health {
                current: 80.0,
                max: 100.0,
            },
        );
        world.body.fatigues.insert(e, Fatigue { current: 0.0 });
        world.body.combat_stats.insert(
            e,
            CombatStats {
                attack: 10.0,
                defense: 5.0,
                aggression: 0.8,
            },
        );
        world.body.gait_profiles.insert(e, GaitProfile::biped());
        world.body.current_gaits.insert(e, Gait::Walk);
        world
            .body
            .move_cooldowns
            .insert(e, MoveCooldown { remaining: 5 });
        world.body.icons.insert(e, Icon { ch: 'g' });
        world.body.names.insert(
            e,
            Name {
                value: "Goblin".to_string(),
            },
        );
        world.mind.nutritions.insert(e, Nutrition { value: 40.0 });
        world.mind.intentions.insert(
            e,
            Intention {
                action: ActionId::Idle,
                target: None,
            },
        );
        world.mind.action_states.insert(
            e,
            ActionState {
                current_action: None,
                ticks_in_action: 0,
                cooldowns: HashMap::new(),
            },
        );
        world.mind.wander_targets.insert(
            e,
            WanderTarget {
                goal_x: 3,
                goal_y: 7,
            },
        );
        world.mind.occupations.insert(
            e,
            Occupation {
                activity: "boulanger".to_string(),
                naics: "311".to_string(),
            },
        );
        world
            .gis
            .home_buildings
            .insert(e, HomeBuilding(crate::registry::BuildingId(1)));
        world
            .gis
            .workplaces
            .insert(e, Workplace(crate::registry::BuildingId(1)));

        world.despawn(e);

        assert!(!world.alive.contains(&e));
        assert!(!world.body.positions.contains_key(&e));
        assert!(!world.mind.hungers.contains_key(&e));
        assert!(!world.body.healths.contains_key(&e));
        assert!(!world.body.fatigues.contains_key(&e));
        assert!(!world.body.combat_stats.contains_key(&e));
        assert!(!world.body.gait_profiles.contains_key(&e));
        assert!(!world.body.current_gaits.contains_key(&e));
        assert!(!world.body.move_cooldowns.contains_key(&e));
        assert!(!world.body.icons.contains_key(&e));
        assert!(!world.body.names.contains_key(&e));
        assert!(!world.mind.nutritions.contains_key(&e));
        assert!(!world.mind.intentions.contains_key(&e));
        assert!(!world.mind.action_states.contains_key(&e));
        assert!(!world.mind.wander_targets.contains_key(&e));
        assert!(!world.mind.occupations.contains_key(&e));
        assert!(!world.gis.home_buildings.contains_key(&e));
        assert!(!world.gis.workplaces.contains_key(&e));
    }

    #[test]
    fn validate_passes_for_clean_world() {
        let world = World::new_with_seed(42);
        validate_world(&world);
    }

    #[test]
    fn validate_passes_with_entities() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 0, y: 0 });
        world.mind.hungers.insert(
            e,
            Hunger {
                current: 0.0,
                max: 100.0,
            },
        );
        validate_world(&world);
    }

    #[test]
    #[should_panic(expected = "zombie entity")]
    fn validate_catches_zombie_entity() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 0, y: 0 });
        world.alive.remove(&e); // Create zombie
        validate_world(&world);
    }

    #[test]
    fn new_with_seed_initializes_correctly() {
        let world = World::new_with_seed(42);
        assert!(world.alive.is_empty());
        assert!(world.pending_deaths.is_empty());
        assert!(world.body.positions.is_empty());
        assert!(world.spatial_index.is_empty());
        assert_eq!(world.tick, Tick(0));
        assert_eq!(world.tiles.width(), 64);
        assert_eq!(world.tiles.height(), 64);
    }

    #[test]
    fn spatial_index_rebuild_indexes_positions() {
        let mut world = World::new_with_seed(42);
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();
        world.body.positions.insert(e1, Position { x: 5, y: 10 });
        world.body.positions.insert(e2, Position { x: 5, y: 10 }); // same tile as e1
        world.body.positions.insert(e3, Position { x: 3, y: 7 });

        world.rebuild_spatial_index();

        let at_5_10: Vec<Entity> = world.entities_at(5, 10).collect();
        assert_eq!(at_5_10.len(), 2);
        assert!(at_5_10.contains(&e1));
        assert!(at_5_10.contains(&e2));

        let at_3_7: Vec<Entity> = world.entities_at(3, 7).collect();
        assert_eq!(at_3_7.len(), 1);
        assert!(at_3_7.contains(&e3));

        // Empty tile returns empty iterator.
        assert_eq!(world.entities_at(0, 0).count(), 0);
    }

    #[test]
    fn spatial_index_excludes_dead_entities() {
        let mut world = World::new_with_seed(42);
        let alive = world.spawn();
        let dead = world.spawn();
        world.body.positions.insert(alive, Position { x: 1, y: 1 });
        world.body.positions.insert(dead, Position { x: 1, y: 1 });

        // Simulate despawn: remove from alive but leave stale position entry.
        world.alive.remove(&dead);

        world.rebuild_spatial_index();

        let at_1_1: Vec<Entity> = world.entities_at(1, 1).collect();
        assert_eq!(at_1_1.len(), 1);
        assert!(at_1_1.contains(&alive));
        assert!(!at_1_1.contains(&dead));
    }

    #[test]
    fn entities_in_range_finds_nearby() {
        let mut world = World::new_with_seed(42);
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();
        world.body.positions.insert(e1, Position { x: 5, y: 5 });
        world.body.positions.insert(e2, Position { x: 7, y: 5 }); // Chebyshev dist 2
        world.body.positions.insert(e3, Position { x: 50, y: 50 }); // far away

        world.rebuild_spatial_index();

        let nearby: Vec<Entity> = world.entities_in_range(5, 5, 3).collect();
        assert!(nearby.contains(&e1));
        assert!(nearby.contains(&e2));
        assert!(!nearby.contains(&e3));
    }

    #[test]
    fn spatial_index_clears_on_rebuild() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 2, y: 3 });

        world.rebuild_spatial_index();
        assert_eq!(world.entities_at(2, 3).count(), 1);

        // Move entity to new position and rebuild.
        world.body.positions.insert(e, Position { x: 8, y: 9 });
        world.rebuild_spatial_index();

        // Old position is now empty, new position has the entity.
        assert_eq!(world.entities_at(2, 3).count(), 0);
        assert_eq!(world.entities_at(8, 9).count(), 1);
        assert!(world.entities_at(8, 9).any(|x| x == e));
    }
}
