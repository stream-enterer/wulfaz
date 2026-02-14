use crate::components::Tick;
use crate::world::World;

pub fn run_death(world: &mut World, tick: Tick) {
    let _ = tick; // available for future use

    let to_despawn: Vec<_> = world.pending_deaths.drain(..).collect();
    for entity in to_despawn {
        world.despawn(entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::world::{World, validate_world};

    #[test]
    fn test_death_despawns_entities() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 5, y: 5 });
        world.hungers.insert(e, Hunger { current: 50.0, max: 100.0 });

        world.pending_deaths.push(e);
        run_death(&mut world, Tick(0));

        assert!(!world.alive.contains(&e));
        assert!(!world.positions.contains_key(&e));
        assert!(!world.hungers.contains_key(&e));
        assert!(world.pending_deaths.is_empty());
    }

    #[test]
    fn test_death_clears_pending_deaths() {
        let mut world = World::new_with_seed(42);
        let e1 = world.spawn();
        let e2 = world.spawn();
        world.pending_deaths.push(e1);
        world.pending_deaths.push(e2);

        run_death(&mut world, Tick(0));

        assert!(world.pending_deaths.is_empty());
        assert!(!world.alive.contains(&e1));
        assert!(!world.alive.contains(&e2));
    }

    #[test]
    fn test_death_validates_clean() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 0, y: 0 });
        world.pending_deaths.push(e);

        run_death(&mut world, Tick(0));
        validate_world(&world); // should pass — no zombies
    }

    #[test]
    fn test_no_pending_deaths_is_noop() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 5, y: 5 });

        run_death(&mut world, Tick(0));

        assert!(world.alive.contains(&e)); // still alive
        assert!(world.positions.contains_key(&e)); // still has position
    }
}
