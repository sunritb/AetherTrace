use crate::rng::Rng;
use crate::vec3::{Color, Vec3};

#[derive(Clone, Copy, Debug)]
pub enum Environment {
    /// Procedural gradient sky with an importance-sampled sun disk.
    Sky {
        sun_dir: Vec3,
        sun_color: Color,
        sun_angular_radius: f32,
        glow: f32,
        zenith: Color,
        horizon: Color,
        ground: Color,
    },
    /// Single solid background color.
    Constant(Color),
    /// Two-hemisphere studio gradient (top/bottom).
    Studio {
        top: Color,
        bottom: Color,
        horizon: Color,
    },
}

const FRAC_1_4PI: f32 = 0.079_577_47; // 1 / (4*pi)
const P_SUN: f32 = 0.5;

impl Environment {
    pub fn sky(sun_dir: Vec3, sun_color: Color) -> Self {
        Environment::Sky {
            sun_dir: sun_dir.normalize(),
            sun_color,
            sun_angular_radius: 0.0060,
            glow: 2.5,
            zenith: Color::new(0.06, 0.16, 0.38),
            horizon: Color::new(0.55, 0.72, 0.92),
            ground: Color::new(0.06, 0.05, 0.045),
        }
    }

    #[inline]
    pub fn radiance(&self, dir: Vec3) -> Color {
        match self {
            Environment::Constant(c) => *c,
            Environment::Studio {
                top,
                bottom,
                horizon,
            } => {
                let h = 0.5 + 0.5 * dir.y;
                let g = 1.0 - h;
                *top * (h * h) + *bottom * (g * g) + *horizon * (2.0 * h * g)
            }
            Environment::Sky {
                sun_dir,
                sun_color,
                sun_angular_radius,
                glow,
                zenith,
                horizon,
                ground,
            } => {
                let d = dir.normalize();
                let up = d.y;
                let base = if up >= 0.0 {
                    let t = up.powf(0.55);
                    (*horizon) * (1.0 - t) + (*zenith) * t
                } else {
                    *ground
                };
                let s = d.dot(*sun_dir);
                if s <= 0.0 {
                    return base;
                }
                // Broad atmospheric glow around the sun.
                let glow_term = s.powf(48.0) * *glow;
                // Sharp sun disk.
                let cos_alpha = sun_angular_radius.cos();
                let disk = if s >= cos_alpha {
                    *sun_color
                } else {
                    Color::zero()
                };
                base + (*sun_color) * glow_term + disk
            }
        }
    }

    /// Solid angle PDF of sampling direction `dir`.
    #[inline]
    pub fn pdf(&self, dir: Vec3) -> f32 {
        match self {
            Environment::Constant(_) | Environment::Studio { .. } => FRAC_1_4PI,
            Environment::Sky {
                sun_dir,
                sun_angular_radius,
                ..
            } => {
                let cone_solid = 2.0 * std::f32::consts::PI * (1.0 - sun_angular_radius.cos());
                let in_cone = dir.normalize().dot(*sun_dir) >= sun_angular_radius.cos();
                if in_cone {
                    P_SUN / cone_solid + (1.0 - P_SUN) * FRAC_1_4PI
                } else {
                    (1.0 - P_SUN) * FRAC_1_4PI
                }
            }
        }
    }

    /// Sample a direction from the environment. Returns (dir, radiance, pdf).
    #[inline]
    pub fn sample(&self, rng: &mut Rng) -> (Vec3, Color, f32) {
        match self {
            Environment::Constant(c) => (rng.unit_vector(), *c, FRAC_1_4PI),
            Environment::Studio { .. } => {
                let d = rng.unit_vector();
                (d, self.radiance(d), FRAC_1_4PI)
            }
            Environment::Sky {
                sun_dir,
                sun_angular_radius,
                ..
            } => {
                let (u1, u2) = rng.f32_2();
                if u1 < P_SUN {
                    // Sample the sun cone.
                    let cos_alpha = sun_angular_radius.cos();
                    let cos_a = 1.0 - (u1 / P_SUN) * (1.0 - cos_alpha);
                    let sin_a = (1.0 - cos_a * cos_a).max(0.0).sqrt();
                    let phi = 2.0 * std::f32::consts::PI * u2;
                    let (t, b, n) = sun_dir.build_basis();
                    let d = t * (sin_a * phi.cos()) + b * (sin_a * phi.sin()) + n * cos_a;
                    let cone_solid = 2.0 * std::f32::consts::PI * (1.0 - cos_alpha);
                    let pdf = P_SUN / cone_solid + (1.0 - P_SUN) * FRAC_1_4PI;
                    (d, self.radiance(d), pdf)
                } else {
                    let d = rng.unit_vector();
                    (d, self.radiance(d), self.pdf(d))
                }
            }
        }
    }
}
