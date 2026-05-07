use glam::Vec3;

use bvh::BVH;
use camera::Camera;
use lights::Light;
use materials::{Diffuse, Emissive, Material, Reflective};
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
    let origin = Vec3::new(0.0, 1.2, -3.0);
    let lookat = Vec3::new(0.0, 1.2,  5.0);
    let view = Vec3::new(0.0, 1.0,  0.0);
    let fov = 35.0;
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
    // ── Bright, Oversized Room ──────────────────────────────────────────────
    // let grey = mat!(materials, Diffuse::new(SolidColor::new(0.8, 0.8, 0.8).into(), 0.0));

    // Floor: Extend it wide to hide the side seams
    world.add(Plane::new(Axis::XZ, Bounds2D::new(-20.0..20.0, -5.0..25.0), 0.0, grey).into_geometry());

    // Back Wall: Close enough to be bright, but far enough to not feel like a closet
    world.add(Plane::new(Axis::XY, Bounds2D::new(-20.0..20.0, 0.0..15.0), 12.0, grey).into_reversed());

    // Side Walls: Pushed to X = +/- 20.0. They still bounce light/reflections but aren't visible.
    world.add(Plane::new(Axis::YZ, Bounds2D::new( 0.0..15.0, -5.0..25.0),-20.0, grey).into_geometry());
    world.add(Plane::new(Axis::YZ, Bounds2D::new( 0.0..15.0, -5.0..25.0), 20.0, grey).into_reversed());

    // IMPORTANT: Do NOT add YZ planes (side walls) or the top XZ plane (ceiling).
    // The "Reflection" you need comes from the massive floor and back wall.

    // world.add(Plane::new(Axis::XZ, Bounds2D::new(-4.0..4.0, -0.5..8.0), 0.0, grey).into_geometry()); // floor
    // world.add(Plane::new(Axis::XZ, Bounds2D::new(-4.0..4.0, -0.5..8.0), 4.0, grey).into_reversed()); // ceiling
    // world.add(Plane::new(Axis::XY, Bounds2D::new(-4.0..4.0,  0.0..4.0), 8.0, grey).into_reversed()); // back wall
    // world.add(Plane::new(Axis::YZ, Bounds2D::new( 0.0..4.0, -0.5..8.0),-4.0, grey).into_geometry()); // left wall
    // world.add(Plane::new(Axis::YZ, Bounds2D::new( 0.0..4.0, -0.5..8.0), 4.0, grey).into_reversed()); // right wall

    // ── Reflective plates ───────────────────────────────────────────────────
    
    let silver = Vec3::new(0.75, 0.75, 0.75);
    let plates = [
        (2.0, 0.15, -14.0, 0.16),
        (2.8, 0.4, -23.0, 0.08),
        (3.6, 0.75, -32.0, 0.04),
        (4.4, 1.2, -42.0, 0.02),
        (5.2, 1.9, -52.0, 0.01),
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
    let light_z = 4.75_f32;

    let sphere_lights: [(f32, f32, f32); 3] = [
        ( 2.0, 0.01, 100.0),
        // ( 0.75, 0.05,  25.0),
        (0.0, 0.20,   6.5),
        (-2.0, 0.50, 4.0)
    ];
    for (x, r, intensity) in sphere_lights {
        let mat = mat!(materials, Emissive::new(SolidColor::new(intensity, intensity, intensity).into()));
        world.add(Sphere::new(
            Vec3::new(x, light_y, light_z),
            Vec3::new(x, light_y, light_z),
            r, mat, 0.0, 1.0,
        ).into());
    }

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    let light_sources: Vec<Light> = sphere_lights.iter().map(|&(x, r, intensity)| {
        Light::new(
            Sphere::new(Vec3::new(x, light_y, light_z), Vec3::new(x, light_y, light_z), r, grey, 0.0, 1.0).into(),
            Vec3::splat(intensity),
        )
    }).collect();

    Scene::new(
        String::from("Veach MIS"),
        bvh,
        materials,
        camera,
        light_sources,
        None,
        false,
    )
}
