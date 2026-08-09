use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray {
    #[inline(always)]
    pub fn new(origin: Vec3, dir: Vec3) -> Self {
        Self { origin, dir }
    }

    #[inline(always)]
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.dir * t
    }
}
