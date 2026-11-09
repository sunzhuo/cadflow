//! Deterministic Random Number Generation for DES.
//!
//! Uses PCG (Permuted Congruential Generator) via `rand_pcg::Pcg64Mcg`,
//! providing high quality, deterministic, seed-reproducible pseudo-random streams
//! across different architectures.

use rand::Rng;
use rand::SeedableRng;
use rand_distr::{Exp, Normal, Uniform, Distribution};
use rand_pcg::Pcg64Mcg;

#[derive(Clone, Debug)]
pub struct DeterministicRng {
    seed: u64,
    rng: Pcg64Mcg,
}

impl DeterministicRng {
    /// Initialize with a 64-bit seed.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            rng: Pcg64Mcg::seed_from_u64(seed),
        }
    }

    /// Reseed the generator with a new seed.
    pub fn reseed(&mut self, seed: u64) {
        self.seed = seed;
        self.rng = Pcg64Mcg::seed_from_u64(seed);
    }

    /// Current seed value.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Uniform real distribution in `[low, high)`.
    pub fn uniform(&mut self, low: f64, high: f64) -> f64 {
        if (high - low).abs() < 1e-12 {
            return low;
        }
        let (min, max) = if low <= high { (low, high) } else { (high, low) };
        let dist = Uniform::new(min, max);
        dist.sample(&mut self.rng)
    }

    /// Standard unit real distribution in `[0.0, 1.0)`.
    pub fn unit(&mut self) -> f64 {
        self.rng.gen_range(0.0..1.0)
    }

    /// Exponential distribution with rate parameter `lambda`.
    pub fn exponential(&mut self, lambda: f64) -> f64 {
        let safe_lambda = if lambda > 0.0 { lambda } else { 1.0 };
        match Exp::new(safe_lambda) {
            Ok(dist) => dist.sample(&mut self.rng),
            Err(_) => 0.0,
        }
    }

    /// Normal (Gaussian) distribution with given `mean` and `std_dev`.
    pub fn normal(&mut self, mean: f64, std_dev: f64) -> f64 {
        let safe_std_dev = if std_dev > 0.0 { std_dev } else { 1.0 };
        match Normal::new(mean, safe_std_dev) {
            Ok(dist) => dist.sample(&mut self.rng),
            Err(_) => mean,
        }
    }

    /// Bernoulli trial returning true with probability `p` in `[0.0, 1.0]`.
    pub fn bool_with_prob(&mut self, p: f64) -> bool {
        if p <= 0.0 {
            false
        } else if p >= 1.0 {
            true
        } else {
            self.rng.gen_bool(p)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reproducibility() {
        let mut rng1 = DeterministicRng::new(42);
        let mut rng2 = DeterministicRng::new(42);

        let vals1: Vec<f64> = (0..100).map(|_| rng1.uniform(0.0, 10.0)).collect();
        let vals2: Vec<f64> = (0..100).map(|_| rng2.uniform(0.0, 10.0)).collect();

        assert_eq!(vals1, vals2);
    }

    #[test]
    fn test_distributions() {
        let mut rng = DeterministicRng::new(12345);
        for _ in 0..50 {
            let u = rng.unit();
            assert!((0.0..1.0).contains(&u));

            let e = rng.exponential(2.0);
            assert!(e >= 0.0);

            let _n = rng.normal(10.0, 2.0);
        }
    }
}
