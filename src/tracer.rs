use crate::camera::Camera;
use crate::ray::Ray;
use crate::rng::Rng;
use crate::scene::Scene;
use crate::vec3::Color;

pub const EPS: f32 = 1e-3;
const T_MAX: f32 = 1e9;

/// Render a single sample for pixel coordinates (x, y) in [0,1].
#[inline(always)]
pub fn trace_pixel(
    scene: &Scene,
    camera: &Camera,
    rng: &mut Rng,
    x: f32,
    y: f32,
    max_depth: usize,
) -> Color {
    let ray = camera.generate_ray(rng, x, y);
    trace_ray_depth(scene, &ray, rng, max_depth)
}

#[inline(always)]
fn trace_ray_depth(scene: &Scene, ray: &Ray, rng: &mut Rng, max_depth: usize) -> Color {
    let mut color = Color::zero();
    let mut throughput = Color::one();
    let mut prev_pdf = f32::INFINITY; // camera ray is a delta (no MIS)
    let mut depth = 0;

    let mut cur_ray = *ray;

    loop {
        if let Some((hit, shape_idx)) = scene.bvh.intersect(&scene.shapes, &cur_ray, EPS, T_MAX) {
            let mat = scene.material(shape_idx);

            if mat.is_emissive() {
                if depth == 0 || prev_pdf.is_infinite() {
                    // Camera ray, or a delta (dielectric) path: no MIS.
                    color += throughput * mat.emission;
                } else {
                    // BSDF-sampled ray hit a light: add MIS-weighted emission so
                    // specular highlights of area lights aren't lost.
                    let pdf_light = match scene.lights.iter().find(|l| l.shape == shape_idx) {
                        Some(l) => {
                            let area = scene.shapes[l.shape as usize].area();
                            let n_lights = scene.lights.len() as f32;
                            let dist = (cur_ray.origin - hit.pos).length();
                            let cos = hit.n.dot(-cur_ray.dir).abs().max(1e-6);
                            (1.0 / (area * n_lights)) * (dist * dist / cos)
                        }
                        None => f32::INFINITY,
                    };
                    color += throughput * mat.emission * (prev_pdf / (prev_pdf + pdf_light));
                }
                return color;
            }

            let wo = -cur_ray.dir; // outgoing direction (away from surface)
            let n = hit.n;
            let pos = hit.pos;

            // --- Direct lighting: sample area lights (NEE) ---
            if let Some(light) = scene.sample_light(rng) {
                let to_light = light.pos - pos;
                let dist2 = to_light.length_squared();
                let dist = dist2.sqrt();
                let wi = to_light / dist;
                let cos_light = light.n.dot(-wi);
                let cos_surf = n.dot(wi);
                if cos_light > 1e-4 && cos_surf > 1e-4 {
                    let shadow = Ray::new(pos, wi);
                    if !scene.bvh.occluded(&scene.shapes, &shadow, EPS, dist - EPS) {
                        let f = mat.eval(wo, wi, n, pos);
                        let pdf_light = light.pdf_area * dist2 / cos_light;
                        let pdf_bsdf = mat.pdf(wo, wi, n);
                        if !f.approx_zero() && pdf_light > 1e-12 {
                            let w = pdf_light / (pdf_light + pdf_bsdf);
                            color += throughput * f * light.emission * (cos_surf / pdf_light) * w;
                        }
                    }
                }
            }

            // --- Direct lighting: sample the environment (MIS) ---
            let (env_dir, env_rad, pdf_env) = scene.env.sample(rng);
            let cos_env = n.dot(env_dir);
            if cos_env > 1e-4 && !env_rad.approx_zero() {
                let f = mat.eval(wo, env_dir, n, pos);
                if !f.approx_zero() {
                    let pdf_bsdf = mat.pdf(wo, env_dir, n);
                    let w = pdf_env / (pdf_env + pdf_bsdf);
                    color += throughput * f * env_rad * (cos_env / pdf_env) * w;
                }
            }

            // --- Russian roulette ---
            depth += 1;
            if depth > max_depth {
                return color;
            }
            let surv = throughput.max_component().min(0.95);
            if surv < 1.0 {
                if rng.f32() > surv {
                    return color;
                }
                throughput = throughput / surv;
            }

            // --- Scatter to the next event ---
            let (wi, weight, pdf) = mat.scatter(rng, wo, n, pos);
            if weight.approx_zero() || pdf <= 0.0 {
                return color;
            }
            throughput = throughput * weight;
            prev_pdf = pdf;
            cur_ray = Ray::new(pos + wi * EPS, wi);
        } else {
            // Ray missed — accumulate environment radiance with MIS weighting.
            let li = scene.env.radiance(cur_ray.dir);
            if !li.approx_zero() {
                let w = if depth == 0 || prev_pdf.is_infinite() {
                    1.0 // delta source (camera or glass): no MIS
                } else {
                    let pdf_env = scene.env.pdf(cur_ray.dir);
                    prev_pdf / (prev_pdf + pdf_env)
                };
                color += throughput * li * w;
            }
            return color;
        }
    }
}
