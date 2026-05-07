use glam::Vec3;

use bvh::BVH;
use camera::Camera;
use light_source::LightSource;
use materials::{Diffuse, Light, Material, Reflective};
use plane::{Axis, Bounds2D, Plane};
use scene::Scene;
use sphere::Sphere;
use texture::SolidColor;
use transformations::TransformedMesh;
use world::World;
use mat;

/// Veach Multiple Importance Sampling test scene.
///
/// Four sphere lights of equal total power (4πr²×I = const) hang at Y=3.0.
/// Four reflective plates rise progressively in height and tilt — the front
/// plate is nearly horizontal (gentle tilt, low roughness) and each successive
/// plate is higher and more tilted (higher roughness). This demonstrates:
///   - Smooth/shallow plates: BSDF sampling is best (narrow specular lobe)
///   - Rough/steep plates: NEE / light sampling is best (wide lobe)
pub fn veach_mis_scene(width: usize, height: usize) -> Scene {
    let origin = Vec3::new(0.0, 1.5, -2.5);
    let lookat = Vec3::new(0.0, 0.8,  3.5);
    let view = Vec3::new(0.0, 1.0,  0.0);
    let fov = 45.0;
    let aspect_ratio = width as f32 / height as f32;
    let aperture = 0.0;
    let focus_distance = 10.0;
    let start_time = 0.0;
    let end_time = 1.0;

    let camera = Camera::new(origin, lookat, view, fov, aspect_ratio,
        aperture, focus_distance, start_time, end_time);

    let mut world = World::new();
    let mut materials: Vec<Material> = Vec::new();

    // ── Room ────────────────────────────────────────────────────────────────
    let grey = mat!(materials, Diffuse::new(SolidColor::new(0.9, 0.9, 0.9).into(), 0.0));
    world.add(Plane::new(Axis::XZ, Bounds2D::new(-4.0..4.0, -0.5..8.0), 0.0, grey).into_geometry());
    world.add(Plane::new(Axis::XZ, Bounds2D::new(-4.0..4.0, -0.5..8.0), 4.0, grey).into_reversed());
    world.add(Plane::new(Axis::XY, Bounds2D::new(-4.0..4.0,  0.0..4.0), 8.0, grey).into_reversed());
    world.add(Plane::new(Axis::YZ, Bounds2D::new( 0.0..4.0, -0.5..8.0),-4.0, grey).into_geometry());
    world.add(Plane::new(Axis::YZ, Bounds2D::new( 0.0..4.0, -0.5..8.0), 4.0, grey).into_reversed());

    // ── Reflective plates ───────────────────────────────────────────────────
    
    let silver = Vec3::new(1.0, 0.85, 0.57);
    let plates = [
        (2.0f32, 0.2f32, -10.0f32, 0.16f32),
        (3.0f32, 0.6f32, -20.0f32, 0.08f32),
        (4.0f32, 1.2f32, -30.0f32, 0.04f32),
        (5.0f32, 2.0f32, -55.0f32, 0.00f32),
    ];
    for (z, y, tilt, fuzz) in plates {
        let mat_id = mat!(materials, Reflective::new(silver, fuzz));
        let rot = Vec3::new(tilt, 0.0, 0.0);
        let base = Plane::new(Axis::XZ, Bounds2D::new(-2.5..2.5, -0.5..0.5), 0.0, mat_id)
            .into_geometry();
        world.add(TransformedMesh::new(Vec3::new(0.0, y, z), rot, 1.0, base).into());
    }

    // ── Four sphere lights of equal total luminous flux ──────────────────────
    // Tiny (x=3): highest radiance, smallest solid angle — BSDF sampling works poorly
    // Large (x=-3): lowest radiance, largest solid angle — NEE sampling works well
    let light_y = 3.0_f32;
    let light_z = 5.0_f32;

    let sphere_lights: [(f32, f32, f32); 4] = [
        ( 2.0, 0.025, 100.0),
        ( 0.75, 0.05,  25.0),
        (-0.75, 0.20,   6.25),
        (-2.0, 0.50, 4.0)
    ];
    for (x, r, intensity) in sphere_lights {
        let mat = mat!(materials, Light::new(SolidColor::new(intensity, intensity, intensity).into()));
        world.add(Sphere::new(
            Vec3::new(x, light_y, light_z),
            Vec3::new(x, light_y, light_z),
            r, mat, 0.0, 1.0,
        ).into());
    }

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    // The large sphere is the NEE target: clone it before moving into the world.
    let light_mat = mat!(materials, Light::new(SolidColor::new(0.0, 0.0, 0.0).into()));
    let light_shape = Sphere::new(Vec3::new(2.0, 3.0, 5.0), Vec3::new(2.0, 3.0, 5.0), 4.0, light_mat, 0.0, 1.0);



    Scene::new(
        String::from("Veach MIS"),
        bvh,
        materials,
        camera,
        Some(LightSource::Sphere(light_shape)),
        None,
        false,
    )
}
