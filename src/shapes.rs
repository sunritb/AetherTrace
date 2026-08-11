use crate::aabb::Aabb;
use crate::ray::Ray;
use crate::rng::Rng;
use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct ShapeHit {
    pub t: f32,
    pub pos: Vec3,
    /// Shading normal, oriented to face the incoming ray.
    pub n: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
    pub n0: Vec3,
    pub n1: Vec3,
    pub n2: Vec3,
    e1: Vec3,
    e2: Vec3,
    face_n: Vec3,
    area: f32,
}

impl Triangle {
    #[inline(always)]
    pub fn flat(v0: Vec3, v1: Vec3, v2: Vec3) -> Self {
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        let face_n = e1.cross(e2).normalize();
        let area = 0.5 * e1.cross(e2).length();
        Self {
            v0,
            v1,
            v2,
            n0: face_n,
            n1: face_n,
            n2: face_n,
            e1,
            e2,
            face_n,
            area,
        }
    }

    #[inline(always)]
    pub fn area(&self) -> f32 {
        self.area
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Sphere(Sphere),
    Triangle(Triangle),
}

impl Shape {
    #[inline(always)]
    pub fn aabb(&self) -> Aabb {
        match self {
            Shape::Sphere(s) => Aabb::new(
                s.center - Vec3::new(s.radius, s.radius, s.radius),
                s.center + Vec3::new(s.radius, s.radius, s.radius),
            ),
            Shape::Triangle(t) => {
                Aabb::from_pts(t.v0.min(t.v1).min(t.v2), t.v0.max(t.v1).max(t.v2))
            }
        }
    }

    #[inline(always)]
    pub fn area(&self) -> f32 {
        match self {
            Shape::Sphere(s) => 4.0 * std::f32::consts::PI * s.radius * s.radius,
            Shape::Triangle(t) => t.area(),
        }
    }

    /// Uniform sample of surface point + geometric normal.
    #[inline(always)]
    pub fn sample(&self, rng: &mut Rng) -> (Vec3, Vec3) {
        match self {
            Shape::Sphere(s) => {
                let n = rng.unit_vector();
                (s.center + n * s.radius, n)
            }
            Shape::Triangle(t) => {
                let (u, v) = rng.f32_2();
                let r1 = u.sqrt();
                let a = 1.0 - r1;
                let b = r1 * v;
                let pos = t.v0 * a + t.v1 * b + t.v2 * (1.0 - a - b);
                let n = (t.n0 * a + t.n1 * b + t.n2 * (1.0 - a - b)).normalize();
                (pos, n)
            }
        }
    }

    #[inline(always)]
    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<ShapeHit> {
        match self {
            Shape::Sphere(s) => {
                let oc = ray.origin - s.center;
                let a = ray.dir.length_squared();
                let half_b = oc.dot(ray.dir);
                let c = oc.length_squared() - s.radius * s.radius;
                let disc = half_b * half_b - a * c;
                if disc < 0.0 {
                    return None;
                }
                let sqrt_d = disc.sqrt();
                let mut t = (-half_b - sqrt_d) / a;
                if t < t_min || t > t_max {
                    t = (-half_b + sqrt_d) / a;
                    if t < t_min || t > t_max {
                        return None;
                    }
                }
                let pos = ray.at(t);
                let g_n = (pos - s.center) / s.radius;
                let n = if g_n.dot(-ray.dir) > 0.0 { g_n } else { -g_n };
                Some(ShapeHit { t, pos, n })
            }
            Shape::Triangle(t) => {
                let pvec = ray.dir.cross(t.e2);
                let det = t.e1.dot(pvec);
                if det.abs() < 1e-9 {
                    return None;
                }
                let inv_det = 1.0 / det;
                let tvec = ray.origin - t.v0;
                let u = tvec.dot(pvec) * inv_det;
                if !(0.0..=1.0).contains(&u) {
                    return None;
                }
                let qvec = tvec.cross(t.e1);
                let v = ray.dir.dot(qvec) * inv_det;
                if v < 0.0 || u + v > 1.0 {
                    return None;
                }
                let t_hit = t.e2.dot(qvec) * inv_det;
                if t_hit < t_min || t_hit > t_max {
                    return None;
                }
                let pos = ray.at(t_hit);
                let w = 1.0 - u - v;
                let mut n = t.n0 * w + t.n1 * u + t.n2 * v;
                if n.length_squared() < 1e-12 {
                    n = t.face_n;
                }
                n = n.normalize();
                let n = if n.dot(-ray.dir) > 0.0 { n } else { -n };
                Some(ShapeHit { t: t_hit, pos, n })
            }
        }
    }
}
