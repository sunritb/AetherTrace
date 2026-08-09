use crate::ray::Ray;
use crate::rng::Rng;
use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub pos: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub lens_radius: f32,
    pub focal_dist: f32,
    pub half_w: f32,
    pub half_h: f32,
}

impl Camera {
    pub fn new(
        pos: Vec3,
        look_at: Vec3,
        up: Vec3,
        vfov_deg: f32,
        aspect: f32,
        aperture: f32,
        focus_dist: f32,
    ) -> Self {
        let forward = (look_at - pos).normalize();
        let right = forward.cross(up).normalize();
        let up = right.cross(forward).normalize();
        let half_h = (vfov_deg.to_radians() * 0.5).tan();
        let half_w = half_h * aspect;
        Self {
            pos,
            forward,
            right,
            up,
            lens_radius: aperture * 0.5,
            focal_dist: focus_dist,
            half_w,
            half_h,
        }
    }

    /// Generate a ray through normalized pixel (u, v) in [0,1]x[0,1].
    #[inline]
    pub fn generate_ray(&self, rng: &mut Rng, u: f32, v: f32) -> Ray {
        // Point on the focal plane.
        let px = (2.0 * u - 1.0) * self.half_w * self.focal_dist;
        let py = (1.0 - 2.0 * v) * self.half_h * self.focal_dist;
        let focus_pt = self.pos + self.forward * self.focal_dist + self.right * px + self.up * py;

        if self.lens_radius <= 0.0 {
            let dir = (focus_pt - self.pos).normalize();
            Ray::new(self.pos, dir)
        } else {
            let rd = rng.unit_disk() * self.lens_radius;
            let origin = self.pos + self.right * rd.x + self.up * rd.y;
            let dir = (focus_pt - origin).normalize();
            Ray::new(origin, dir)
        }
    }
}
