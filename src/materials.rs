use crate::rng::Rng;
use crate::sample::{cosine_hemisphere, sample_ggx_reflection};
use crate::vec3::{Color, Vec3, reflect, schlick, schlick_color};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MaterialKind {
    Lambert,
    Pbr,
    Dielectric,
    Emissive,
}

#[derive(Clone, Copy, Debug)]
pub enum Texture {
    Solid,
    Checker { scale: f32, c1: Color, c2: Color },
}

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub kind: MaterialKind,
    pub albedo: Color,
    pub texture: Texture,
    pub metallic: f32,
    pub roughness: f32,
    pub ior: f32,
    pub emission: Color,
}

impl Material {
    #[inline(always)]
    pub fn lambert(albedo: Color) -> Self {
        Self {
            kind: MaterialKind::Lambert,
            albedo,
            texture: Texture::Solid,
            metallic: 0.0,
            roughness: 1.0,
            ior: 1.45,
            emission: Color::zero(),
        }
    }

    #[inline(always)]
    pub fn checker(scale: f32, c1: Color, c2: Color) -> Self {
        let mut m = Self::lambert(Color::one());
        m.texture = Texture::Checker { scale, c1, c2 };
        m
    }

    #[inline(always)]
    pub fn pbr(albedo: Color, metallic: f32, roughness: f32) -> Self {
        Self {
            kind: MaterialKind::Pbr,
            albedo,
            texture: Texture::Solid,
            metallic: metallic.clamp(0.0, 1.0),
            roughness: roughness.clamp(0.02, 1.0),
            ior: 1.45,
            emission: Color::zero(),
        }
    }

    #[inline(always)]
    pub fn glass(ior: f32) -> Self {
        Self {
            kind: MaterialKind::Dielectric,
            albedo: Color::one(),
            texture: Texture::Solid,
            metallic: 0.0,
            roughness: 0.0,
            ior,
            emission: Color::zero(),
        }
    }

    #[inline(always)]
    pub fn emissive(emission: Color) -> Self {
        Self {
            kind: MaterialKind::Emissive,
            albedo: Color::one(),
            texture: Texture::Solid,
            metallic: 0.0,
            roughness: 1.0,
            ior: 1.45,
            emission,
        }
    }

    #[inline(always)]
    pub fn is_emissive(&self) -> bool {
        self.kind == MaterialKind::Emissive
    }

    #[inline(always)]
    pub fn albedo_at(&self, pos: Vec3) -> Color {
        match self.texture {
            Texture::Solid => self.albedo,
            Texture::Checker { scale, c1, c2 } => {
                let s = (pos.x * scale).floor() + (pos.y * scale).floor() + (pos.z * scale).floor();
                if (s as i64).rem_euclid(2) == 0 {
                    c1
                } else {
                    c2
                }
            }
        }
    }

    /// F0 at normal incidence (dielectric + metallic tint).
    #[inline(always)]
    fn f0(&self, albedo: Color) -> Color {
        let c = ((self.ior - 1.0) / (self.ior + 1.0)).powi(2);
        albedo * self.metallic + Color::new(c, c, c) * (1.0 - self.metallic)
    }

    /// Probability of sampling the specular (GGX) lobe.
    #[inline(always)]
    fn spec_prob(&self, albedo: Color) -> f32 {
        let f0 = self.f0(albedo);
        let refl = (f0.x + f0.y + f0.z) / 3.0;
        let base = 0.5 * (1.0 - self.roughness.sqrt()) * (1.0 - self.metallic) + self.metallic;
        (base + refl * 0.25).clamp(0.1, 0.95)
    }

    /// BRDF f(wo, wi); both directions point away from the surface.
    #[inline(always)]
    pub fn eval(&self, wo: Vec3, wi: Vec3, n: Vec3, pos: Vec3) -> Color {
        match self.kind {
            MaterialKind::Lambert | MaterialKind::Emissive => {
                self.albedo_at(pos) * std::f32::consts::FRAC_1_PI
            }
            MaterialKind::Pbr => {
                let albedo = self.albedo_at(pos);
                let ndotv = n.dot(wo).max(1e-6);
                let ndotl = n.dot(wi).max(1e-6);
                let h = (wo + wi).normalize();
                let ndoth = n.dot(h).max(1e-6);
                let vdoth = wo.dot(h).max(1e-6);
                let a2 = (self.roughness * self.roughness).powi(2);
                let f0 = self.f0(albedo);
                let f = schlick_color(vdoth, f0);
                let spec = ggx_spec(ndotv, ndotl, ndoth, vdoth, a2, f);
                let diff = (1.0 - self.metallic)
                    * albedo
                    * std::f32::consts::FRAC_1_PI
                    * (Color::one() - f);
                diff + spec
            }
            MaterialKind::Dielectric => Color::zero(),
        }
    }

