use aethertrace::rng::Rng;
use aethertrace::scenes::{self, SceneName};
use aethertrace::vec3::Vec3;

fn main() {
    let names = [SceneName::Cornell, SceneName::Spheres, SceneName::Studio, SceneName::Sunset];
    for name in names {
        let b = scenes::build(&name, 1.0);
        let mut rng = Rng::from_u64(42);
        let mut mismatches = 0usize;
        let n = 20000;
        for _ in 0..n {
            let o = Vec3::new(
                rng.f32() * 4.0 - 2.0,
                rng.f32() * 3.0 - 1.5,
                rng.f32() * 4.0 - 2.0,
            );
            let d = Vec3::new(
                rng.f32() * 2.0 - 1.0,
                rng.f32() * 2.0 - 1.0,
                rng.f32() * 2.0 - 1.0,
            ).normalize();
            let ray = aethertrace::ray::Ray::new(o, d);
            let brute: Option<f32> = b.scene.shapes.iter().filter_map(|s| s.hit(&ray, 1e-3, 1e9)).map(|h| h.t).min_by(|a, c| a.partial_cmp(c).unwrap());
            let bvh = b.scene.bvh.intersect(&b.scene.shapes, &ray, 1e-3, 1e9).map(|(h, _)| h.t);
            let same = match (brute, bvh) {
                (None, None) => true,
                (Some(a), Some(c)) => (a - c).abs() < 1e-3,
                _ => false,
            };
            if !same { mismatches += 1; }
            let t_max = 0.7f32;
            let b_occ = b.scene.shapes.iter().any(|s| s.hit(&ray, 1e-3, t_max).is_some());
            let v_occ = b.scene.bvh.occluded(&b.scene.shapes, &ray, 1e-3, t_max);
            if b_occ != v_occ { mismatches += 1; }
        }
        println!("{:?}: {} mismatches / {}", match name {
            SceneName::Cornell => "cornell",
            SceneName::Spheres => "spheres",
            SceneName::Studio => "studio",
            SceneName::Sunset => "sunset",
        }, mismatches, n);
    }
}
