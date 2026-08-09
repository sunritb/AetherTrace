use crate::camera::Camera;
use crate::materials::Material;
use crate::scene::Scene;
use crate::sky::Environment;
use crate::vec3::{Color, Vec3};

pub enum SceneName {
    Cornell,
    Spheres,
    Studio,
    Sunset,
}

pub struct BuiltScene {
    pub scene: Scene,
    pub camera: Camera,
}

fn add_quad(scene: &mut Scene, p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, material: u32) {
    scene.add_quad(p0, p1, p2, p3, material);
}

fn add_box(scene: &mut Scene, cx: f32, cy: f32, cz: f32, sx: f32, sy: f32, sz: f32, mat: u32) {
    let mn = Vec3::new(cx - sx * 0.5, cy - sy * 0.5, cz - sz * 0.5);
    let mx = Vec3::new(cx + sx * 0.5, cy + sy * 0.5, cz + sz * 0.5);
    add_quad(
        scene,
        Vec3::new(mx.x, mn.y, mn.z),
        Vec3::new(mx.x, mx.y, mn.z),
        Vec3::new(mx.x, mx.y, mx.z),
        Vec3::new(mx.x, mn.y, mx.z),
        mat,
    );
    add_quad(
        scene,
        Vec3::new(mn.x, mn.y, mx.z),
        Vec3::new(mn.x, mx.y, mx.z),
        Vec3::new(mn.x, mx.y, mn.z),
        Vec3::new(mn.x, mn.y, mn.z),
        mat,
    );
    add_quad(
        scene,
        Vec3::new(mn.x, mx.y, mn.z),
        Vec3::new(mn.x, mx.y, mx.z),
        Vec3::new(mx.x, mx.y, mx.z),
        Vec3::new(mx.x, mx.y, mn.z),
        mat,
    );
    add_quad(
        scene,
        Vec3::new(mn.x, mn.y, mn.z),
        Vec3::new(mx.x, mn.y, mn.z),
        Vec3::new(mx.x, mn.y, mx.z),
        Vec3::new(mn.x, mn.y, mx.z),
        mat,
    );
    add_quad(
        scene,
        Vec3::new(mx.x, mn.y, mx.z),
        Vec3::new(mx.x, mx.y, mx.z),
        Vec3::new(mn.x, mx.y, mx.z),
        Vec3::new(mn.x, mn.y, mx.z),
        mat,
    );
    add_quad(
        scene,
        Vec3::new(mn.x, mn.y, mn.z),
        Vec3::new(mn.x, mx.y, mn.z),
        Vec3::new(mx.x, mx.y, mn.z),
        Vec3::new(mx.x, mn.y, mn.z),
        mat,
    );
}

fn add_box_rot_y(scene: &mut Scene, cx: f32, cy: f32, cz: f32, sx: f32, sy: f32, sz: f32, deg: f32, mat: u32) {
    let (sin, cos) = deg.to_radians().sin_cos();
    let rot = |p: Vec3| Vec3::new(p.x * cos + p.z * sin, p.y, -p.x * sin + p.z * cos);
    let tr = |p: Vec3| rot(p) + Vec3::new(cx, cy, cz);
    let h = Vec3::new(sx * 0.5, sy * 0.5, sz * 0.5);
    let mn = -h;
    let mx = h;

    let a = |x: f32, y: f32, z: f32| tr(Vec3::new(x, y, z));
    add_quad(scene, a(mx.x, mn.y, mn.z), a(mx.x, mx.y, mn.z), a(mx.x, mx.y, mx.z), a(mx.x, mn.y, mx.z), mat);
    add_quad(scene, a(mn.x, mn.y, mx.z), a(mn.x, mx.y, mx.z), a(mn.x, mx.y, mn.z), a(mn.x, mn.y, mn.z), mat);
    add_quad(scene, a(mn.x, mx.y, mn.z), a(mn.x, mx.y, mx.z), a(mx.x, mx.y, mx.z), a(mx.x, mx.y, mn.z), mat);
    add_quad(scene, a(mn.x, mn.y, mn.z), a(mx.x, mn.y, mn.z), a(mx.x, mn.y, mx.z), a(mn.x, mn.y, mx.z), mat);
    add_quad(scene, a(mx.x, mn.y, mx.z), a(mx.x, mx.y, mx.z), a(mn.x, mx.y, mx.z), a(mn.x, mn.y, mx.z), mat);
    add_quad(scene, a(mn.x, mn.y, mn.z), a(mn.x, mx.y, mn.z), a(mx.x, mx.y, mn.z), a(mx.x, mn.y, mn.z), mat);
}

