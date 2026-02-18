use crate::components::{Entity, Gait, Tick};
use crate::world::World;

/// Stamina drain/recovery rate per tick based on current gait.
/// Fast gaits drain stamina; slow gaits recover it.
fn stamina_rate(gait: Gait) -> f32 {
    match gait {
        Gait::Sprint => -2.0, // drains 100 in 50 ticks (0.5s)
        Gait::Run => -1.0,    // drains 100 in 100 ticks (1.0s)
        Gait::Hustle => -0.3, // drains 100 in ~333 ticks (3.3s)
        Gait::Walk => 0.5,    // recovers 100 in 200 ticks (2.0s)
        Gait::Stroll => 1.0,  // recovers 100 in 100 ticks (1.0s)
        Gait::Creep => 2.0,   // recovers 100 in 50 ticks (0.5s)
    }
}

/// Phase 2 (Needs): Update stamina based on the previous tick's gait.
///
/// Drains stamina for fast gaits (Hustle/Run/Sprint) and recovers for slow
/// gaits (Walk/Stroll/Creep). Clamped to [0, max]. Skips pending deaths.
pub fn run_stamina(world: &mut World, _tick: Tick) {
    let changes: Vec<(Entity, f32)> = world
        .staminas
        .iter()
        .filter(|&(&e, _)| !world.pending_deaths.contains(&e))
        .map(|(&e, s)| {
            let gait = world.current_gaits.get(&e).copied().unwrap_or(Gait::Walk);
            let rate = stamina_rate(gait);
            let new_val = (s.current + rate).clamp(0.0, s.max);
            (e, new_val)
        })
        .collect();

    for (e, new_val) in changes {
        if let Some(s) = world.staminas.get_mut(&e) {
            s.current = new_val;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::world::World;

    #[test]
    fn test_stamina_drains_at_sprint() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.staminas.insert(
            e,
            Stamina {
                current: 100.0,
                max: 100.0,
            },
        );
        world.current_gaits.insert(e, Gait::Sprint);

        run_stamina(&mut world, Tick(0));
        assert!((world.staminas[&e].current - 98.0).abs() < 0.001);
    }

    #[test]
    fn test_stamina_recovers_at_walk() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.staminas.insert(
            e,
            Stamina {
                current: 50.0,
                max: 100.0,
            },
        );
        world.current_gaits.insert(e, Gait::Walk);

        run_stamina(&mut world, Tick(0));
        assert!((world.staminas[&e].current - 50.5).abs() < 0.001);
    }

    #[test]
    fn test_stamina_clamps_to_zero() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.staminas.insert(
            e,
            Stamina {
                current: 1.0,
                max: 100.0,
            },
        );
        world.current_gaits.insert(e, Gait::Sprint);

        run_stamina(&mut world, Tick(0));
        assert_eq!(world.staminas[&e].current, 0.0);
    }

    #[test]
    fn test_stamina_clamps_to_max() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.staminas.insert(
            e,
            Stamina {
                current: 99.5,
                max: 100.0,
            },
        );
        world.current_gaits.insert(e, Gait::Walk);

        run_stamina(&mut world, Tick(0));
        assert_eq!(world.staminas[&e].current, 100.0);
    }

    #[test]
    fn test_stamina_skips_pending_death() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.staminas.insert(
            e,
            Stamina {
                current: 100.0,
                max: 100.0,
            },
        );
        world.current_gaits.insert(e, Gait::Sprint);
        world.pending_deaths.push(e);

        run_stamina(&mut world, Tick(0));
        assert_eq!(world.staminas[&e].current, 100.0);
    }

    #[test]
    fn test_stamina_defaults_to_walk_rate_without_gait() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.staminas.insert(
            e,
            Stamina {
                current: 50.0,
                max: 100.0,
            },
        );
        // No current_gaits entry → defaults to Walk (+0.5)

        run_stamina(&mut world, Tick(0));
        assert!((world.staminas[&e].current - 50.5).abs() < 0.001);
    }

    #[test]
    fn test_hustle_drain_rate() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.staminas.insert(
            e,
            Stamina {
                current: 100.0,
                max: 100.0,
            },
        );
        world.current_gaits.insert(e, Gait::Hustle);

        run_stamina(&mut world, Tick(0));
        assert!((world.staminas[&e].current - 99.7).abs() < 0.001);
    }
}
