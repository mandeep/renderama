use std::f32;

use glam::Vec3A;

use bvh::BVH;
use camera::Camera;
use materials::{Diffuse, Reflective, Refractive, Material};
use scene::Scene;
use sphere::Sphere;
use texture::{EnvironmentMap, SolidColor, Texture};
use world::World;
use mat;

pub fn random_spheres_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(13.0, 2.0, 3.0);
    let lookat = Vec3A::new(0.0, 0.0, 0.0);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let fov = 20.0;
    let aspect_ratio = (width.unwrap_or(2048) as f32, height.unwrap_or(1024) as f32);
    let aperture = 0.1;
    let focus_distance = 10.0;

    let camera = Camera::new(origin,
                             lookat,
                             view,
                             fov,
                             aspect_ratio,
                             aperture,
                             focus_distance);

    let mut world = World::new();
    let mut materials: Vec<Material> = Vec::new();

    let environment = EnvironmentMap::new("docs/textures/pure_sky_qwantani.exr").into();

    let floor_idx = mat!(materials, Diffuse::new(SolidColor::new(0.5, 0.5, 0.5).into(), 0.0));

    world.add(Sphere::new(Vec3A::new(0.0, -1000.0, 0.0),
                          1000.0,
                          floor_idx).into());

    let red_center = Vec3A::new(-2.0, 1.0, 0.0);
    let glass_center = Vec3A::new(0.0, 1.0, 0.0);
    let metal_center = Vec3A::new(2.0, 1.0, 0.0);

    let mut placed: Vec<(Vec3A, f32)> = vec![
        (red_center, 1.0),
        (glass_center, 1.0),
        (metal_center, 1.0),
    ];

    for a in -11..11 {
        for b in -11..11 {
            let material = rand::random::<f32>();
            let center: Vec3A = Vec3A::new(
                a as f32 + 0.9 * rand::random::<f32>(),
                0.2,
                b as f32 + 0.9 * rand::random::<f32>());

            let overlaps = placed.iter().any(|(c, r)| {
                (center - *c).length() < 0.2 + r + 0.05
            });
            if overlaps { continue; }
            placed.push((center, 0.2));

            if material < 0.75 {
                let material = SolidColor::new(
                    rand::random::<f32>() * rand::random::<f32>(),
                    rand::random::<f32>() * rand::random::<f32>(),
                    rand::random::<f32>() * rand::random::<f32>());
                    // let roughness = rand::distributions::Uniform::new(0.0, 1.0);
                    let random_idx = mat!(materials, Diffuse::new(material.into(), 0.0));
                    world.add(Sphere::new(center, 0.2, random_idx).into());
            } else if material < 0.95 {
                let material = Reflective::new(Vec3A::new(
                    0.5 * (1.0 * rand::random::<f32>()),
                    0.5 * (1.0 * rand::random::<f32>()),
                    0.5 * (1.0 * rand::random::<f32>())),
                    0.5 * rand::random::<f32>());
                    let random_idx = mat!(materials, material);
                    world.add(
                        Sphere::new(center, 0.2, random_idx).into());
            } else {
                    let refl_idx = mat!(materials, Refractive::new(1.5, Vec3A::ONE));
                    world.add(Sphere::new(center, 0.2, refl_idx).into());
                    let refr_idx = mat!(materials, Refractive::new(1.5, Vec3A::ONE));
                    world.add(Sphere::new(center, -0.19,refr_idx).into());
            }
        }
    }

    let red_idx = mat!(materials, Diffuse::new(SolidColor::new(0.75, 0.25, 0.25).into(), 0.0));
    world.add(Sphere::new(Vec3A::new(-2.0, 1.0, 0.0),
                          1.0,
                          red_idx).into());

    let refr_idx1 = mat!(materials, Refractive::new(1.5, Vec3A::ONE));
    world.add(Sphere::new(Vec3A::new(0.0, 1.0, 0.0),
                          1.0,
                          refr_idx1).into());

    world.add(Sphere::new(Vec3A::new(0.0, 1.0, 0.0),
                          -0.99,
                          refr_idx1).into());

    let refl_idx = mat!(materials, Reflective::new(Vec3A::new(0.5, 0.5, 0.5), 0.05));
    world.add(Sphere::new(Vec3A::new(2.0, 1.0, 0.0),
                          1.0,
                          refl_idx).into());

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    Scene::new(String::from("Random Spheres"), bvh, materials, camera, vec![], Some(environment), false)
}