pub fn build(name: &SceneName, aspect: f32) -> BuiltScene {
    match name {
        SceneName::Cornell => build_cornell(aspect),
        SceneName::Spheres => build_spheres(aspect),
        SceneName::Studio => build_studio(aspect),
        SceneName::Sunset => build_sunset(aspect),
    }
}

fn build_cornell(aspect: f32) -> BuiltScene {
    let mut scene = Scene::new(Environment::Constant(Color::zero()));
    let white = scene.add_material(Material::lambert(Color::new(0.75, 0.75, 0.75)));
    let red = scene.add_material(Material::lambert(Color::new(0.85, 0.2, 0.2)));
    let green = scene.add_material(Material::lambert(Color::new(0.2, 0.85, 0.2)));
    let gold = scene.add_material(Material::pbr(Color::new(1.0, 0.71, 0.29), 1.0, 0.18));
    let glass = scene.add_material(Material::glass(1.5));
    let light = scene.add_material(Material::emissive(Color::new(14.0, 13.0, 11.0)));

    // Room shell: x in [-1,1], y in [-1,1], z in [-2,1] (front open).
    // Floor (normal +y).
    scene.add_quad(
        Vec3::new(-1.0, -1.0, 1.0),
        Vec3::new(1.0, -1.0, 1.0),
        Vec3::new(1.0, -1.0, -2.0),
        Vec3::new(-1.0, -1.0, -2.0),
        white,
    );
    // Ceiling (normal -y).
    scene.add_quad(
        Vec3::new(-1.0, 1.0, -2.0),
        Vec3::new(1.0, 1.0, -2.0),
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(-1.0, 1.0, 1.0),
        white,
    );
    // Back wall (normal +z).
    scene.add_quad(
        Vec3::new(-1.0, -1.0, -2.0),
        Vec3::new(1.0, -1.0, -2.0),
        Vec3::new(1.0, 1.0, -2.0),
        Vec3::new(-1.0, 1.0, -2.0),
        white,
    );
    // Left wall (normal +x).
    scene.add_quad(
        Vec3::new(-1.0, -1.0, -2.0),
        Vec3::new(-1.0, 1.0, -2.0),
        Vec3::new(-1.0, 1.0, 1.0),
        Vec3::new(-1.0, -1.0, 1.0),
        red,
    );
    // Right wall (normal -x).
    scene.add_quad(
        Vec3::new(1.0, -1.0, 1.0),
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(1.0, 1.0, -2.0),
        Vec3::new(1.0, -1.0, -2.0),
        green,
    );

    // Ceiling light panel (emissive).
    scene.add_quad(
        Vec3::new(-0.65, 0.995, -0.65),
        Vec3::new(0.65, 0.995, -0.65),
        Vec3::new(0.65, 0.995, 0.65),
        Vec3::new(-0.65, 0.995, 0.65),
        light,
    );

    // Objects.
    add_box(&mut scene, -0.45, -0.5, 0.1, 0.55, 1.0, 0.55, white);
    add_box_rot_y(&mut scene, 0.42, -0.3, -0.15, 0.55, 0.6, 0.55, 24.0, gold);
    scene.add_sphere(Vec3::new(0.12, -0.65, 0.05), 0.35, glass);

    scene.finalize();
    let camera = Camera::new(
        Vec3::new(0.0, 0.0, 1.6),
        Vec3::new(0.0, 0.0, -0.6),
        Vec3::new(0.0, 1.0, 0.0),
        42.0,
        aspect,
        0.0,
        2.2,
    );
    BuiltScene { scene, camera }
}

