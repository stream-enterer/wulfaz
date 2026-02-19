use std::collections::{HashMap, HashSet};

use rand::rngs::StdRng;

use crate::components::*;
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
            utility_config: UtilityConfig::default(),
        }
    }

    fn remove(&mut self, entity: &Entity) {
        self.hungers.remove(entity);
        self.nutritions.remove(entity);
        self.intentions.remove(entity);
        self.action_states.remove(entity);
        self.wander_targets.remove(entity);
    }
}

pub struct GisTables {
    pub buildings: BuildingRegistry,
    pub blocks: BlockRegistry,
    /// Maps quartier_id (1-based index) to quartier name string.
    pub quartier_names: Vec<String>,
    /// Street registry, reconstructed from building address data.
    #[allow(dead_code)]
    pub streets: StreetRegistry,
    /// Active SoDUCo snapshot year for occupant display.
    #[allow(dead_code)]
    pub active_year: u16,
}

impl GisTables {
    fn new() -> Self {
        Self {
            buildings: BuildingRegistry::new(),
            blocks: BlockRegistry::new(),
            quartier_names: Vec::new(),
            streets: StreetRegistry::new(),
            active_year: 1839,
        }
    }
}

pub struct World {
    // Entity tracking
    pub alive: HashSet<Entity>,
    pub pending_deaths: Vec<Entity>,
    #[allow(dead_code)]
    next_entity_id: u64,

    // Sub-struct property tables
    pub body: BodyTables,
    pub mind: MindTables,
    pub gis: GisTables,

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
            pending_deaths: Vec::new(),
            next_entity_id: 1, // 0 is reserved/unused

            body: BodyTables::new(),
            mind: MindTables::new(),
            gis: GisTables::new(),

            tiles: TileMap::new(64, 64), // 64m × 64m
            events: EventLog::default_capacity(),
            rng: create_rng(seed),
            tick: Tick(0),
            player: None,
        }
    }

    #[allow(dead_code)]
    /// Spawn a new entity. Returns the Entity with a unique ID.
    /// The entity is added to the alive set but has no components yet.
    pub fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next_entity_id);
        self.next_entity_id += 1;
        self.alive.insert(entity);
        entity
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
        assert_eq!(world.tick, Tick(0));
        assert_eq!(world.tiles.width(), 64);
        assert_eq!(world.tiles.height(), 64);
    }
}
