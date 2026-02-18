use crate::components::{ActionId, Entity, Gait, Tick};
use crate::world::World;

/// Phase 4 (Actions, before wander): Select gait based on intention and stamina.
///
/// - Charge: Sprint if stamina > 30%, Run if > 10%, else Hustle
/// - Flee: Sprint if stamina > 20%, Run if > 5%, else Walk
/// - Attack/Eat/Wander: Walk
/// - Idle: Stroll (recovers stamina faster)
///
/// Exhausted chargers degrade to Hustle (still faster than Walk).
/// Exhausted fleers drop to Walk (vulnerable — predators sustain pursuit better).
pub fn run_gait_selection(world: &mut World, _tick: Tick) {
    let changes: Vec<(Entity, Gait)> = world
        .gait_profiles
        .keys()
        .filter(|e| !world.pending_deaths.contains(e))
        .copied()
        .map(|e| {
            let action = world
                .intentions
                .get(&e)
                .map(|i| i.action)
                .unwrap_or(ActionId::Idle);

            let stamina_ratio = world
                .staminas
                .get(&e)
                .map(|s| if s.max > 0.0 { s.current / s.max } else { 1.0 })
                .unwrap_or(1.0);

            let gait = match action {
                ActionId::Charge => {
                    if stamina_ratio > 0.3 {
                        Gait::Sprint
                    } else if stamina_ratio > 0.1 {
                        Gait::Run
                    } else {
                        Gait::Hustle
                    }
                }
                ActionId::Flee => {
                    if stamina_ratio > 0.2 {
                        Gait::Sprint
                    } else if stamina_ratio > 0.05 {
                        Gait::Run
                    } else {
                        Gait::Walk
                    }
                }
                ActionId::Attack | ActionId::Eat | ActionId::Wander => Gait::Walk,
                ActionId::Idle => Gait::Stroll,
            };

            (e, gait)
        })
        .collect();

    for (e, gait) in changes {
        world.current_gaits.insert(e, gait);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::world::World;

    fn setup_entity(world: &mut World, action: ActionId, stamina_ratio: f32) -> Entity {
        let e = world.spawn();
        world.gait_profiles.insert(e, GaitProfile::biped());
        world.current_gaits.insert(e, Gait::Walk);
        world.staminas.insert(
            e,
            Stamina {
                current: stamina_ratio * 100.0,
                max: 100.0,
            },
        );
        world.intentions.insert(
            e,
            Intention {
                action,
                target: None,
            },
        );
        e
    }

    #[test]
    fn test_charge_high_stamina_sprints() {
        let mut world = World::new_with_seed(42);
        let e = setup_entity(&mut world, ActionId::Charge, 0.5);

        run_gait_selection(&mut world, Tick(0));
        assert_eq!(world.current_gaits[&e], Gait::Sprint);
    }

    #[test]
    fn test_charge_medium_stamina_runs() {
        let mut world = World::new_with_seed(42);
        let e = setup_entity(&mut world, ActionId::Charge, 0.2);

        run_gait_selection(&mut world, Tick(0));
        assert_eq!(world.current_gaits[&e], Gait::Run);
    }

    #[test]
    fn test_charge_low_stamina_hustles() {
        let mut world = World::new_with_seed(42);
        let e = setup_entity(&mut world, ActionId::Charge, 0.05);

        run_gait_selection(&mut world, Tick(0));
        assert_eq!(world.current_gaits[&e], Gait::Hustle);
    }

    #[test]
    fn test_flee_high_stamina_sprints() {
        let mut world = World::new_with_seed(42);
        let e = setup_entity(&mut world, ActionId::Flee, 0.5);

        run_gait_selection(&mut world, Tick(0));
        assert_eq!(world.current_gaits[&e], Gait::Sprint);
    }

    #[test]
    fn test_flee_medium_stamina_runs() {
        let mut world = World::new_with_seed(42);
        let e = setup_entity(&mut world, ActionId::Flee, 0.1);

        run_gait_selection(&mut world, Tick(0));
        assert_eq!(world.current_gaits[&e], Gait::Run);
    }

    #[test]
    fn test_flee_exhausted_walks() {
        let mut world = World::new_with_seed(42);
        let e = setup_entity(&mut world, ActionId::Flee, 0.03);

        run_gait_selection(&mut world, Tick(0));
        assert_eq!(world.current_gaits[&e], Gait::Walk);
    }

    #[test]
    fn test_attack_always_walks() {
        let mut world = World::new_with_seed(42);
        let e = setup_entity(&mut world, ActionId::Attack, 1.0);

        run_gait_selection(&mut world, Tick(0));
        assert_eq!(world.current_gaits[&e], Gait::Walk);
    }

    #[test]
    fn test_idle_strolls() {
        let mut world = World::new_with_seed(42);
        let e = setup_entity(&mut world, ActionId::Idle, 1.0);

        run_gait_selection(&mut world, Tick(0));
        assert_eq!(world.current_gaits[&e], Gait::Stroll);
    }

    #[test]
    fn test_skips_pending_death() {
        let mut world = World::new_with_seed(42);
        let e = setup_entity(&mut world, ActionId::Charge, 1.0);
        world.pending_deaths.push(e);

        run_gait_selection(&mut world, Tick(0));
        // Should remain Walk (unchanged from setup)
        assert_eq!(world.current_gaits[&e], Gait::Walk);
    }

    #[test]
    fn test_no_gait_profile_skipped() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        // No gait_profiles entry — should not appear in output
        world.intentions.insert(
            e,
            Intention {
                action: ActionId::Charge,
                target: None,
            },
        );

        run_gait_selection(&mut world, Tick(0));
        assert!(!world.current_gaits.contains_key(&e));
    }
}
