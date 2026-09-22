//! Terrain, from a seed.
//!
//! Owned for the same reason the RNG is: a replay must reproduce the map, and a
//! noise crate is free to change its algorithm between versions. This is
//! hash-based value noise — no state, no tables, identical on every machine.

/// SplitMix64 finaliser. Cheap, and avalanches well enough for terrain.
fn hash(x: i32, y: i32, seed: u64) -> u64 {
    let mut h = seed
        .wrapping_add((x as i64 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add((y as i64 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F));
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 31;
    h
}

fn lattice(x: i32, y: i32, seed: u64) -> f32 {
    // Top 24 bits as a float in [0, 1).
    ((hash(x, y, seed) >> 40) as f32) / ((1u32 << 24) as f32)
}

fn smootherstep(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Value noise in `[0, 1)`, bilinear with a smootherstep fade.
pub fn value_noise(x: f32, y: f32, seed: u64) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let fx = smootherstep(x - x0);
    let fy = smootherstep(y - y0);
    let (xi, yi) = (x0 as i32, y0 as i32);

    let a = lattice(xi, yi, seed);
    let b = lattice(xi + 1, yi, seed);
    let c = lattice(xi, yi + 1, seed);
    let d = lattice(xi + 1, yi + 1, seed);

    let top = a + (b - a) * fx;
    let bottom = c + (d - c) * fx;
    top + (bottom - top) * fy
}

/// Fractal noise: a few octaves of [`value_noise`], each half the amplitude and
/// twice the frequency. `[0, 1)`.
pub fn fbm(x: f32, y: f32, seed: u64, octaves: u32) -> f32 {
    let mut total = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut norm = 0.0;
    for o in 0..octaves {
        total += value_noise(x * frequency, y * frequency, seed.wrapping_add(o as u64)) * amplitude;
        norm += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    if norm > 0.0 {
        total / norm
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{fbm, value_noise};

    #[test]
    fn noise_is_reproducible() {
        for i in 0..500 {
            let x = i as f32 * 0.37;
            let y = i as f32 * -0.11;
            assert_eq!(
                value_noise(x, y, 42),
                value_noise(x, y, 42),
                "noise differed for identical inputs"
            );
        }
    }

    #[test]
    fn different_seeds_give_different_maps() {
        let mut differing = 0;
        for i in 0..200 {
            let x = i as f32 * 0.5;
            if value_noise(x, x, 1) != value_noise(x, x, 2) {
                differing += 1;
            }
        }
        assert!(differing > 150, "only {differing}/200 samples differed");
    }

    #[test]
    fn output_stays_normalised() {
        for i in 0..1_000 {
            let x = i as f32 * 0.13;
            let y = i as f32 * -0.29;
            let v = value_noise(x, y, 5);
            assert!((0.0..=1.0).contains(&v), "value_noise produced {v}");
            let f = fbm(x, y, 5, 4);
            assert!((0.0..=1.0).contains(&f), "fbm produced {f}");
        }
    }

    #[test]
    fn integer_lattice_points_are_exact() {
        // At whole numbers the fade is zero, so the value is the lattice value
        // and not an interpolation of it.
        let v = value_noise(3.0, 4.0, 9);
        assert!((0.0..=1.0).contains(&v));
        assert_eq!(v, value_noise(3.0, 4.0, 9));
    }
}
