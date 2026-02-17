use std::collections::{HashMap, HashSet};

use rand::rngs::StdRng;

use crate::components::*;
use crate::events::EventLog;
use crate::rng::create_rng;
use crate::systems::decisions::UtilityConfig;
use crate::tile_map::TileMap;

pub struct World {
    // Entity tracking
    pub alive: HashSet<Entity>,
    pub pending_deaths: Vec<Entity>,
    next_entity_id: u64,

    // Property tables (HashMap<Entity, T>)
    pub positions: HashMap<Entity, Position>,
    pub hungers: HashMap<Entity, Hunger>,
    pub healths: HashMap<Entity, Health>,
    pub combat_stats: HashMap<Entity, CombatStats>,
    pub speeds: HashMap<Entity, Speed>,
    pub move_cooldowns: HashMap<Entity, MoveCooldown>,
    pub icons: HashMap<Entity, Icon>,
    pub names: HashMap<Entity, Name>,
    pub nutritions: HashMap<Entity, Nutrition>,
    pub intentions: HashMap<Entity, Intention>,
    pub action_states: HashMap<Entity, ActionState>,
    pub wander_targets: HashMap<Entity, WanderTarget>,

    // Non-property-table fields
    pub tiles: TileMap,
    pub events: EventLog,
    pub rng: StdRng,
    pub tick: Tick,
    pub utility_config: UtilityConfig,
}

impl World {
    /// Create a new World with all fields initialized and a deterministic RNG seed.
    pub fn new_with_seed(seed: u64) -> Self {
        Self {
            alive: HashSet::new(),
            pending_deaths: Vec::new(),
            next_entity_id: 1, // 0 is reserved/unused

            positions: HashMap::new(),
            hungers: HashMap::new(),
            healths: HashMap::new(),
            combat_stats: HashMap::new(),
            speeds: HashMap::new(),
            move_cooldowns: HashMap::new(),
            icons: HashMap::new(),
            names: HashMap::new(),
            nutritions: HashMap::new(),
            intentions: HashMap::new(),
            action_states: HashMap::new(),
            wander_targets: HashMap::new(),

            tiles: TileMap::new(64, 64), // 64m × 64m
            events: EventLog::default_capacity(),
            rng: create_rng(seed),
            tick: Tick(0),
            utility_config: UtilityConfig::default(),
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

    /// Remove an entity from ALL tables. Called ONLY by run_death.
    /// CRITICAL: Every HashMap property table MUST have a .remove() call here.
    /// If you add a new property table to World, add a corresponding remove here.
    pub fn despawn(&mut self, entity: Entity) {
        self.alive.remove(&entity);
        self.positions.remove(&entity);
        self.hungers.remove(&entity);
        self.healths.remove(&entity);
        self.combat_stats.remove(&entity);
        self.speeds.remove(&entity);
        self.move_cooldowns.remove(&entity);
        self.icons.remove(&entity);
        self.names.remove(&entity);
        self.nutritions.remove(&entity);
        self.intentions.remove(&entity);
        self.action_states.remove(&entity);
        self.wander_targets.remove(&entity);
    }
}

/// Validate world invariants. Run every tick in debug builds.
/// Checks that no entity exists in any property table without being in alive.
pub fn validate_world(world: &World) {
    // Check: no entity in any property table is missing from alive
    for entity in world.positions.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in positions but not in alive",
            entity
        );
    }

    for entity in world.hungers.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in hungers but not in alive",
            entity
        );
    }

    for entity in world.healths.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in healths but not in alive",
            entity
        );
    }

    for entity in world.combat_stats.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in combat_stats but not in alive",
            entity
        );
    }

    for entity in world.speeds.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in speeds but not in alive",
            entity
        );
    }

    for entity in world.move_cooldowns.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in move_cooldowns but not in alive",
            entity
        );
    }

    for entity in world.icons.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in icons but not in alive",
            entity
        );
    }

    for entity in world.names.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in names but not in alive",
            entity
        );
    }

    for entity in world.nutritions.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in nutritions but not in alive",
            entity
        );
    }

    for entity in world.intentions.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in intentions but not in alive",
            entity
        );
    }

    for entity in world.action_states.keys() {
        assert!(
            world.alive.contains(entity),
            "zombie entity {:?} in action_states but not in alive",
            entity
        );
    }

    for entity in world.wander_targets.keys() {
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
        world.positions.insert(e, Position { x: 5, y: 10 });
        world.hungers.insert(
            e,
            Hunger {
                current: 50.0,
                max: 100.0,
            },
        );
        world.healths.insert(
            e,
            Health {
                current: 80.0,
                max: 100.0,
            },
        );
        world.combat_stats.insert(
            e,
            CombatStats {
                attack: 10.0,
                defense: 5.0,
                aggression: 0.8,
            },
        );
        world.speeds.insert(e, Speed { value: 2 });
        world
            .move_cooldowns
            .insert(e, MoveCooldown { remaining: 5 });
        world.icons.insert(e, Icon { ch: 'g' });
        world.names.insert(
            e,
            Name {
                value: "Goblin".to_string(),
            },
        );
        world.nutritions.insert(e, Nutrition { value: 40.0 });
        world.intentions.insert(
            e,
            Intention {
                action: ActionId::Idle,
                target: None,
            },
        );
        world.action_states.insert(
            e,
            ActionState {
                current_action: None,
                ticks_in_action: 0,
                cooldowns: HashMap::new(),
            },
        );
        world.wander_targets.insert(
            e,
            WanderTarget {
                goal_x: 3,
                goal_y: 7,
            },
        );

        world.despawn(e);

        assert!(!world.alive.contains(&e));
        assert!(!world.positions.contains_key(&e));
        assert!(!world.hungers.contains_key(&e));
        assert!(!world.healths.contains_key(&e));
        assert!(!world.combat_stats.contains_key(&e));
        assert!(!world.speeds.contains_key(&e));
        assert!(!world.move_cooldowns.contains_key(&e));
        assert!(!world.icons.contains_key(&e));
        assert!(!world.names.contains_key(&e));
        assert!(!world.nutritions.contains_key(&e));
        assert!(!world.intentions.contains_key(&e));
        assert!(!world.action_states.contains_key(&e));
        assert!(!world.wander_targets.contains_key(&e));
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
        world.positions.insert(e, Position { x: 0, y: 0 });
        world.hungers.insert(
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
        world.positions.insert(e, Position { x: 0, y: 0 });
        world.alive.remove(&e); // Create zombie
        validate_world(&world);
    }

    #[test]
    fn new_with_seed_initializes_correctly() {
        let world = World::new_with_seed(42);
        assert!(world.alive.is_empty());
        assert!(world.pending_deaths.is_empty());
        assert!(world.positions.is_empty());
        assert_eq!(world.tick, Tick(0));
        assert_eq!(world.tiles.width(), 64);
        assert_eq!(world.tiles.height(), 64);
    }
}
