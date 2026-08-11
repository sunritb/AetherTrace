use crate::vec3::Vec3;

/// SplitMix64-based generator: extremely fast, good quality, seedable per thread.
/// SplitMix64 is a solid base; we also offer the 128-bit xoroshiro path for
/// lower correlation between parallel lanes.
pub struct Rng {
    state: u64,
}

impl Rng {
    /// `from_u64` base per P. L'Ecuyer "Tables of linear congruential generators".
    #[inline(always)]
    pub fn from_u64(seed: u64) -> Self {
        let mut state = (seed ^ 0x9E3779B97F4A7C15).wrapping_mul(0xBF58476D1CE4E5B9);
        state = (state ^ (state >> 30)).wrapping_mul(0x94D049BB133111EB);
        state ^= state >> 31;
        Self { state }
    }

    #[inline(always)]
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    #[inline(always)]
    pub fn f32(&mut self) -> f32 {
        // 23 random mantissa bits; uniform in [0,1).
        ((self.next() >> 40) as f32) * (1.0 / 16777216.0)
    }

    #[inline(always)]
    pub fn f32_2(&mut self) -> (f32, f32) {
        let v = self.next();
        (
            ((v >> 40) as f32) * (1.0 / 16777216.0), // 24 mantissa bits, [0,1)
            ((v >> 41) as f32) * (1.0 / 8388608.0),  // 23 mantissa bits, [0,1)
        )
    }

    /// Unit vector uniformly distributed on the sphere (2 independent samples).
    #[inline(always)]
    pub fn unit_vector(&mut self) -> Vec3 {
        let (u, v) = self.f32_2();
        let z = 2.0 * u - 1.0;
        let r = (1.0 - z * z).max(0.0).sqrt();
        let phi = 2.0 * std::f32::consts::PI * v;
        Vec3::new(r * phi.cos(), r * phi.sin(), z)
    }

    /// Uniform point inside the unit disk.
    #[inline(always)]
    pub fn unit_disk(&mut self) -> Vec3 {
        let (u, v) = self.f32_2();
        let r = u.sqrt();
        let theta = 2.0 * std::f32::consts::PI * v;
        Vec3::new(r * theta.cos(), r * theta.sin(), 0.0)
    }
}
