use crate::bvh::Bvh;
use crate::materials::Material;
use crate::rng::Rng;
use crate::shapes::{Shape, Sphere, Triangle};
use crate::sky::Environment;
use crate::vec3::{Color, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct LightRef {
    pub shape: u32,
    pub material: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct LightSample {
    pub pos: Vec3,
    pub n: Vec3,
    pub emission: Color,
    pub pdf_area: f32,
}

pub struct Scene {
    pub shapes: Vec<Shape>,
    pub materials: Vec<Material>,
    /// Parallel to `shapes`: material index per shape.
    pub shape_materials: Vec<u32>,
    pub lights: Vec<LightRef>,
    pub bvh: Bvh,
    pub env: Environment,
}

impl Scene {
    pub fn new(env: Environment) -> Self {
        Self {
            shapes: Vec::new(),
            materials: Vec::new(),
            shape_materials: Vec::new(),
            lights: Vec::new(),
            bvh: Bvh::build(&[]),
            env,
        }
    }

    #[inline(always)]
    pub fn material(&self, shape_index: u32) -> &Material {
        &self.materials[self.shape_materials[shape_index as usize] as usize]
    }

    pub fn add_material(&mut self, m: Material) -> u32 {
        self.materials.push(m);
        (self.materials.len() - 1) as u32
    }

    pub fn add_sphere(&mut self, center: Vec3, radius: f32, material: u32) {
        self.shapes.push(Shape::Sphere(Sphere { center, radius }));
        self.shape_materials.push(material);
        if self.materials[material as usize].is_emissive() {
            self.lights.push(LightRef {
                shape: (self.shapes.len() - 1) as u32,
                material,
            });
        }
    }

    pub fn add_triangle(&mut self, t: Triangle, material: u32) {
        self.shapes.push(Shape::Triangle(t));
        self.shape_materials.push(material);
        if self.materials[material as usize].is_emissive() {
            self.lights.push(LightRef {
                shape: (self.shapes.len() - 1) as u32,
                material,
            });
        }
    }

    /// Two-triangle quad, with an option for which side faces out.
    pub fn add_quad(&mut self, p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, material: u32) {
        self.add_triangle(Triangle::flat(p0, p1, p2), material);
        self.add_triangle(Triangle::flat(p0, p2, p3), material);
    }

    pub fn finalize(&mut self) {
        self.bvh = Bvh::build(&self.shapes);
    }

    /// Uniformly sample one emissive light: returns the sampled point info.
    #[inline]
    pub fn sample_light(&self, rng: &mut Rng) -> Option<LightSample> {
        let n = self.lights.len();
        if n == 0 {
            return None;
        }
        let li = (rng.f32() * n as f32) as usize;
        let light = &self.lights[li.min(n - 1)];
        let shape = self.shapes[light.shape as usize];
        let (pos, n_vec) = shape.sample(rng);
        let area = shape.area();
        let emission = self.materials[light.material as usize].emission;
        Some(LightSample {
            pos,
            n: n_vec,
            emission,
            pdf_area: 1.0 / (area * n as f32),
        })
    }
}