    /// PDF of scattering toward `wi`.
    #[inline(always)]
    pub fn pdf(&self, wo: Vec3, wi: Vec3, n: Vec3) -> f32 {
        match self.kind {
            MaterialKind::Lambert | MaterialKind::Emissive => {
                n.dot(wi).max(0.0) * std::f32::consts::FRAC_1_PI
            }
            MaterialKind::Pbr => {
                let ndotl = n.dot(wi);
                if ndotl <= 0.0 {
                    return 0.0;
                }
                let h = (wo + wi).normalize();
                let ndoth = n.dot(h).max(1e-6);
                let vdoth = wo.dot(h).max(1e-6);
                let a2 = (self.roughness * self.roughness).powi(2);
                let pdf_ggx = ggx_ndf(ndoth, a2) * ndoth / (4.0 * vdoth).max(1e-6);
                let pdf_cos = ndotl * std::f32::consts::FRAC_1_PI;
                let p_spec = self.spec_prob(Color::one());
                p_spec * pdf_ggx + (1.0 - p_spec) * pdf_cos
            }
            MaterialKind::Dielectric => 0.0,
        }
    }

    /// Sample scattered direction. Returns (travel_dir, throughput weight, pdf).
    #[inline(always)]
    pub fn scatter(&self, rng: &mut Rng, wo: Vec3, n: Vec3, pos: Vec3) -> (Vec3, Color, f32) {
        match self.kind {
            MaterialKind::Lambert | MaterialKind::Emissive => {
                let wi = cosine_hemisphere(rng, n);
                let pdf = n.dot(wi).max(0.0) * std::f32::consts::FRAC_1_PI;
                let weight = self.albedo_at(pos);
                (wi, weight, pdf)
            }
            MaterialKind::Pbr => {
                let albedo = self.albedo_at(pos);
                let p_spec = self.spec_prob(albedo);
                let a2 = (self.roughness * self.roughness).powi(2);
                let f0 = self.f0(albedo);

                let (wi, f_spec, pdf_spec);
                if rng.f32() < p_spec {
                    let (w, ps) = sample_ggx_reflection(rng, n, self.roughness);
                    let ndotv = n.dot(wo).max(1e-6);
                    let ndotl = n.dot(w).max(1e-6);
                    let h = (wo + w).normalize();
                    let ndoth = n.dot(h).max(1e-6);
                    let vdoth = wo.dot(h).max(1e-6);
                    let f = schlick_color(vdoth, f0);
                    wi = w;
                    f_spec = ggx_spec(ndotv, ndotl, ndoth, vdoth, a2, f);
                    pdf_spec = ps;
                } else {
                    let w = cosine_hemisphere(rng, n);
                    wi = w;
                    f_spec = Color::zero();
                    pdf_spec = 0.0;
                }

                let ndotl = n.dot(wi).max(0.0);
                let h = (wo + wi).normalize();
                let vdoth = wo.dot(h).max(1e-6);
                let f = schlick_color(vdoth, f0);
                let f_diff = (1.0 - self.metallic)
                    * albedo
                    * std::f32::consts::FRAC_1_PI
                    * (Color::one() - f);
                let pdf_cos = ndotl * std::f32::consts::FRAC_1_PI;
                let pdf = p_spec * pdf_spec + (1.0 - p_spec) * pdf_cos;
                let weight = if pdf > 1e-8 {
                    (f_spec + f_diff) * (ndotl / pdf)
                } else {
                    (f_spec + f_diff) * ndotl
                };
                (wi, weight, pdf)
            }
            MaterialKind::Dielectric => {
                // wo points away from surface; incident = -wo.
                let cos_i = wo.dot(n).clamp(-1.0, 1.0);
                let eta = if cos_i > 0.0 {
                    self.ior.recip()
                } else {
                    self.ior
                };
                let f0 = ((self.ior - 1.0) / (self.ior + 1.0)).powi(2);
                let fres = schlick(cos_i.abs(), f0);
                let inc = -wo;
                let ndot_inc = n.dot(inc).clamp(-1.0, 1.0);
                let cos_t2 = 1.0 - eta * eta * (1.0 - ndot_inc * ndot_inc);
                if cos_t2 <= 0.0 {
                    // Total internal reflection.
                    let dir = reflect(inc, n);
                    (dir, Color::one(), f32::INFINITY)
                } else if rng.f32() < fres {
                    let dir = reflect(inc, n);
                    (dir, Color::new(fres, fres, fres) / fres, f32::INFINITY)
                } else {
                    let perp = eta * (inc - n * ndot_inc);
                    let par = -n * cos_t2.sqrt();
                    let dir = (perp + par).normalize();
                    let trans = Color::one() - Color::new(fres, fres, fres);
                    (dir, trans / (1.0 - fres), f32::INFINITY)
                }
            }
        }
    }
}

#[inline(always)]
fn ggx_ndf(ndoth: f32, a2: f32) -> f32 {
    let denom = 1.0 + (a2 - 1.0) * ndoth * ndoth;
    a2 / (std::f32::consts::PI * denom * denom)
}

#[inline(always)]
fn ggx_spec(ndotv: f32, ndotl: f32, ndoth: f32, _vdoth: f32, a2: f32, f: Color) -> Color {
    let g1_v = 2.0 * ndotv / (ndotv + (a2 + (1.0 - a2) * ndotv * ndotv).max(1e-6).sqrt());
    let g1_l = 2.0 * ndotl / (ndotl + (a2 + (1.0 - a2) * ndotl * ndotl).max(1e-6).sqrt());
    let d = ggx_ndf(ndoth, a2);
    let denom = (4.0 * ndotv * ndotl).max(1e-6);
    f * (g1_v * g1_l * d / denom)
}
