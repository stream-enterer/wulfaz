use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

/// Unique entity identifier. Never use raw u64 where an Entity is meant.
/// Never cast between Entity and Tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity(pub u64);

impl Hash for Entity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Simulation tick counter. Never use raw u64 where a Tick is meant.
/// Never cast between Tick and Entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tick(pub u64);

/// Spatial position on the tile grid.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Hunger need — increases over time, reduced by eating.
#[derive(Debug, Clone, Copy)]
pub struct Hunger {
    pub current: f32,
    pub max: f32,
}

/// Health points — reduced by combat/damage, entity dies at 0.
#[derive(Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

/// Fatigue — accumulated from combat, degrades effectiveness.
/// Starts at 0. Effects: -1 defense per 10, -1 attack per 20.
/// At 100: unconscious. Over 200: excess converts to HP damage.
#[derive(Debug, Clone, Copy)]
pub struct Fatigue {
    pub current: f32,
}

/// Combat stats for entities that can fight.
#[derive(Debug, Clone, Copy)]
pub struct CombatStats {
    pub attack: f32,
    pub defense: f32,
    pub aggression: f32,
}

/// Gait tier — determines movement speed. All creatures share the same
/// slow gaits (Creep/Stroll/Walk); fast gaits differ by body plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Gait {
    Creep,  // 29 ticks/tile — 3.4 tiles/sec
    Stroll, // 19 ticks/tile — 5.3 tiles/sec
    Walk,   // 9 ticks/tile  — 11.1 tiles/sec (DF default)
    Hustle, // Jog (biped 7) / Trot (quadruped 4)
    Run,    // Run (biped 5) / Canter (quadruped 3)
    Sprint, // Sprint (biped 3) / Gallop (quadruped 2)
}

/// Movement cooldowns (ticks per tile) for each gait tier.
/// Index order matches Gait enum variants.
#[derive(Debug, Clone)]
pub struct GaitProfile {
    pub cooldowns: [u32; 6],
}

#[allow(dead_code)]
impl GaitProfile {
    /// Get the ticks-per-tile cooldown for a gait.
    pub fn cooldown(&self, gait: Gait) -> u32 {
        self.cooldowns[gait as usize]
    }

    /// Standard biped gaits matching DF dwarf.
    pub fn biped() -> Self {
        GaitProfile {
            cooldowns: [29, 19, 9, 7, 5, 3],
        }
    }

    /// Standard quadruped gaits matching DF wolf/deer.
    pub fn quadruped() -> Self {
        GaitProfile {
            cooldowns: [29, 19, 9, 4, 3, 2],
        }
    }
}

/// Ticks remaining until this entity can move again.
/// Wander system decrements each tick; moves only when remaining == 0.
#[derive(Debug, Clone, Copy)]
pub struct MoveCooldown {
    pub remaining: u32,
}

/// Display icon for rendering (single character).
#[derive(Debug, Clone)]
pub struct Icon {
    pub ch: char,
}

/// Name of the entity (creature type, item type, etc.).
#[derive(Debug, Clone)]
pub struct Name {
    pub value: String,
}

/// Nutrition value for food items.
#[derive(Debug, Clone, Copy)]
pub struct Nutrition {
    pub value: f32,
}

/// Available actions for the utility scorer. Variant order determines tiebreaking priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActionId {
    Idle,
    Wander,
    Eat,
    Attack,
}

/// What an entity intends to do this tick, written by the Phase 3 scorer.
#[derive(Debug, Clone)]
pub struct Intention {
    pub action: ActionId,
    pub target: Option<Entity>,
}

/// Cached wander destination for A* pathfinding.
#[derive(Debug, Clone, Copy)]
pub struct WanderTarget {
    pub goal_x: i32,
    pub goal_y: i32,
}

/// Per-entity scoring state: current action, how long it's been doing it, cooldowns.
#[derive(Debug, Clone)]
pub struct ActionState {
    pub current_action: Option<ActionId>,
    pub ticks_in_action: u64,
    pub cooldowns: HashMap<ActionId, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn entity_can_be_hashmap_key() {
        let mut map: HashMap<Entity, i32> = HashMap::new();
        let e1 = Entity(1);
        let e2 = Entity(2);
        map.insert(e1, 10);
        map.insert(e2, 20);
        assert_eq!(map[&e1], 10);
        assert_eq!(map[&e2], 20);
    }

    #[test]
    fn entity_can_be_in_hashset() {
        let mut set: HashSet<Entity> = HashSet::new();
        let e = Entity(42);
        set.insert(e);
        assert!(set.contains(&e));
        assert!(!set.contains(&Entity(99)));
    }

    #[test]
    fn entity_equality() {
        assert_eq!(Entity(1), Entity(1));
        assert_ne!(Entity(1), Entity(2));
    }

    #[test]
    fn entity_copy_semantics() {
        let e1 = Entity(5);
        let e2 = e1; // Copy
        assert_eq!(e1, e2); // e1 still usable
    }

    #[test]
    fn tick_ordering() {
        assert!(Tick(0) < Tick(1));
        assert!(Tick(100) > Tick(50));
        assert_eq!(Tick(7), Tick(7));
    }

    #[test]
    fn tick_copy_semantics() {
        let t1 = Tick(10);
        let t2 = t1; // Copy
        assert_eq!(t1, t2); // t1 still usable
    }

    #[test]
    fn position_fields() {
        let pos = Position { x: -3, y: 7 };
        assert_eq!(pos.x, -3);
        assert_eq!(pos.y, 7);
    }

    #[test]
    fn hunger_fields() {
        let h = Hunger {
            current: 25.0,
            max: 100.0,
        };
        assert_eq!(h.current, 25.0);
        assert_eq!(h.max, 100.0);
    }

    #[test]
    fn health_fields() {
        let hp = Health {
            current: 80.0,
            max: 100.0,
        };
        assert_eq!(hp.current, 80.0);
        assert_eq!(hp.max, 100.0);
    }

    #[test]
    fn combat_stats_fields() {
        let cs = CombatStats {
            attack: 10.0,
            defense: 5.0,
            aggression: 0.8,
        };
        assert_eq!(cs.attack, 10.0);
        assert_eq!(cs.defense, 5.0);
        assert_eq!(cs.aggression, 0.8);
    }

    #[test]
    fn gait_profile_biped() {
        let p = GaitProfile::biped();
        assert_eq!(p.cooldown(Gait::Walk), 9);
        assert_eq!(p.cooldown(Gait::Sprint), 3);
    }

    #[test]
    fn gait_profile_quadruped() {
        let p = GaitProfile::quadruped();
        assert_eq!(p.cooldown(Gait::Walk), 9);
        assert_eq!(p.cooldown(Gait::Sprint), 2);
    }

    #[test]
    fn icon_field() {
        let i = Icon { ch: 'g' };
        assert_eq!(i.ch, 'g');
    }

    #[test]
    fn name_field() {
        let n = Name {
            value: "Goblin".to_string(),
        };
        assert_eq!(n.value, "Goblin");
    }

    #[test]
    fn nutrition_field() {
        let n = Nutrition { value: 25.0 };
        assert_eq!(n.value, 25.0);
    }

    #[test]
    fn fatigue_fields() {
        let f = Fatigue { current: 30.0 };
        assert_eq!(f.current, 30.0);
    }
}
