use rand::SeedableRng;
use rand::rngs::StdRng;

/// Create a deterministic StdRng from a u64 seed.
/// This is the ONLY way to create an RNG in the simulation.
/// All randomness flows through world.rng which is created by this function.
pub fn create_rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

    #[test]
    fn same_seed_produces_same_sequence() {
        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let seq1: Vec<u64> = (0..10).map(|_| rng1.random::<u64>()).collect();
        let seq2: Vec<u64> = (0..10).map(|_| rng2.random::<u64>()).collect();

        assert_eq!(seq1, seq2);
    }

    #[test]
    fn different_seeds_produce_different_sequences() {
        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(99);

        let val1: u64 = rng1.random();
        let val2: u64 = rng2.random();

        assert_ne!(val1, val2);
    }
}
