use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    #[inline(always)]
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    #[inline(always)]
    pub fn empty() -> Self {
        Self {
            min: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    #[inline(always)]
    pub fn from_pts(a: Vec3, b: Vec3) -> Self {
        Self {
            min: a.min(b),
            max: a.max(b),
        }
    }

    #[inline(always)]
    pub fn grow_aabb(&mut self, o: &Aabb) {
        self.min = self.min.min(o.min);
        self.max = self.max.max(o.max);
    }

    #[inline(always)]
    pub fn centroid(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    #[inline(always)]
    pub fn surface_area(&self) -> f32 {
        let e = self.max - self.min;
        2.0 * (e.x * e.y + e.y * e.z + e.z * e.x)
    }

    /// Slab-method ray intersection, returns (t_min, t_max).
    #[inline(always)]
    pub fn hit(&self, orig: Vec3, inv_dir: Vec3) -> Option<(f32, f32)> {
        let mut tmin: f32 = 0.0;
        let mut tmax: f32 = f32::INFINITY;
        // unrolled slab tests
        for i in 0..3 {
            let o = orig.component(i);
            let inv = inv_dir.component(i);
            let mut t0 = (self.min.component(i) - o) * inv;
            let mut t1 = (self.max.component(i) - o) * inv;
            if inv < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }
            tmin = tmin.max(t0);
            tmax = tmax.min(t1);
            if tmax < tmin {
                return None;
            }
        }
        Some((tmin, tmax))
    }
}
