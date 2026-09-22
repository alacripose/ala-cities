//! A deterministic RNG, owned rather than borrowed.
//!
//! `rand`'s generators are explicitly not stable across major versions, and a
//! replay re-runs a season's worth of decisions. The numbers a replay depends
//! on must not be able to change underneath it when a dependency updates.
//! PCG32 is forty lines and will still produce this exact sequence in a decade.

use serde::{Deserialize, Serialize};

const MULT: u64 = 6_364_136_223_846_793_005;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pcg32 {
    state: u64,
    inc: u64,
}

impl Pcg32 {
    /// A generator with a fixed stream, seeded. Two generators built from the
    /// same seed produce identical sequences forever.
    pub fn new(seed: u64) -> Self {
        let mut rng = Self {
            state: 0,
            inc: (seed << 1) | 1,
        };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(seed);
        rng.next_u32();
        rng
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(MULT).wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// Uniform in `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }

    /// Uniform integer in `[0, n)`. Returns 0 when `n` is 0 rather than
    /// panicking, because a division by zero here would take the city with it.
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            0
        } else {
            self.next_u32() % n
        }
    }

    /// True with probability `p`.
    pub fn chance(&mut self, p: f32) -> bool {
        self.next_f32() < p
    }
}

#[cfg(test)]
mod tests {
    use super::Pcg32;

    #[test]
    fn the_same_seed_gives_the_same_sequence() {
        let mut a = Pcg32::new(0xC117_0001);
        let mut b = Pcg32::new(0xC117_0001);
        for _ in 0..1_000 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Pcg32::new(1);
        let mut b = Pcg32::new(2);
        let mut same = 0;
        for _ in 0..1_000 {
            if a.next_u32() == b.next_u32() {
                same += 1;
            }
        }
        assert!(same < 5, "seeds 1 and 2 produced {same} identical draws");
    }

    #[test]
    fn floats_stay_in_range_and_are_not_stuck() {
        let mut rng = Pcg32::new(7);
        let mut low = 0;
        let mut high = 0;
        for _ in 0..10_000 {
            let f = rng.next_f32();
            assert!((0.0..1.0).contains(&f), "next_f32 produced {f}");
            if f < 0.5 {
                low += 1;
            } else {
                high += 1;
            }
        }
        assert!(low > 4_000 && high > 4_000, "{low} low / {high} high");
    }

    #[test]
    fn below_never_exceeds_its_bound() {
        let mut rng = Pcg32::new(11);
        for n in 1..64u32 {
            for _ in 0..100 {
                assert!(rng.below(n) < n);
            }
        }
        assert_eq!(rng.below(0), 0);
    }

    #[test]
    fn state_survives_a_round_trip() {
        let mut rng = Pcg32::new(99);
        for _ in 0..50 {
            rng.next_u32();
        }
        let json = serde_json::to_string(&rng).expect("serialise");
        let mut restored: Pcg32 = serde_json::from_str(&json).expect("deserialise");
        let mut expected = rng.clone();
        for _ in 0..100 {
            assert_eq!(restored.next_u32(), expected.next_u32());
        }
    }
}