fn build_spheres(aspect: f32) -> BuiltScene {
    let sun = Vec3::new(-0.55, 0.4, -0.75).normalize();
    let mut scene = Scene::new(Environment::sky(sun, Color::new(26.0, 24.0, 20.0)));

    let checker = scene.add_material(Material::checker(2.5, Color::new(0.9, 0.9, 0.9), Color::new(0.12, 0.12, 0.12)));
    let blue = scene.add_material(Material::lambert(Color::new(0.12, 0.35, 0.7)));
    let glass = scene.add_material(Material::glass(1.5));
    let gold = scene.add_material(Material::pbr(Color::new(1.0, 0.72, 0.25), 1.0, 0.15));
    let chrome = scene.add_material(Material::pbr(Color::new(0.92, 0.95, 1.0), 1.0, 0.04));
    let plastic = scene.add_material(Material::pbr(Color::new(0.9, 0.15, 0.2), 0.0, 0.4));

    scene.add_sphere(Vec3::new(0.0, -100.5, -1.0), 100.0, checker);
    scene.add_sphere(Vec3::new(-1.05, 0.0, -1.0), 0.5, glass);
    scene.add_sphere(Vec3::new(-0.35, 0.0, -0.6), 0.5, blue);
    scene.add_sphere(Vec3::new(0.6, 0.0, -1.0), 0.5, gold);
    scene.add_sphere(Vec3::new(1.4, 0.0, -1.5), 0.5, chrome);
    scene.add_sphere(Vec3::new(0.15, 0.55, -1.25), 0.35, plastic);

    scene.finalize();
    let camera = Camera::new(
        Vec3::new(0.0, 0.15, 1.4),
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(0.0, 1.0, 0.0),
        40.0,
        aspect,
        0.0,
        2.4,
    );
    BuiltScene { scene, camera }
}

