use crate::rng::Rng;
use crate::vec3::{Vec3, reflect};

/// Cosine-weighted hemisphere direction in the local +Z frame, then
/// transformed to the world frame given `n`.
#[inline(always)]
pub fn cosine_hemisphere(rng: &mut Rng, n: Vec3) -> Vec3 {
    let (u, v) = rng.f32_2();
    let r = u.sqrt();
    let phi = 2.0 * std::f32::consts::PI * v;
    let x = r * phi.cos();
    let y = r * phi.sin();
    let z = (1.0 - u).max(0.0).sqrt();
    let local = Vec3::new(x, y, z);
    let (t, b, nz) = n.build_basis();
    t * local.x + b * local.y + nz * local.z
}

/// Sample the GGX (Trowbridge–Reitz) NDF in the local frame with `cos_theta`.
/// Returns (direction_in_local_z_up, pdf_over_solid_angle).
#[inline(always)]
pub fn sample_ggx(rng: &mut Rng, roughness: f32) -> (Vec3, f32) {
    let (u, v) = rng.f32_2();
    let a = roughness * roughness;
    let a2 = a * a;
    let phi = 2.0 * std::f32::consts::PI * v;
    let cos_theta = ((1.0 - u) / (1.0 + (a2 - 1.0) * u)).max(0.0).min(1.0).sqrt();
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    let dir = Vec3::new(
        sin_theta * phi.cos(),
        sin_theta * phi.sin(),
        cos_theta,
    );
    // pdf of the half vector
    let d = a2 / (std::f32::consts::PI * (1.0 + (a2 - 1.0) * cos_theta * cos_theta).powi(2));
    let pdf = d * cos_theta;
    (dir, pdf)
}

/// Reflection direction sampled around `n` with GGX, returns (dir, pdf).
#[inline(always)]
pub fn sample_ggx_reflection(rng: &mut Rng, n: Vec3, roughness: f32) -> (Vec3, f32) {
    let (half_local, pdf_h) = sample_ggx(rng, roughness);
    let (t, b, nz) = n.build_basis();
    let h = (t * half_local.x + b * half_local.y + nz * half_local.z).normalize();
    let dir = reflect(-Vec3::new(0.0, 0.0, 1.0), h);
    // pdf w.r.t. solid angle around outgoing direction: pdf_h / (4 dot(v,h))
    let vdh = half_local.z;
    let pdf = pdf_h / (4.0 * vdh).max(1e-6);
    (dir, pdf)
}