fn build_studio(aspect: f32) -> BuiltScene {
    let mut scene = Scene::new(Environment::Studio {
        top: Color::new(0.04, 0.04, 0.05),
        bottom: Color::new(0.008, 0.008, 0.009),
        horizon: Color::new(0.02, 0.02, 0.024),
    });

    let floor = scene.add_material(Material::pbr(Color::new(0.03, 0.03, 0.034), 0.0, 0.05));
    let backdrop = scene.add_material(Material::pbr(Color::new(0.12, 0.12, 0.14), 0.0, 0.75));
    let gold = scene.add_material(Material::pbr(Color::new(1.0, 0.76, 0.33), 1.0, 0.12));
    let glass = scene.add_material(Material::glass(1.5));
    let red = scene.add_material(Material::lambert(Color::new(0.8, 0.18, 0.2)));
    let black_plastic = scene.add_material(Material::pbr(Color::new(0.02, 0.02, 0.02), 0.0, 0.3));
    let soft = scene.add_material(Material::emissive(Color::new(11.0, 10.5, 9.5)));
    let rim = scene.add_material(Material::emissive(Color::new(2.2, 3.5, 5.0)));

    // Floor.
    scene.add_quad(
        Vec3::new(-3.0, 0.0, 3.0),
        Vec3::new(-3.0, 0.0, -3.0),
        Vec3::new(3.0, 0.0, -3.0),
        Vec3::new(3.0, 0.0, 3.0),
        floor,
    );
    // Backdrop.
    scene.add_quad(
        Vec3::new(-3.0, 0.0, -2.4),
        Vec3::new(-3.0, 3.0, -2.4),
        Vec3::new(3.0, 3.0, -2.4),
        Vec3::new(3.0, 0.0, -2.4),
        backdrop,
    );
    // Key softbox (faces down, normal -y).
    scene.add_quad(
        Vec3::new(-1.1, 2.2, -0.6),
        Vec3::new(1.1, 2.2, -0.6),
        Vec3::new(1.1, 2.2, 0.6),
        Vec3::new(-1.1, 2.2, 0.6),
        soft,
    );
    // Cool rim light behind.
    scene.add_quad(
        Vec3::new(1.8, 1.2, -1.9),
        Vec3::new(1.8, 2.4, -1.9),
        Vec3::new(2.4, 2.4, -1.9),
        Vec3::new(2.4, 1.2, -1.9),
        rim,
    );
    scene.add_quad(
        Vec3::new(-2.4, 1.2, -1.9),
        Vec3::new(-1.8, 1.2, -1.9),
        Vec3::new(-1.8, 2.4, -1.9),
        Vec3::new(-2.4, 2.4, -1.9),
        rim,
    );

    scene.add_sphere(Vec3::new(-0.75, 0.5, 0.0), 0.5, glass);
    scene.add_sphere(Vec3::new(0.0, 0.5, 0.0), 0.5, gold);
    scene.add_sphere(Vec3::new(0.75, 0.5, 0.0), 0.5, red);
    add_box(&mut scene, 0.0, 0.25, -0.9, 0.6, 0.5, 0.6, black_plastic);

    scene.finalize();
    let camera = Camera::new(
        Vec3::new(0.0, 0.55, 2.9),
        Vec3::new(0.0, 0.42, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        38.0,
        aspect,
        0.0,
        2.9,
    );
    BuiltScene { scene, camera }
}

fn build_sunset(aspect: f32) -> BuiltScene {
    let sun = Vec3::new(-0.35, 0.12, 0.7).normalize();
    let mut scene = Scene::new(Environment::sky(sun, Color::new(60.0, 34.0, 12.0)));

    let ground = scene.add_material(Material::lambert(Color::new(0.16, 0.14, 0.11)));
    let rock = scene.add_material(Material::pbr(Color::new(0.32, 0.3, 0.28), 0.0, 0.85));
    let glass = scene.add_material(Material::glass(1.52));
    let dark_metal = scene.add_material(Material::pbr(Color::new(0.05, 0.05, 0.05), 1.0, 0.25));
    let bright = scene.add_material(Material::pbr(Color::new(0.9, 0.2, 0.15), 0.0, 0.3));

    // Huge ground plane.
    scene.add_quad(
        Vec3::new(-50.0, -0.01, -50.0),
        Vec3::new(-50.0, -0.01, 50.0),
        Vec3::new(50.0, -0.01, 50.0),
        Vec3::new(50.0, -0.01, -50.0),
        ground,
    );

    // Some rocks and objects.
    scene.add_sphere(Vec3::new(-1.4, 0.2, 0.4), 0.2, rock);
    scene.add_sphere(Vec3::new(1.6, 0.3, 0.2), 0.3, rock);
    scene.add_sphere(Vec3::new(0.6, 0.7, 0.0), 0.7, glass);
    add_box(&mut scene, -1.1, 0.55, -0.6, 1.0, 1.1, 1.0, dark_metal);
    scene.add_sphere(Vec3::new(1.9, 0.5, -0.8), 0.5, bright);

    scene.finalize();
    let camera = Camera::new(
        Vec3::new(0.0, 1.0, 3.4),
        Vec3::new(0.0, 0.5, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        45.0,
        aspect,
        0.0,
        3.4,
    );
    BuiltScene { scene, camera }
}

pub fn from_name(s: &str) -> Option<SceneName> {
    match s.to_ascii_lowercase().as_str() {
        "cornell" | "box" => Some(SceneName::Cornell),
        "spheres" | "rtiow" | "balls" => Some(SceneName::Spheres),
        "studio" => Some(SceneName::Studio),
        "sunset" => Some(SceneName::Sunset),
        _ => None,
    }
}
